use crate::qingyun::{
    encoder::简码覆盖, 不好的大集合键, 元素安排, 冰雪清韵决策, 冰雪清韵决策空间, 冰雪清韵编码信息,
    动态拆分项, 原始音节信息, 固定拆分项, 大集合, 小集合, 常用简繁范围, 拆分输入, 冰雪清韵条件,
    条件元素安排, 空格, 笔画, 编码, 转换, 进制, 音节信息, 频序, 频率,
};
use chai::{
    config::{条件, 安排, 广义码位, 安排描述, 配置},
    contexts::上下文,
    interfaces::{command_line::读取文本文件, 默认输入},
    objectives::metric::指法标记,
    元素, 原始当量信息, 原始键位分布信息, 棱镜, 错误,
};
use chrono::Local;
use core::panic;
use csv::WriterBuilder;
use indexmap::IndexMap;
use itertools::Itertools;
use regex::Regex;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Serialize;
use serde_yaml::{from_str, to_string};
use std::{
    cmp::Reverse,
    fs::{File, read_to_string},
    io::Write,
    path::PathBuf,
};

pub fn 写入文本文件<I, T>(path: PathBuf, content: T)
where
    I: Serialize,
    T: IntoIterator<Item = I>,
{
    let mut writer = WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .flexible(true)
        .from_path(path)
        .unwrap();
    for item in content {
        writer.serialize(item).unwrap();
    }
    writer.flush().unwrap();
}

#[derive(Clone)]
pub struct 冰雪清韵上下文 {
    pub 配置: 配置,
    pub 棱镜: 棱镜,
    pub 初始决策: 冰雪清韵决策,
    pub 决策空间: 冰雪清韵决策空间,
    pub 原始键位分布信息: 原始键位分布信息,
    pub 原始当量信息: 原始当量信息,
    pub 固定拆分: Vec<固定拆分项>,
    pub 动态拆分: Vec<动态拆分项>,
    pub 块转数字: FxHashMap<String, usize>,
    pub 数字转块: FxHashMap<usize, String>,
    pub 简体顺序: Vec<usize>,
    pub 繁体顺序: Vec<usize>,
    pub 下游字根: FxHashMap<元素, Vec<元素>>,
    pub 拼音: Vec<音节信息>,
}

impl 上下文 for 冰雪清韵上下文 {
    type 决策 = 冰雪清韵决策;

    fn 序列化(&self, 解: &冰雪清韵决策) -> String {
        let mut 新配置 = self.配置.clone();
        新配置.info.as_mut().unwrap().version =
            Some(format!("{}", Local::now().format("%Y-%m-%d+%H:%M:%S")));
        let mut mapping = IndexMap::new();
        mapping.insert("补码-1".into(), 安排::Basic(解.补码键.into()));
        mapping.insert("主根-1".into(), 安排::Basic(解.第一主根.into()));
        mapping.insert("主根-2".into(), 安排::Basic(解.第二主根.into()));
        for (元素, 安排) in 解.元素.iter().enumerate() {
            let mapped: 安排 = 安排.to_mapped(&self.棱镜);
            if mapped != 安排::Unused(()) {
                mapping.insert(self.棱镜.数字转元素[&元素].clone(), mapped);
            }
        }
        新配置.form.mapping = mapping;
        to_string(&新配置).unwrap()
    }
}

impl 冰雪清韵上下文 {
    fn _采用冰雪四拼声母布局(决策: &mut 冰雪清韵决策, 棱镜: &棱镜) {
        let 声母列表 = ["声-zh", "声-ch", "声-sh", "声-0", "声-w", "声-y", "声-yu"];
        let 键位列表 = ['w', 'y', 'v', 'r', 's', 'f', 'k'];
        for (声母, 键位) in 声母列表.into_iter().zip(键位列表.into_iter()) {
            let 声母 = 棱镜.元素转数字[声母];
            决策.元素[声母] = 元素安排::键位(键位);
        }
    }

    fn _采用有理笔画(决策: &mut 冰雪清韵决策, 棱镜: &棱镜) {
        let 笔画读音 = [
            ("声-h", "韵-eng"),
            ("声-sh", "韵-u"),
            ("声-p", "韵-ie"),
            ("声-d", "韵-ian"),
            ("声-zh", "韵-e"),
            ("声-zh", "韵-e"),
        ];
        for (笔, (声母, 韵母)) in 笔画.into_iter().zip(笔画读音) {
            let 笔画数字 = 棱镜.元素转数字[笔];
            决策.元素[笔画数字] = 元素安排::声母韵母 {
                声母: 棱镜.元素转数字[声母],
                韵母: 棱镜.元素转数字[韵母],
            };
        }
    }

