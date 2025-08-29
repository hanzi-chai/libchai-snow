use crate::qingyun::{
    冰雪清韵决策, 冰雪清韵决策空间, 冰雪清韵编码信息, 动态拆分项, 固定拆分项, 大集合, 字根安排,
    小集合, 拆分输入, 条件, 条件字根安排, 笔画, 进制, 韵母安排,
};
use chai::{
    config::{Condition, Mapped, ValueDescription, 配置},
    contexts::上下文,
    interfaces::{command_line::读取文本文件, 默认输入},
    objectives::metric::指法标记,
    元素, 原始当量信息, 原始键位分布信息, 棱镜, 码表项, 编码, 错误,
};
use chrono::Local;
use core::panic;
use indexmap::IndexMap;
use itertools::Itertools;
use rustc_hash::FxHashMap;
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
    pub 繁体顺序: Vec<usize>,
    pub 简繁通打顺序: Vec<usize>,
    pub 下游字根: FxHashMap<String, Vec<String>>,
}

impl 上下文 for 冰雪清韵上下文 {
    type 解类型 = 冰雪清韵决策;

    fn 序列化(&self, 解: &冰雪清韵决策) -> String {
        let mut 新配置 = self.配置.clone();
        新配置.info.as_mut().unwrap().version =
            Some(format!("{}", Local::now().format("%Y-%m-%d+%H:%M:%S")));
        let mut mapping = IndexMap::new();
        for (元素, 安排) in 解.声母.iter() {
            mapping.insert(元素.clone(), Mapped::Basic(安排.to_string()));
        }
        for (元素, 安排) in 解.韵母.iter() {
            match 安排 {
                韵母安排::乱序 { 键位 } => {
                    mapping.insert(元素.clone(), Mapped::Basic(format!("{键位}")));
                }
                韵母安排::归并 { 韵母 } => {
                    mapping.insert(
                        元素.clone(),
                        Mapped::Grouped {
                            element: 韵母.clone(),
                        },
                    );
                }
            }
        }
        for (元素, 安排) in 解.字根.iter() {
            let mapped: Mapped = 安排.clone().into();
            if mapped != Mapped::Unused(()) {
                mapping.insert(元素.clone(), mapped);
            }
        }
        新配置.form.mapping = mapping;
        to_string(&新配置).unwrap()
    }
}

impl 冰雪清韵上下文 {
    fn _采用冰雪四拼声母布局(决策: &mut 冰雪清韵决策) {
        let 声母列表 = ["声-zh", "声-ch", "声-sh", "声-0", "声-w", "声-y", "声-yu"];
        let 键位列表 = ['w', 'y', 'v', 'r', 's', 'f', 'k'];
        for (声母, 键位) in 声母列表.into_iter().zip(键位列表.into_iter()) {
            if 决策.声母.contains_key(声母) {
                决策.声母.insert(声母.to_string(), 键位);
            } else {
                panic!("决策中没有找到声母: {}", 声母);
            }
        }
    }

    fn _采用有序笔画(决策: &mut 冰雪清韵决策) {
        let 笔画读音 = [
            ("声-h", "韵-eng"),
            ("声-sh", "韵-u"),
            ("声-p", "韵-ie"),
            ("声-d", "韵-ian"),
            ("声-zh", "韵-e"),
        ];
        for (笔, (声母, 韵母)) in 笔画.into_iter().zip(笔画读音) {
            决策.字根[笔] = 字根安排::读音 {
                声母: 声母.to_string(),
                韵母: 韵母.to_string(),
            };
        }
    }

