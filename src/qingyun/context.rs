use crate::qingyun::{
    不好的大集合键, 元素安排, 冰雪清韵决策, 冰雪清韵决策空间, 冰雪清韵编码信息, 动态拆分项,
    原始音节信息, 固定拆分项, 大集合, 小集合, 常用简繁范围, 拆分输入, 条件, 条件元素安排, 笔画,
    编码, 进制, 音节信息, 频序, 频率,
};
use chai::{
    config::{Condition, Mapped, ValueDescription, 配置},
    contexts::上下文,
    interfaces::{command_line::读取文本文件, 默认输入},
    objectives::metric::指法标记,
    元素, 原始当量信息, 原始键位分布信息, 棱镜, 码表项, 错误,
};
use chrono::Local;
use core::panic;
use indexmap::IndexMap;
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use serde_yaml::{from_str, to_string};
use std::{
    fs::{File, read_to_string},
    io::Write,
    path::PathBuf,
};

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
    type 解类型 = 冰雪清韵决策;

    fn 序列化(&self, 解: &冰雪清韵决策) -> String {
        let mut 新配置 = self.配置.clone();
        新配置.info.as_mut().unwrap().version =
            Some(format!("{}", Local::now().format("%Y-%m-%d+%H:%M:%S")));
        let mut mapping = IndexMap::new();
        mapping.insert("补码-1".into(), Mapped::Basic(解.补码键.into()));
        mapping.insert("主根-1".into(), Mapped::Basic(解.第一主根.into()));
        mapping.insert("主根-2".into(), Mapped::Basic(解.第二主根.into()));
        for (元素, 安排) in 解.元素.iter().enumerate() {
            let mapped: Mapped = 安排.to_mapped(&self.棱镜);
            if mapped != Mapped::Unused(()) {
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
        let 原始乱序生成器 = 布局.mapping_generator.unwrap();
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
        };

        let mut 下游字根: FxHashMap<元素, Vec<_>> = FxHashMap::default();
        let 最大数量 = 棱镜.数字转元素.len() + 1;
        let mut 决策空间 = 冰雪清韵决策空间 {
            元素: vec![vec![]; 最大数量],
            声母: vec![],
            韵母: vec![],
            字根: vec![],
        };
        let Mapped::Basic(补码键) = 原始决策["补码-1"].clone() else {
            panic!("补码键必须指定");
        };
        let Mapped::Basic(第一主根) = 原始决策["主根-1"].clone() else {
            panic!("第一主根必须指定");
        };
        let Mapped::Basic(第二主根) = 原始决策["主根-2"].clone() else {
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
            let 编码 = 原始决策.get(元素).unwrap_or(&Mapped::Unused(()));
            if ["补码-1", "主根-1", "主根-2"].contains(&元素.as_str()) {
                continue;
            }
            if 元素.starts_with("声") {
                决策空间.声母.push(序号);
                let Mapped::Basic(编码) = 编码 else {
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
                if let Mapped::Grouped { element } = 编码.clone() {
                    初始决策.元素[序号] = 元素安排::归并(棱镜.元素转数字[&element]);
                    决策空间.元素[序号] = vec![初始决策.元素[序号].clone().into()];
                } else {
                    let Mapped::Basic(编码) = 编码 else {
                        unreachable!();
                    };
                    let 键位 = 编码.chars().next().unwrap();
                    初始决策.元素[序号] = 元素安排::键位(键位);
                    let 鼓励归并 = [
                        ("韵-ai", 'i'),
                        ("韵-ei", 'a'),
                        ("韵-ou", 'o'),
                        ("韵-ü", 'u'),
                    ];
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
                                    if 鼓励归并
                                        .iter()
                                        .any(|&(韵, 鼓励键)| 元素 == 韵 && 鼓励键 == 键位)
                                    {
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
                let 当前决策 = 原始决策.get(元素).unwrap_or(&Mapped::Unused(()));
                if !原始安排列表.iter().any(|x| &x.value == 当前决策) {
                    原始安排列表.push(ValueDescription {
                        value: 当前决策.clone(),
                        score: 0.0,
                        condition: None,
                    });
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
                        let 默认条件 = Condition {
                            element: 棱镜.数字转元素[&归并字根].clone(),
                            op: "不是".to_string(),
                            value: Mapped::Unused(()),
                        };
                        if !原始条件.iter().any(|x| x == &默认条件) {
                            原始条件.push(默认条件);
                        }
                    }
                    let 条件列表: Vec<条件> = 原始条件
                        .into_iter()
                        .map(|c| 条件 {
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
                if 原始乱序生成器.elements.contains(&元素) {
                    let 条件列表 = 安排列表
                        .iter()
                        .find(|x| matches!(x.安排, 元素安排::声母韵母 { .. }))
                        .map(|x| &x.条件列表)
                        .unwrap_or(&vec![])
                        .clone();
                    for 键位 in 大集合 {
                        let 安排 = 元素安排::键位第二(键位);
                        if 安排列表.iter().any(|x| x.安排 == 安排) {
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
                简体频序: 0,
                繁体频率,
                繁体频序: 0,
                通打频率: 0.0,
                字块,
                通规: 词.通规,
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
                .then_with(|| b.gb2312.cmp(&a.gb2312))
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

    pub fn 生成码表(&self, 编码结果: &[冰雪清韵编码信息]) -> Vec<码表项> {
        let mut 码表 = Vec::new();
        let 转编码 = |code: 编码| self.棱镜.数字转编码(code as u64).iter().collect();
        for (序号, 可编码对象) in self.固定拆分.iter().enumerate() {
            let 码表项 = 码表项 {
                name: 可编码对象.词.to_string(),
                full: 转编码(编码结果[序号].全码),
                full_rank: 0,
                short: 转编码(编码结果[序号].简体简码),
                short_rank: 0,
            };
            码表.push(码表项);
        }
        码表
    }

    pub fn 翻转码表(
        &self,
        编码结果: &[冰雪清韵编码信息],
        顺序: &Vec<usize>,
        标签: &impl Fn(&冰雪清韵编码信息) -> bool,
        频率: &impl Fn(&冰雪清韵编码信息) -> 频率,
    ) -> Vec<(编码, Vec<char>, 频率)> {
        let mut 翻转码表 = FxHashMap::default();
        let mut 重码组列表 = vec![];
        for 索引 in 顺序 {
            翻转码表
                .entry(编码结果[*索引].全码)
                .or_insert_with(|| vec![])
                .push(self.固定拆分[*索引].词);
        }
        for 索引 in 顺序 {
            if 标签(&编码结果[*索引]) {
                let 重码组 = &翻转码表[&编码结果[*索引].全码];
                重码组列表.push((编码结果[*索引].全码, 重码组.clone(), 频率(&编码结果[*索引])));
            }
        }
        重码组列表
    }

    // 分析前 3000 字中全码重码和简码差指法的情况
    pub fn 分析码表(
        &self,
        编码结果: &[冰雪清韵编码信息],
        路径: &PathBuf,
    ) -> Result<(), 错误> {
        let mut 文件 = File::create(路径).unwrap();
        let 简体前三千: Vec<_> = self.简体顺序.iter().take(3000).cloned().collect();
        let 繁体前三千: Vec<_> = self.繁体顺序.iter().take(3000).cloned().collect();
        let 通打前三千: Vec<_> = (0..3000).collect();
        let 简体重码组列表 =
            self.翻转码表(编码结果, &简体前三千, &|x| x.简体选重 > 0, &|x| x.简体频率);
        let 繁体重码组列表 =
            self.翻转码表(编码结果, &繁体前三千, &|x| x.繁体选重 > 0, &|x| x.繁体频率);
        let 通打重码组列表 =
            self.翻转码表(编码结果, &通打前三千, &|x| x.通打选重 > 0, &|x| x.通打频率);
        for (label, 重码组列表) in [
            ("简体", 简体重码组列表),
            ("繁体", 繁体重码组列表),
            ("通打", 通打重码组列表),
        ] {
            writeln!(文件, "# 前 3000 中{label}全码重码\n")?;
            for (全码, 重码组, 次选频率) in 重码组列表 {
                let 全码: String = self.棱镜.数字转编码(全码 as u64).iter().collect();
                let 百万分之频率 = 次选频率 * 1_000_000.0;
                writeln!(文件, "- {全码} {重码组:?} [{百万分之频率:.2} μ]")?;
            }
            writeln!(文件, "")?;
        }
        let 指法标记 = 指法标记::new();
        let mut 差指法 = vec![];
        let mut 四键字 = vec![];
        for 序号 in 简体前三千.into_iter() {
            let 编码信息 = &编码结果[序号];
            let 词 = self.固定拆分[序号].词;
            let 简码 = self.棱镜.数字转编码(编码信息.简体简码 as u64);
            if 简码.len() == 4 && 序号 < 500 {
                四键字.push((词, 简码.iter().collect::<String>()));
            }
            if 序号 < 1500 {
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
            writeln!(文件, "- {字} {编码}")?;
        }
        writeln!(文件, "\n# 前 500 中四键字\n").unwrap();
        for (字, 编码) in 四键字 {
            writeln!(文件, "- {字} {编码}")?;
        }
        Ok(())
    }
}