    pub fn 新建(输入: 默认输入) -> Result<Self, 错误> {
        let 布局 = 输入.配置.form.clone();
        let 原始决策 = 布局.mapping;
        let 原始决策空间 = 布局.mapping_space.unwrap();
        let 原始乱序生成器 = 布局.mapping_generators.unwrap();
        let 原始乱序生成器 = 原始乱序生成器[0].clone();
        let mut 元素转数字 = FxHashMap::default();
        let mut 数字转元素 = FxHashMap::default();
        let mut 键转数字 = FxHashMap::default();
        let mut 数字转键 = FxHashMap::default();
        let mut 序号 = 0;
        for c in 大集合.into_iter().chain(小集合.into_iter()) {
            序号 += 1;
            元素转数字.insert(c.to_string(), 序号);
            数字转元素.insert(序号, c.to_string());
            键转数字.insert(c, 序号 as u64);
            数字转键.insert(序号 as u64, c);
        }
        let 所有元素: Vec<String> = from_str(&read_to_string("data/rules.yaml").unwrap()).unwrap();
        for 元素 in &所有元素 {
            序号 += 1;
            元素转数字.insert(元素.clone(), 序号);
            数字转元素.insert(序号, 元素.clone());
        }
        let 棱镜 = 棱镜 {
            键转数字,
            数字转键,
            元素转数字,
            数字转元素,
            进制: 进制 as u64,
            可选元素位图索引: Default::default()
        };

        let mut 下游字根: FxHashMap<元素, Vec<_>> = FxHashMap::default();
        let 最大数量 = 棱镜.数字转元素.len() + 1;
        let mut 决策空间 = 冰雪清韵决策空间 {
            元素: vec![vec![]; 最大数量],
            声母: vec![],
            韵母: vec![],
            字根: vec![],
        };
        let 安排::Basic(补码键) = 原始决策["补码-1"].clone() else {
            panic!("补码键必须指定");
        };
        let 安排::Basic(第一主根) = 原始决策["主根-1"].clone() else {
            panic!("第一主根必须指定");
        };
        let 安排::Basic(第二主根) = 原始决策["主根-2"].clone() else {
            panic!("第二主根必须指定");
        };
        let mut 初始决策 = 冰雪清韵决策 {
            元素: vec![元素安排::未选取; 最大数量],
            补码键: 补码键.chars().next().unwrap(),
            第一主根: 第一主根.chars().next().unwrap(),
            第二主根: 第二主根.chars().next().unwrap(),
        };
        for 元素 in &所有元素 {
            let 序号 = 棱镜.元素转数字[元素];
            let 编码 = 原始决策.get(元素).unwrap_or(&安排::Unused(()));
            if ["补码-1", "主根-1", "主根-2"].contains(&元素.as_str()) {
                continue;
            }
            if 元素.starts_with("声") {
                决策空间.声母.push(序号);
                let 安排::Basic(编码) = 编码 else {
                    unreachable!();
                };
                let 键位 = 编码.chars().next().unwrap();
                初始决策.元素[序号] = 元素安排::键位(键位);
                match 元素.as_str() {
                    "声-zh" | "声-ch" | "声-sh" | "声-0" => {
                        决策空间.元素[序号] = 大集合
                            .into_iter()
                            .filter(|&c| !不好的大集合键.contains(&c))
                            .map(|键位| 元素安排::键位(键位).into())
                            .collect();
                    }
                    _ => {
                        决策空间.元素[序号] = vec![初始决策.元素[序号].clone().into()];
                    }
                }
            } else if 元素.starts_with("韵") {
                决策空间.韵母.push(序号);
                if let 安排::Grouped { element } = 编码.clone() {
                    初始决策.元素[序号] = 元素安排::归并(棱镜.元素转数字[&element]);
                    决策空间.元素[序号] = vec![初始决策.元素[序号].clone().into()];
                } else {
                    let 安排::Basic(编码) = 编码 else {
                        println!("元素 {} 的编码不是 Basic 或 Grouped", 元素);
                        unreachable!();
                    };
                    let 键位 = 编码.chars().next().unwrap();
                    初始决策.元素[序号] = 元素安排::键位(键位);
                    match 元素.as_str() {
                        "韵-a" | "韵-e" | "韵-i" | "韵-o" | "韵-u" => {
                            决策空间.元素[序号] = vec![初始决策.元素[序号].clone().into()];
                        }
                        "韵-an" | "韵-en" | "韵-ang" | "韵-eng" => {
                            决策空间.元素[序号] = [';', ',', '.', '/']
                                .into_iter()
                                .map(|键位| 元素安排::键位(键位).into())
                                .collect();
                        }
                        _ => {
                            决策空间.元素[序号] = ['a', 'e', 'i', 'o', 'u']
                                .into_iter()
                                .map(|键位| {
                                    if 元素 == "韵-ü" && 键位 == 'u' {
                                        条件元素安排 {
                                            安排: 元素安排::键位(键位),
                                            条件列表: vec![],
                                            打分: -1.0,
                                        }
                                    } else {
                                        元素安排::键位(键位).into()
                                    }
                                })
                                .collect();
                        }
                    }
                }
            } else {
                决策空间.字根.push(序号);
                let mut 原始安排列表 = 原始决策空间.get(元素).cloned().unwrap_or(vec![]);
                let 当前决策 = 原始决策.get(元素).unwrap_or(&安排::Unused(()));
                let 当前决策为乱序 = if let 安排::Advanced(v) = 当前决策 {
                    if let 广义码位::Ascii(_) = v[0] {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                if !当前决策为乱序 && !原始安排列表.iter().any(|x| &x.value == 当前决策)
                {
                    原始安排列表.insert(
                        0,
                        安排描述 {
                            value: 当前决策.clone(),
                            score: 0.0,
                            condition: None,
                        },
                    );
                }
                let mut 安排列表 = vec![];
                for 原始安排 in &原始安排列表 {
                    let 字根安排 = 元素安排::from(&原始安排.value, &棱镜);
                    let mut 原始条件 = 原始安排.condition.clone().unwrap_or_default();
                    let 归并字根 = if let 元素安排::归并(字根) = &字根安排 {
                        Some(字根.clone())
                    } else if let 元素安排::归并韵母 { 字根, .. } = &字根安排 {
                        Some(字根.clone())
                    } else {
                        None
                    };
                    if let Some(归并字根) = 归并字根 {
                        let 默认条件 = 条件 {
                            element: 棱镜.数字转元素[&归并字根].clone(),
                            op: "不是".to_string(),
                            value: 安排::Unused(()),
                        };
                        if !原始条件.iter().any(|x| x == &默认条件) {
                            原始条件.push(默认条件);
                        }
                    }
                    let 条件列表: Vec<冰雪清韵条件> = 原始条件
                        .into_iter()
                        .map(|c| 冰雪清韵条件 {
                            元素: 棱镜.元素转数字[&c.element],
                            谓词: c.op == "是",
                            值: 元素安排::from(&c.value, &棱镜),
                        })
                        .collect();
                    for 条件 in &条件列表 {
                        if 下游字根.contains_key(&条件.元素) {
                            if !下游字根[&条件.元素].contains(&序号) {
                                下游字根.get_mut(&条件.元素).unwrap().push(序号);
                            }
                        } else {
                            下游字根.insert(条件.元素.clone(), vec![序号]);
                        }
                    }
                    let 条件字根安排 = 条件元素安排 {
                        安排: 字根安排,
                        条件列表,
                        打分: 原始安排.score,
                    };
                    安排列表.push(条件字根安排);
                }
                // 第一主根
                if 笔画.contains(&元素.as_str()) {
                    for 键位 in 大集合 {
                        let 安排 = 元素安排::键位第一(键位);
                        if 安排列表.iter().any(|x| x.安排 == 安排) {
                            continue;
                        }
                        if 元素 == "1" && !['d', 'f', 'j', 'k'].contains(&键位) {
                            continue;
                        }
                        安排列表.push(条件元素安排 {
                            安排,
                            条件列表: vec![],
                            打分: 0.0,
                        });
                    }
                }
                // 第二主根
                let regex = Regex::new(&原始乱序生成器.regex).unwrap();
                if regex.is_match(&元素) {
                    let 条件列表 = 安排列表
                        .iter()
                        .find(|x| matches!(x.安排, 元素安排::声母韵母 { .. }))
                        .map(|x| &x.条件列表)
                        .unwrap_or(&vec![])
                        .clone();
                    for 键位 in 大集合 {
                        let 安排 = 元素安排::键位第二(键位);
                        if 安排列表.iter().any(|x| {
                            if x.安排 == 安排 {
                                true
                            } else if let 元素安排::声母韵母 { 声母, .. } = x.安排 {
                                初始决策.元素[声母] == 元素安排::键位(键位)
                            } else {
                                false
                            }
                        }) {
                            continue;
                        }
                        安排列表.push(条件元素安排 {
                            安排,
                            条件列表: 条件列表.clone(),
                            打分: 0.0,
                        });
                    }
                }
                let 安排列表: Vec<_> = 安排列表.into_iter().collect();
                初始决策.元素[序号] = 元素安排::from(当前决策, &棱镜);
                决策空间.元素[序号] = 安排列表;
            }
        }

        let mut 所有第一主根键位 = vec![];
        let mut 所有第二主根键位 = vec![];
        for 安排 in 初始决策.元素.iter() {
            if let 元素安排::键位第一(键位) = 安排 {
                所有第一主根键位.push(*键位);
            } else if let 元素安排::键位第二(键位) = 安排 {
                所有第二主根键位.push(*键位);
            }
        }
        assert!(
            所有第一主根键位.len() == 6,
            "初始决策中的乱序键位不完整: {所有第一主根键位:?}"
        );
        assert!(
            所有第二主根键位.len() == 21 && 大集合.iter().all(|c| 所有第二主根键位.contains(&c)),
            "初始决策中的乱序键位不完整: {所有第二主根键位:?}"
        );

        let (固定拆分, 动态拆分, 块转数字, 数字转块, 简体顺序, 繁体顺序) =
            Self::解析动态拆分(&棱镜, &决策空间);
        let 拼音 = Self::读取拼音(&棱镜);
        Ok(Self {
            配置: 输入.配置,
            棱镜,
            初始决策,
            决策空间,
            原始键位分布信息: 输入.原始键位分布信息,
            原始当量信息: 输入.原始当量信息,
            固定拆分,
            动态拆分,
            块转数字,
            数字转块,
            简体顺序,
            繁体顺序,
            下游字根,
            拼音,
        })
    }

    pub fn 预处理当量信息(&self) -> Vec<f32> {
        let mut 当量信息 = vec![0.0; 编码::编码空间大小()];
        let s = vec![];
        let 空格1 = 空格 as u8;
        let 进制1 = 进制 as u8;
        for c1 in 0..空格1 {
            let mut s1 = s.clone();
            self.棱镜.数字转键.get(&(c1 as u64)).map(|z| s1.push(*z));
            for c2 in 0..空格1 {
                let mut s2 = s1.clone();
                self.棱镜.数字转键.get(&(c2 as u64)).map(|z| s2.push(*z));
                for c3 in 0..空格1 {
                    let mut s3 = s2.clone();
                    self.棱镜.数字转键.get(&(c3 as u64)).map(|z| s3.push(*z));
                    for c4 in 0..进制1 {
                        let mut s4 = s3.clone();
                        self.棱镜.数字转键.get(&(c4 as u64)).map(|z| s4.push(*z));
                        let c = [c1, c2, c3, c4].hash();
                        for range in [0..2, 1..3, 2..4, 0..3, 1..4, 0..4] {
                            if range.end > s4.len() {
                                continue;
                            }
                            let substr: String = s4[range].iter().collect();
                            当量信息[c as usize] +=
                                *self.原始当量信息.get(&substr).unwrap_or(&0.0) as f32;
                        }
                    }
                }
            }
        }
        当量信息
    }

    fn 读取拼音(棱镜: &棱镜) -> Vec<音节信息> {
        let 原始拼音: Vec<原始音节信息> = 读取文本文件("data/pinyin.txt".into());
        let mut 拼音 = Vec::new();
        for 原始音节信息 {
            声母, 韵母, 频率,
        ..
        } in &原始拼音
        {
            let 声母 = format!("声-{声母}");
            let 韵母 = format!("韵-{韵母}");
            if !棱镜.元素转数字.contains_key(&声母) {
                panic!("拼音声母 {} 不在棱镜中", 声母);
            }
            if !棱镜.元素转数字.contains_key(&韵母) {
                panic!("拼音韵母 {} 不在棱镜中", 韵母);
            }
            let 声母 = 棱镜.元素转数字[&声母];
            let 韵母 = 棱镜.元素转数字[&韵母];
            拼音.push(音节信息 {
                声母,
                韵母,
                频率: *频率 as 频率,
            });
        }
        let 总频率: 频率 = 拼音.iter().map(|x| x.频率).sum();
        for 音节 in &mut 拼音 {
            音节.频率 /= 总频率;
        }
        拼音
    }

    fn 对齐(列表: Vec<元素>, 默认值: 元素) -> [元素; 4] {
        [0, 1, 2, 3].map(|i| {
            if i == 3 && 列表.len() > 3 {
                列表[列表.len() - 1]
            } else if i < 列表.len() {
                列表[i]
            } else {
                默认值
            }
        })
    }

    pub fn 解析动态拆分(
        棱镜: &棱镜,
        决策空间: &冰雪清韵决策空间,
    ) -> (
        Vec<固定拆分项>,
        Vec<动态拆分项>,
        FxHashMap<String, usize>,
        FxHashMap<usize, String>,
        Vec<usize>,
        Vec<usize>,
    ) {
        let 拆分输入: 拆分输入 =
            from_str(&read_to_string("data/dynamic_analysis.yaml").unwrap()).unwrap();
        let 繁体字频: FxHashMap<char, u64> = 读取文本文件(PathBuf::from("data/ftzp.txt"));
        let 陆标转台标: FxHashMap<char, char> = 读取文本文件(PathBuf::from("data/t2tw.txt"));
        let mut 动态拆分 = vec![];
        let mut 块转数字 = FxHashMap::default();
        let mut 数字转块 = FxHashMap::default();
        for (块, 原始拆分方式列表) in 拆分输入.动态拆分 {
            let 块序号 = 动态拆分.len();
            块转数字.insert(块.clone(), 块序号);
            数字转块.insert(块序号, 块.clone());
            let mut 拆分方式列表 = vec![];
            for 原始拆分方式 in &原始拆分方式列表 {
                for 拆分方式 in 原始拆分方式 {
                    assert!(
                        棱镜.元素转数字.contains_key(拆分方式),
                        "元素 {} 不在棱镜中",
                        拆分方式
                    );
                }
                let 拆分方式 = Self::对齐(
                    原始拆分方式
                        .iter()
                        .map(|字根| 棱镜.元素转数字[字根])
                        .collect(),
                    0,
                );
                拆分方式列表.push(拆分方式);
            }
            // 检查原始拆分方式列表的最后一项都是必选字根
            let 最后一项 = 原始拆分方式列表.last().unwrap();
            if !最后一项.iter().all(|x| {
                !决策空间.元素[棱镜.元素转数字[x]]
                    .iter()
                    .any(|x| x.安排 == 元素安排::未选取)
            }) {
                panic!("动态拆分方式列表的最后一项必须都是必选字根, {块:?}, {原始拆分方式列表:?}");
            }
            动态拆分.push(拆分方式列表);
        }
        let mut 固定拆分 = vec![];
        let mut 简体总频数: u64 = 0;
        let mut 繁体总频数: u64 = 0;
        let 国字常用: FxHashSet<char> = 拆分输入
            .固定拆分
            .iter()
            .filter(|x| x.国字常用)
            .map(|x| x.汉字)
            .collect();
        for 词 in &拆分输入.固定拆分 {
            let 字块 = Self::对齐(词.拆分.iter().map(|块| 块转数字[块]).collect(), usize::MAX);
            let mut 简体频率 = 0.0;
            let mut 繁体频率 = 0.0;
            if 词.gb2312 {
                简体频率 = 词.频率 as 频率;
                简体总频数 += 词.频率;
            }
            let mut 陆标 = false;
            if 词.国字常用 {
                繁体频率 = *繁体字频.get(&词.汉字).unwrap_or(&0) as 频率;
                繁体总频数 += 繁体频率 as u64;
            } else {
                if let Some(&台标) = 陆标转台标.get(&词.汉字) {
                    if 国字常用.contains(&台标) {
                        繁体频率 = *繁体字频.get(&台标).unwrap_or(&0) as 频率;
                        繁体总频数 += 繁体频率 as u64;
                        陆标 = true;
                    }
                }
            }
            固定拆分.push(固定拆分项 {
                词: 词.汉字,
                简体频率,
                简体频序: 频序::MAX,
                繁体频率,
                繁体频序: 频序::MAX,
                通打频率: 0.0,
                字块,
                通规: 词.通规 > 0,
                gb2312: 词.gb2312,
                国字常用: 词.国字常用,
                陆标,
            });
        }
        // 归一化频率
        for 项 in &mut 固定拆分 {
            项.简体频率 /= 简体总频数 as 频率;
            项.繁体频率 /= 繁体总频数 as 频率;
            项.通打频率 = (项.简体频率 + 项.繁体频率) / 2.0;
        }
        固定拆分.sort_by(|a, b| {
            b.通打频率
                .partial_cmp(&a.通打频率)
                .unwrap()
                .then_with(|| (b.国字常用 || b.陆标).cmp(&(a.国字常用 || a.陆标)))
                .then_with(|| (b.gb2312).cmp(&(a.gb2312)))
        });
        for i in 0..常用简繁范围 {
            assert!(
                固定拆分[i].gb2312 || 固定拆分[i].国字常用 || 固定拆分[i].陆标,
                "前 {} 个字中第 {} 个字既不是简体常用字也不是繁体常用字: {:?}",
                常用简繁范围,
                i + 1,
                固定拆分[i]
            );
        }
        for i in 常用简繁范围..固定拆分.len() {
            assert!(
                !固定拆分[i].gb2312 && !固定拆分[i].国字常用 && !固定拆分[i].陆标,
                "第 {} 个字是简体或繁体常用字: {:?}",
                i + 1,
                固定拆分[i]
            );
        }
        let 简体顺序: Vec<_> = 固定拆分
            .iter()
            .enumerate()
            .filter(|(_, x)| x.gb2312)
            .sorted_by(|(_, a), (_, b)| b.简体频率.partial_cmp(&a.简体频率).unwrap())
            .map(|(i, _)| i)
            .collect();
        let 繁体顺序: Vec<_> = 固定拆分
            .iter()
            .enumerate()
            .filter(|(_, x)| x.国字常用 || x.陆标)
            .sorted_by(|(_, a), (_, b)| b.繁体频率.partial_cmp(&a.繁体频率).unwrap())
            .map(|(i, _)| i)
            .collect();
        for (简体频序, 索引) in 简体顺序.iter().enumerate() {
            固定拆分[*索引].简体频序 = 简体频序 as 频序;
        }
        for (繁体频序, 索引) in 繁体顺序.iter().enumerate() {
            固定拆分[*索引].繁体频序 = 繁体频序 as 频序;
        }
        (固定拆分, 动态拆分, 块转数字, 数字转块, 简体顺序, 繁体顺序)
    }

    fn 转编码(&self, code: 编码) -> String {
        code.iter()
            .filter_map(|x| self.棱镜.数字转键.get(&(*x as u64)))
            .cloned()
            .collect()
    }

    fn 排序编码(&self, res: &Vec<Regex>, code: &String) -> (usize, usize, Vec<usize>) {
        let order = "bpmfdtnlgkhjqxzcsrvwyaoeiu;,./";
        let mut category = usize::MAX;
        for (index, re) in res.iter().enumerate() {
            if re.is_match(code) {
                category = index;
                break;
            }
        }
        let length = code.len();
        let orders = code
            .chars()
            .map(|c| order.find(c).unwrap_or(usize::MAX))
            .collect();
        (category, length, orders)
    }

    pub fn 生成码表(&self, 编码结果: &[冰雪清韵编码信息], 目录: Option<PathBuf>) {
        let 目录 = 目录.unwrap_or_else(|| PathBuf::from("output"));
        let mut 宇浩测评码表 = Vec::new();
        let mut 大竹码表 = Vec::new();
        let mut 形码盒子测评码表 = Vec::new();
        let mut 未排序固态词典码表 = FxHashMap::default();
        let mut 已占据编码 = FxHashSet::default();
        let mut 当前最短码长 = FxHashMap::default();
        for (可编码对象, 编码信息) in self
            .固定拆分
            .iter()
            .zip(编码结果)
            .sorted_by_key(|(_, x)| x.简体频序)
        {
            let 全码 = self.转编码(编码信息.计重全码);
            if 可编码对象.gb2312 && 全码.len() == 3 {
                已占据编码.insert(全码.clone());
            }
            let 无空格全码 = 全码.replace("_", "");
            let 带空格全码 = 全码.replace("_", " ");
            let 简码 = self.转编码(编码信息.简体简码);
            当前最短码长.insert(可编码对象.词, 简码.len());
            let 无空格简码 = 简码.replace("_", "");
            let 带空格简码 = 简码.replace("_", " ");
            宇浩测评码表.push((可编码对象.词.to_string(), 无空格全码.clone()));
            大竹码表.push((全码.clone(), 可编码对象.词.to_string()));
            形码盒子测评码表.push((可编码对象.词, 带空格全码.clone()));
            if 可编码对象.gb2312 {
                未排序固态词典码表
                    .entry(无空格全码.clone())
                    .or_insert_with(Vec::new)
                    .push(可编码对象.词.to_string());
            }
            if !无空格简码.is_empty() && 无空格简码 != 无空格全码 {
                宇浩测评码表.push((可编码对象.词.to_string(), 无空格简码.clone()));
                大竹码表.push((简码.clone(), 可编码对象.词.to_string()));
                形码盒子测评码表.push((可编码对象.词, 带空格简码.clone()));
                if 可编码对象.gb2312 && self.转编码(编码信息.简体简码).len() > 1 {
                    未排序固态词典码表
                        .entry(无空格简码.clone())
                        .or_insert_with(Vec::new)
                        .push(可编码对象.词.to_string());
                }
            }
        }
        let 拆分结果: Vec<(String, String)> =
            读取文本文件(PathBuf::from("data/拆分结果.txt"));
        for (字, 拆分) in 拆分结果 {
            大竹码表.push((format!("拆分［{}］", 拆分.clone()), 字));
        }
        self.后处理固态词典码表(
            &mut 未排序固态词典码表,
            &mut 大竹码表,
            &mut 宇浩测评码表,
            &mut 已占据编码,
            &当前最短码长,
        );
        let re1 = Regex::new(r"^[bpmfdtnlgkhjqxzcsrvwy]_?$").unwrap();
        let re2 = Regex::new(r"^[bpmfdtnlgkhjqxzcsrvwy][aoeiu;,./]$").unwrap();
        let re3 = Regex::new(r"^[bpmfdtnlgkhjqxzcsrvwy]{2}_?$").unwrap();
        let re4 = Regex::new(r"^[bpmfdtnlgkhjqxzcsrvwy]{2}[aoeiu;,./]$").unwrap();
        let re5 = Regex::new(r"^[bpmfdtnlgkhjqxzcsrvwy]{3}_?$").unwrap();
        let re6 = Regex::new(r"^[bpmfdtnlgkhjqxzcsrvwy]{3}[aoeiu;,./]$").unwrap();
        let re7 = Regex::new(r"^[bpmfdtnlgkhjqxzcsrvwy]{4}$").unwrap();
        let res = vec![re1, re2, re3, re4, re5, re6, re7];
        let 固态词典码表: Vec<_> = 未排序固态词典码表
            .into_iter()
            .sorted_by_key(|(编码字符串, _)| self.排序编码(&res, &编码字符串))
            .map(|(字符串, 词列表)| (字符串, 词列表.join(" ")))
            .collect();
        大竹码表.sort_by_key(|(code, _)| self.排序编码(&res, code));
        写入文本文件(目录.join("冰雪清韵.txt"), 宇浩测评码表);
        写入文本文件(目录.join("大竹码表.txt"), 大竹码表);
        写入文本文件(目录.join("形码盒子测评码表.txt"), 形码盒子测评码表);
        写入文本文件(目录.join("snow_qingyun.fixed.txt"), 固态词典码表);
    }

    fn 读取简词(
        &self,
    ) -> (
        Vec<(String, String, u64)>,
        FxHashMap<(char, char), Vec<String>>,
    ) {
        let 拼音: Vec<(String, String, String, u64)> = 读取文本文件("data/pinyin.txt".into());
        let mut 声韵映射 = FxHashMap::default();
        for (全拼, 声母, 韵母, _) in 拼音 {
            let 声母 = format!("声-{}", 声母);
            let 元素安排::键位(声母按键) = self.初始决策.元素[self.棱镜.元素转数字[&声母]]
            else {
                panic!("声母 {} 不是键位安排", 声母);
            };
            let 韵母 = format!("韵-{}", 韵母);
            let 元素安排::键位(韵母按键) = self.初始决策.元素[self.棱镜.元素转数字[&韵母]]
            else {
                panic!("韵母 {} 不是键位安排", 韵母);
            };
            声韵映射.insert(全拼, (声母按键, 韵母按键));
        }
        let 简词列表: Vec<(String, String, u64)> = 读取文本文件("data/简词.txt".into());
        let mut 简词编码列表 = vec![];
        for (简词, 全拼列表, 词频) in 简词列表 {
            let 全拼列表: Vec<_> = 全拼列表
                .split(' ')
                .map(|x| x[..(x.len() - 1)].to_string())
                .collect();
            let 编码1 = 声韵映射[&全拼列表[0]].0;
            let 编码2 = 声韵映射[&全拼列表[1]].0;
            let 编码3 = 声韵映射[&全拼列表[1]].1;
            let 编码: String = [编码1, 编码2, 编码3].iter().collect();
            简词编码列表.push((简词, 编码, 词频));
        }
        let mut 简词编码排序 = FxHashMap::default();
        for 第一码 in 大集合 {
            for 第二码 in 大集合 {
                let pair = (第一码, 第二码);
                let mut list = vec![];
                for &第三码 in &小集合[1..] {
                    let 编码: String = vec![第一码, 第二码, 第三码].iter().collect();
                    list.push(编码);
                }
                list.sort_by_key(|code| (self.原始当量信息[code] * 100.0) as u64);
                简词编码排序.insert(pair, list);
            }
        }
        (简词编码列表, 简词编码排序)
    }

    pub fn 后处理固态词典码表(
        &self,
        固态词典码表: &mut FxHashMap<String, Vec<String>>,
        大竹码表: &mut Vec<(String, String)>,
        宇浩测评码表: &mut Vec<(String, String)>,
        已占据编码: &mut FxHashSet<String>,
        当前最短码长: &FxHashMap<char, usize>,
    ) {
        let 简码覆盖: 简码覆盖 = from_str(&read_to_string("data/override.yaml").unwrap()).unwrap();
        let (mut 简词编码列表, _) = self.读取简词();
        简词编码列表.sort_by_key(|(简词, _, 词频)| {
            let chars: Vec<_> = 简词.chars().collect();
            let total_length = 当前最短码长[&chars[0]] + 当前最短码长[&chars[1]];
            Reverse((total_length as i64 - 3) * (*词频 as i64))
        });
        // let 词频映射: FxHashMap<String, u64> = 简词编码列表
        //     .iter()
        //     .map(|(简词, _, 词频)| (简词.clone(), *词频))
        //     .collect();
        for (简词, 编码) in 简码覆盖.简词快符.clone() {
            固态词典码表.insert(编码.clone(), vec![简词.clone()]);
        }
        let mut 简词映射 = FxHashMap::default();
        for (词, 编码) in 简码覆盖.二简词.clone() {
            assert!(!已占据编码.contains(&编码), "简词 {词} 的 {编码} 已被占据",);
            简词映射.insert(编码.clone(), 词.clone());
            已占据编码.insert(编码.clone());
        }
        let mut 未编码简词列表 = vec![];
        let mut 自动简词 = 0;
        for (简词, 编码, _) in 简词编码列表.iter() {
            // 跳过一简词
            if 简码覆盖.简词快符.iter().any(|(s, _)| s == 简词) {
                continue;
            }
            // 跳过已手动编码的二简词
            if 简码覆盖.二简词.contains_key(简词) {
                continue;
            }
            let chars: Vec<_> = 简词.chars().collect();
            let total_length = 当前最短码长[&chars[0]] + 当前最短码长[&chars[1]];
            if total_length <= 3 {
                println!(
                    "简词 {} 的编码 {} 被跳过，因为其两字的最短码长之和为 {}",
                    简词, 编码, total_length
                );
                continue;
            }
            if 已占据编码.contains(编码) {
                未编码简词列表.push((简词, 编码));
                continue;
            }
            自动简词 += 1;
            简词映射.insert(编码.clone(), 简词.clone());
            已占据编码.insert(编码.clone());
        }
        println!(
            "二简词编码分配完成：手动 {} 个，自动 {} 个，共 {} 个",
            简码覆盖.二简词.len(),
            自动简词,
            简码覆盖.二简词.len() + 自动简词
        );
        println!(
            "未能分配编码的二简词前 100：{:?}",
            未编码简词列表.iter().take(100).collect::<Vec<_>>()
        );
        for (编码, 简词) in 简词映射.clone() {
            固态词典码表
                .entry(编码.clone())
                .or_insert_with(Vec::new)
                .insert(0, 简词.clone());
        }
        let mut 大竹码表增加 = vec![];
        let mut 宇浩码表增加 = vec![];
        for (简词, 编码) in 简码覆盖.简词快符 {
            大竹码表增加.push((编码.clone(), 简词.clone()));
            宇浩码表增加.push((简词.clone(), 编码.clone()));
        }
        for (编码, 简词) in 简词映射 {
            大竹码表增加.push((编码.clone(), 简词.clone()));
            宇浩码表增加.push((简词.clone(), 编码.clone()));
        }
        大竹码表增加.sort_by_key(|(code, _)| self.排序编码(&vec![], code));
        宇浩码表增加.sort_by_key(|(_, code)| self.排序编码(&vec![], code));
        大竹码表.splice(0..0, 大竹码表增加);
        宇浩测评码表.splice(0..0, 宇浩码表增加);
        let 数字信息 = [
            ('1', '一', "yi"),
            ('2', '二', "vi"),
            ('3', '三', "s;"),
            ('4', '四', "si"),
            ('5', '五', "wu"),
            ('6', '六', "la"),
            ('7', '七', "qi"),
            ('8', '八', "ba"),
            ('9', '九', "ja"),
            ('0', '零', "l/"),
        ];
        for (数字, 汉字, 编码) in 数字信息 {
            let 条目 = &mut 固态词典码表
                .entry(编码.to_string())
                .or_insert_with(Vec::new);
            条目.push(数字.to_string());
            条目.push(汉字.to_string());
        }

        for key in "bpmfdtnlgkhjqxzcsrvwy".chars() {
            let 编码 = format!("{}.", key);
            let 条目 = &mut 固态词典码表.entry(编码.clone()).or_insert_with(Vec::new);
            if 条目.is_empty() {
                条目.push("🈚️".to_string());
            }
            条目.push(key.to_uppercase().to_string());
            条目.push(key.to_string());
        }
        for key in "aoeiu".chars() {
            let 编码 = format!("m{}", key);
            let 条目 = &mut 固态词典码表.entry(编码.clone()).or_insert_with(Vec::new);
            条目.push(key.to_uppercase().to_string());
            条目.push(key.to_string());
        }
    }

    pub fn 翻转码表(
        &self,
        编码结果: &[冰雪清韵编码信息],
        顺序: &Vec<usize>,
        频率: &impl Fn(&冰雪清韵编码信息) -> 频率,
    ) -> Vec<(编码, Vec<char>, 频率)> {
        let mut 翻转码表 = FxHashMap::default();
        let mut 重码组列表 = vec![];
        for 索引 in 顺序 {
            翻转码表
                .entry(编码结果[*索引].计重全码)
                .or_insert_with(|| vec![])
                .push((self.固定拆分[*索引].词, 频率(&编码结果[*索引])));
        }
        for (全码, 重码组) in 翻转码表 {
            if 重码组.len() > 1 {
                let 总频率: 频率 = 重码组[1..].iter().map(|x| x.1).sum();
                重码组列表.push((全码, 重码组.iter().map(|x| x.0).collect(), 总频率));
            }
        }
        重码组列表.sort_by_key(|(_, _, 频率)| std::cmp::Reverse((*频率 * 1_000_000.0) as u64));
        重码组列表
    }

    // 分析前 3000 字中全码重码和简码差指法的情况
    pub fn 分析码表(
        &self,
        编码结果: &[冰雪清韵编码信息],
        目录: Option<PathBuf>,
    ) -> Result<(), 错误> {
        let 分析路径 = 目录
            .unwrap_or_else(|| PathBuf::from("output"))
            .join("分析.md");
        let mut 文件 = File::create(分析路径).unwrap();
        let mut 二码字根字 = FxHashMap::default();
        let mut 三码字根字 = FxHashMap::default();
        let mut 无理一简多重 = FxHashMap::default();
        let mut 二简 = FxHashMap::default();
        for (序号, 编码信息) in 编码结果.iter().enumerate() {
            let 词 = self.固定拆分[序号].词;
            let 频率 = 编码信息.简体频率 * 10000.0;
            if !self.固定拆分[序号].通规 {
                continue;
            }
            if 编码信息.字根字 {
                let 全码 = self.转编码(编码信息.全码);
                let 第一码 = 全码.chars().next().unwrap();
                if 编码信息.全码 == 编码信息.简体简码 {
                    二码字根字
                        .entry(第一码)
                        .or_insert_with(Vec::new)
                        .push(format!("{词} {全码} {频率:.0}"));
                } else {
                    三码字根字
                        .entry(第一码)
                        .or_insert_with(Vec::new)
                        .push(format!("{词} {全码} {频率:.0}"));
                }
            } else {
                let 简码 = self.转编码(编码信息.简体简码);
                if 简码.len() == 2 {
                    let 第一码 = 简码.chars().next().unwrap();
                    无理一简多重
                        .entry(第一码)
                        .or_insert_with(Vec::new)
                        .push(format!("{词} {简码} {频率:.0}"));
                } else if 简码.len() == 3 && 简码.ends_with("_") {
                    let 第一码 = 简码.chars().next().unwrap();
                    二简
                        .entry(第一码)
                        .or_insert_with(Vec::new)
                        .push(format!("{词} {简码} {频率:.0}"));
                }
            }
        }
        writeln!(
            文件,
            "# 无理一简多重 {}\n",
            无理一简多重.values().map(|x| x.len()).sum::<usize>()
        )?;
        for (第一码, 列表) in 无理一简多重.iter().sorted_by_key(|x| x.0) {
            writeln!(文件, "- {第一码}: {}", 列表.join(" "))?;
        }
        writeln!(
            文件,
            "\n# 二码字根字 {}\n",
            二码字根字.values().map(|x| x.len()).sum::<usize>()
        )?;
        for (第一码, 列表) in 二码字根字.iter().sorted_by_key(|x| x.0) {
            writeln!(文件, "- {第一码}: {}", 列表.join(" "))?;
        }
        writeln!(
            文件,
            "\n# 二简 {}\n",
            二简.values().map(|x| x.len()).sum::<usize>()
        )?;
        for (第一码, 列表) in 二简.iter().sorted_by_key(|x| x.0) {
            writeln!(文件, "- {第一码}: {}", 列表.join(" "))?;
        }
        writeln!(
            文件,
            "\n# 三码字根字 {}\n",
            三码字根字.values().map(|x| x.len()).sum::<usize>()
        )?;
        for (第一码, 列表) in 三码字根字.iter().sorted_by_key(|x| x.0) {
            writeln!(文件, "- {第一码}: {}", 列表.join(" "))?;
        }
        let 简体前三千: Vec<_> = self.简体顺序.iter().take(3000).cloned().collect();
        let 繁体前三千: Vec<_> = self.繁体顺序.iter().take(3000).cloned().collect();
        let 通打前三千: Vec<_> = (0..3000).collect();
        let 指法标记 = 指法标记::new();
        let mut 差指法 = vec![];
        let mut 四键字 = vec![];
        let mut 三键字 = vec![];
        for &序号 in 简体前三千.iter() {
            let 编码信息 = &编码结果[序号];
            let 词 = self.固定拆分[序号].词;
            let 简码 = self.转编码(编码信息.简体简码);
            if 简码.len() == 3 && 序号 < 200 {
                三键字.push((词, 简码.clone(), 编码信息.简体频率));
            }
            if 简码.len() == 4 && 序号 < 500 {
                四键字.push((词, 简码.clone(), 编码信息.简体频率));
            }
            if 序号 < 1500 {
                let 简码: Vec<char> = 简码.chars().collect();
                for 键索引 in 0..(简码.len() - 1) {
                    let 组合 = (简码[键索引], 简码[键索引 + 1]);
                    if 指法标记.同指大跨排.contains(&组合) || 指法标记.错手.contains(&组合)
                    {
                        差指法.push((词, 简码.iter().collect::<String>()));
                        break;
                    }
                }
            }
        }
        writeln!(文件, "\n# 前 1500 中简码差指法项\n")?;
        for (字, 编码) in 差指法 {
            write!(文件, "{字} {编码}；")?;
        }
        writeln!(文件, "\n\n# 前 200 中三键字\n").unwrap();
        for (字, 编码, 频率) in 三键字 {
            write!(文件, "{字} {编码} {:.0}；", 频率 * 10000.0)?;
        }
        writeln!(文件, "\n\n# 前 500 中四键字\n").unwrap();
        for (字, 编码, 频率) in 四键字 {
            write!(文件, "{字} {编码} {:.0}；", 频率 * 10000.0)?;
        }
        writeln!(文件, "")?;
        let 简体重码组列表 = self.翻转码表(编码结果, &简体前三千, &|x| x.简体频率);
        let 繁体重码组列表 = self.翻转码表(编码结果, &繁体前三千, &|x| x.繁体频率);
        let 通打重码组列表 = self.翻转码表(编码结果, &通打前三千, &|x| x.通打频率);
        for (label, 重码组列表) in [
            ("简体", 简体重码组列表),
            ("繁体", 繁体重码组列表),
            ("通打", 通打重码组列表),
        ] {
            writeln!(文件, "\n# 前 3000 中{label}全码重码\n")?;
            for (全码, 重码组, 次选频率) in 重码组列表 {
                let 全码 = self.转编码(全码);
                let 百万分之频率 = 次选频率 * 1_000_000.0;
                writeln!(文件, "- {全码} {重码组:?} [{百万分之频率:.2} μ]")?;
            }
        }
        Ok(())
    }
}