    pub fn 新建(输入: 默认输入) -> Result<Self, 错误> {
        let mut 决策空间 = 冰雪清韵决策空间 {
            声母: IndexMap::default(),
            韵母: IndexMap::default(),
            字根: IndexMap::default(),
        };
        let mut 初始决策 = 冰雪清韵决策 {
            声母: IndexMap::default(),
            韵母: IndexMap::default(),
            字根: IndexMap::default(),
            补码键: 'k',
        };
        let 布局 = 输入.配置.form.clone();
        let 原始决策 = 布局.mapping;
        let 原始决策空间 = 布局.mapping_space.unwrap();
        let 原始乱序生成器 = 布局.mapping_generator.unwrap();
        let 原始乱序生成器 = 原始乱序生成器[原始乱序生成器.len() - 1].clone();
        let mut 下游字根: FxHashMap<String, Vec<_>> = FxHashMap::default();
        let mut 元素转数字 = FxHashMap::default();
        let mut 数字转元素 = FxHashMap::default();
        let mut 键转数字 = FxHashMap::default();
        let mut 数字转键 = FxHashMap::default();
        let mut 数字 = 0;
        for c in 大集合.into_iter().chain(小集合.into_iter()) {
            数字 += 1;
            元素转数字.insert(c.to_string(), 数字);
            数字转元素.insert(数字, c.to_string());
            键转数字.insert(c, 数字 as u64);
            数字转键.insert(数字 as u64, c);
        }
        let 所有元素: Vec<String> = from_str(&read_to_string("rules.yaml").unwrap()).unwrap();
        for 元素 in 所有元素 {
            let 编码 = 原始决策.get(&元素).unwrap_or(&Mapped::Unused(()));
            if 元素.starts_with("声") || 元素.starts_with("韵") {
                数字 += 1;
                元素转数字.insert(元素.clone(), 数字);
                数字转元素.insert(数字, 元素.clone());
                if 元素.starts_with("声") {
                    let Mapped::Basic(编码) = 编码 else {
                        unreachable!();
                    };
                    let 键位 = 编码.chars().next().unwrap();
                    初始决策.声母.insert(元素.clone(), 键位);
                    match 元素.as_str() {
                        "声-zh" | "声-ch" | "声-sh" | "声-0" => {
                            决策空间
                                .声母
                                .insert(元素.clone(), 大集合.into_iter().collect());
                        }
                        _ => {
                            决策空间.声母.insert(元素.clone(), vec![键位]);
                        }
                    }
                } else if 元素.starts_with("韵") {
                    if let Mapped::Grouped { element: 韵母 } = 编码.clone() {
                        初始决策.韵母.insert(元素.clone(), 韵母安排::归并 { 韵母 });
                        决策空间
                            .韵母
                            .insert(元素.clone(), vec![初始决策.韵母[&元素].clone()]);
                    } else {
                        let Mapped::Basic(编码) = 编码 else {
                            unreachable!();
                        };
                        let 键位 = 编码.chars().next().unwrap();
                        初始决策.韵母.insert(元素.clone(), 韵母安排::乱序 { 键位 });
                        match 元素.as_str() {
                            "韵-a" | "韵-e" | "韵-i" | "韵-o" | "韵-u" => {
                                决策空间
                                    .韵母
                                    .insert(元素.clone(), vec![初始决策.韵母[&元素].clone()]);
                            }
                            _ => {
                                let 可行键位 = 小集合
                                    .into_iter()
                                    .filter(|&c| c != '_')
                                    .map(|键位| 韵母安排::乱序 { 键位 })
                                    .collect();
                                决策空间.韵母.insert(元素.clone(), 可行键位);
                            }
                        }
                    }
                }
            } else {
                数字 += 1;
                元素转数字.insert(元素.clone(), 数字);
                数字转元素.insert(数字, 元素.clone());
                数字 += 1;
                let 小码 = format!("{元素}.1");
                元素转数字.insert(小码.clone(), 数字);
                数字转元素.insert(数字, 小码.clone());
                let mut 原始安排列表 = 原始决策空间.get(&元素).cloned().unwrap_or(vec![]);
                let 当前决策 = 原始决策.get(&元素).unwrap_or(&Mapped::Unused(()));
                if !原始安排列表.iter().any(|x| &x.value == 当前决策) {
                    原始安排列表.push(ValueDescription {
                        value: 当前决策.clone(),
                        score: 0.0,
                        condition: None,
                    });
                }
                let mut 安排列表 = vec![];
                for 原始安排 in &原始安排列表 {
                    let 字根安排 = 字根安排::from(&原始安排.value);
                    let mut 原始条件 = 原始安排.condition.clone().unwrap_or_default();
                    let 归并字根 = if let 字根安排::归并 { 字根 } = &字根安排 {
                        Some(字根.clone())
                    } else if let 字根安排::半归并 { 字根, .. } = &字根安排 {
                        Some(字根.clone())
                    } else {
                        None
                    };
                    if let Some(归并字根) = 归并字根 {
                        let 默认条件 = Condition {
                            element: 归并字根.clone(),
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
                            元素: c.element.clone(),
                            谓词: c.op == "是",
                            值: 字根安排::from(&c.value),
                        })
                        .collect();
                    for 条件 in &条件列表 {
                        if 下游字根.contains_key(&条件.元素) {
                            if !下游字根[&条件.元素].contains(&元素) {
                                下游字根.get_mut(&条件.元素).unwrap().push(元素.clone());
                            }
                        } else {
                            下游字根.insert(条件.元素.clone(), vec![元素.clone()]);
                        }
                    }
                    let 条件字根安排 = 条件字根安排 {
                        安排: 字根安排,
                        条件列表,
                    };
                    安排列表.push(条件字根安排);
                }
                if 原始乱序生成器.elements.contains(&元素) {
                    let 韵母列表: Vec<_> = 安排列表
                        .iter()
                        .filter_map(|x| {
                            if let 字根安排::读音 { 韵母, .. } = &x.安排 {
                                Some((韵母.to_string(), x.条件列表.clone()))
                            } else {
                                None
                            }
                        })
                        .collect();
                    for 键位 in 大集合 {
                        for (韵母, 条件) in &韵母列表 {
                            let 安排 = 字根安排::乱序 {
                                键位,
                                韵母: 韵母.to_string(),
                            };
                            if 安排列表.iter().any(|x| x.安排 == 安排) {
                                continue;
                            }
                            安排列表.push(条件字根安排 {
                                安排,
                                条件列表: 条件.clone(),
                            });
                        }
                    }
                }
                let 安排列表: Vec<_> = 安排列表.into_iter().collect();
                初始决策.字根.insert(元素.clone(), 当前决策.into());
                决策空间.字根.insert(元素.clone(), 安排列表);
            }
        }

        // Self::采用冰雪四拼声母布局(&mut 初始决策);
        Self::_采用有序笔画(&mut 初始决策);

        let mut 所有乱序键位: Vec<_> = 初始决策
            .字根
            .iter()
            .filter_map(|(_, 安排)| {
                if let 字根安排::乱序 { 键位, .. } = 安排 {
                    Some(*键位)
                } else {
                    None
                }
            })
            .collect();
        所有乱序键位.sort();
        assert!(所有乱序键位.len() == 21 && 大集合.iter().all(|c| 所有乱序键位.contains(&c)),);

        let 棱镜 = 棱镜 {
            键转数字,
            数字转键,
            元素转数字,
            数字转元素,
            进制: 进制,
        };

        let (固定拆分, 动态拆分, 块转数字, 数字转块, 繁体顺序, 简繁通打顺序) =
            Self::解析动态拆分(&棱镜, &决策空间);
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
            繁体顺序,
            简繁通打顺序,
            下游字根,
        })
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
            from_str(&read_to_string("dynamic_analysis.yaml").unwrap()).unwrap();
        let 繁体字频: FxHashMap<char, u64> = 读取文本文件(PathBuf::from("debug/ftzp.txt"));
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
            if !最后一项
                .iter()
                .all(|x| !决策空间.字根[x].iter().any(|x| x.安排 == 字根安排::未选取))
            {
                panic!("动态拆分方式列表的最后一项必须都是必选字根, {块:?}, {原始拆分方式列表:?}");
            }
            动态拆分.push(拆分方式列表);
        }
        let mut 固定拆分 = vec![];
        let 简体总频数: u64 = 拆分输入.固定拆分.iter().map(|x| x.频率).sum();
        let 繁体总频数: u64 = 繁体字频.values().cloned().sum();
        for 词 in &拆分输入.固定拆分 {
            let 字块 = Self::对齐(词.拆分.iter().map(|块| 块转数字[块]).collect(), usize::MAX);
            let 简体频率 = 词.频率 as f64 / 简体总频数 as f64;
            let 最高频读音 = 词.读音.iter().max_by_key(|&x| x.频率).unwrap();
            if !棱镜.元素转数字.contains_key(&最高频读音.声)
                || !棱镜.元素转数字.contains_key(&最高频读音.韵)
            {
                panic!(
                    "固定拆分项 {} 的声韵 {}-{} 不在棱镜中",
                    词.汉字, 最高频读音.声, 最高频读音.韵
                );
            }
            let 字 = 词.汉字.chars().next().unwrap();
            let 繁体频率 =
                繁体字频.get(&字).cloned().unwrap_or_default() as f64 / 繁体总频数 as f64;
            let 混合频率 = (简体频率 + 繁体频率) / 2.0;
            固定拆分.push(固定拆分项 {
                词: 字,
                简体频率,
                繁体频率,
                混合频率,
                声韵: (
                    棱镜.元素转数字[&最高频读音.声] as u8,
                    棱镜.元素转数字[&最高频读音.韵] as u8,
                ),
                字块,
                通规: 词.通规,
                gb2312: 词.gb2312,
                国字常用: 词.国字常用,
            });
        }
        固定拆分.sort_by(|a, b| {
            b.gb2312
                .cmp(&a.gb2312)
                .then_with(|| b.简体频率.partial_cmp(&a.简体频率).unwrap())
                .then_with(|| b.繁体频率.partial_cmp(&a.繁体频率).unwrap())
        });
        let 繁体顺序: Vec<_> = 固定拆分
            .iter()
            .enumerate()
            .filter(|(_, x)| x.国字常用)
            .sorted_by(|(_, a), (_, b)| b.繁体频率.partial_cmp(&a.繁体频率).unwrap())
            .map(|(i, _)| i)
            .collect();
        let 简繁通打顺序: Vec<_> = 固定拆分
            .iter()
            .enumerate()
            .sorted_by(|(_, a), (_, b)| b.混合频率.partial_cmp(&a.混合频率).unwrap())
            .map(|(i, _)| i)
            .collect();
        (
            固定拆分,
            动态拆分,
            块转数字,
            数字转块,
            繁体顺序,
            简繁通打顺序,
        )
    }

    pub fn 生成码表(&self, 编码结果: &[冰雪清韵编码信息]) -> Vec<码表项> {
        let mut 码表 = Vec::new();
        let 转编码 = |code: 编码| self.棱镜.数字转编码(code).iter().collect();
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
        频率: &impl Fn(&冰雪清韵编码信息) -> f64,
    ) -> Vec<(编码, Vec<char>, f64)> {
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
        let 指法标记 = 指法标记::new();
        let mut 文件 = File::create(路径).unwrap();
        let mut 差指法 = vec![];
        let mut 四键字 = vec![];
        let 简体前三千: Vec<_> = (0..3000).collect();
        let 繁体前三千: Vec<_> = self.繁体顺序.iter().take(3000).cloned().collect();
        let 简繁通打前三千: Vec<_> = self.简繁通打顺序.iter().take(3000).cloned().collect();
        for (序号, 编码信息) in 编码结果[..3000].iter().enumerate() {
            let 词 = self.固定拆分[序号].词;
            let 简码 = self.棱镜.数字转编码(编码信息.简体简码);
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
        let 简体重码组列表 =
            self.翻转码表(编码结果, &简体前三千, &|x| x.简体选重, &|x| {
                x.简体频率
            });
        let 繁体重码组列表 =
            self.翻转码表(编码结果, &繁体前三千, &|x| x.繁体选重, &|x| {
                x.繁体频率
            });
        let 简繁通打重码组列表 = self.翻转码表(
            编码结果,
            &简繁通打前三千,
            &|x| x.简繁通打选重,
            &|x| x.混合频率,
        );
        for (label, 重码组列表) in [
            ("简体", 简体重码组列表),
            ("繁体", 繁体重码组列表),
            ("简繁通打", 简繁通打重码组列表),
        ] {
            writeln!(文件, "# 前 3000 中{label}全码重码\n")?;
            for (全码, 重码组, 次选频率) in 重码组列表 {
                let 全码: String = self.棱镜.数字转编码(全码).iter().collect();
                let 百万分之频率 = 次选频率 * 1_000_000.0;
                writeln!(文件, "- {全码} {重码组:?} [{百万分之频率:.2} ppm]")?;
            }
            writeln!(文件, "")?;
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
