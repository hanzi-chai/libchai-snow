pub mod encoder;
pub mod objective;
pub mod operators;
use chai::{
    config::{Mapped, 配置},
    contexts::上下文,
    interfaces::默认输入,
    objectives::metric::指法标记,
    optimizers::解特征,
    元素映射, 原始可编码对象, 原始当量信息, 原始键位分布信息, 棱镜, 码表项, 编码, 编码信息, 错误,
};
use chrono::Local;
use core::panic;
use indexmap::IndexMap;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use serde_yaml::{from_str, to_string};
use std::{
    cmp::Reverse,
    fs::{File, read_to_string},
    io::Write,
    path::PathBuf,
};

pub const 大集合: &str = "bpmfdtnlgkhjqxzcsrwyv";
pub const 小集合: &str = "eaiou;,./_";
pub const 最大码长: u64 = 4;
pub const 进制: u64 = 32;
pub const 空格: u64 = 31;

#[derive(Clone)]
pub struct 冰雪清韵上下文 {
    pub 配置: 配置,
    pub 棱镜: 棱镜,
    pub 初始决策: 冰雪清韵决策,
    pub 决策空间: 冰雪清韵决策空间,
    pub 词列表: Vec<原始可编码对象>,
    pub 原始键位分布信息: 原始键位分布信息,
    pub 原始当量信息: 原始当量信息,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 冰雪清韵决策 {
    pub 声母: IndexMap<String, 声母安排>,
    pub 韵母: IndexMap<String, 韵母安排>,
    pub 字根: IndexMap<String, 字根安排>,
}

impl 冰雪清韵决策 {
    pub fn 线性化(&self, 棱镜: &棱镜) -> 元素映射 {
        let mut 映射 = vec![0; 棱镜.数字转元素.len() + 1];
        for (元素, 编码) in &self.声母 {
            let 索引 = 棱镜.元素转数字[元素];
            映射[索引] = 棱镜.键转数字[编码];
        }
        for (元素, 安排) in &self.韵母 {
            let 索引 = 棱镜.元素转数字[元素];
            match 安排 {
                韵母安排::乱序 { 键位 } => {
                    映射[索引] = 棱镜.键转数字[键位];
                }
                韵母安排::归并 { 韵母 } => {
                    映射[索引] = 映射[棱镜.元素转数字[韵母]];
                }
            }
        }
        for (元素, 安排) in &self.字根 {
            let 索引 = 棱镜.元素转数字[元素];
            if 元素 == "6" {
                let 字根索引 = 棱镜.元素转数字["5"];
                映射[索引] = 映射[字根索引];
                映射[索引 + 1] = 棱镜.键转数字[&'i'];
                continue;
            }
            match 安排 {
                字根安排::未选取 => {}
                字根安排::乱序 { 键位 } => {
                    映射[索引] = 棱镜.键转数字[键位];
                    映射[索引 + 1] = 空格;
                }
                字根安排::读音 { 声母, 韵母 } => {
                    let 声母索引 = 棱镜.元素转数字[声母];
                    let 韵母索引 = 棱镜.元素转数字[韵母];
                    映射[索引] = 映射[声母索引];
                    映射[索引 + 1] = 映射[韵母索引];
                }
                字根安排::归并 { 字根 } => {
                    let 字根索引 = 棱镜.元素转数字[字根];
                    映射[索引] = 映射[字根索引];
                    映射[索引 + 1] = 映射[字根索引 + 1];
                }
            }
        }
        映射
    }
}
#[derive(Debug, Clone)]
pub struct 冰雪清韵决策空间 {
    pub 声母: IndexMap<String, Vec<声母安排>>,
    pub 韵母: IndexMap<String, Vec<韵母安排>>,
    pub 字根: IndexMap<String, Vec<字根安排>>,
    pub 允许乱序: FxHashSet<String>,
}

pub type 声母安排 = char;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum 韵母安排 {
    乱序 { 键位: char },
    归并 { 韵母: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "类型", rename_all = "snake_case")]
pub enum 字根安排 {
    未选取,
    乱序 { 键位: char },
    读音 { 声母: String, 韵母: String },
    归并 { 字根: String },
}

#[derive(Debug, Clone)]
pub struct 冰雪清韵决策变化 {
    pub 拆分改变: bool,
}

impl 冰雪清韵决策变化 {
    pub fn 新建() -> Self {
        冰雪清韵决策变化 {
            拆分改变: false
        }
    }
}

impl 解特征 for 冰雪清韵决策 {
    type 变化 = 冰雪清韵决策变化;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct 规则输入 {
    pub 元素: String,
    pub 规则: Vec<字根安排>,
    pub 允许乱序: Option<bool>,
}

impl 上下文 for 冰雪清韵上下文 {
    type 解类型 = 冰雪清韵决策;

    fn 序列化(&self, 解: &冰雪清韵决策) -> String {
        let mut 新配置 = self.配置.clone();
        新配置.info.as_mut().unwrap().version =
            Some(format!("{}", Local::now().format("%Y-%m-%d+%H:%M:%S")));
        let 映射 = 解.线性化(&self.棱镜);
        let mut mapping = IndexMap::new();
        let 全部元素: Vec<_> = 解
            .声母
            .keys()
            .chain(解.韵母.keys())
            .chain(解.字根.keys())
            .cloned()
            .collect();
        for 元素 in &全部元素 {
            let 索引 = self.棱镜.元素转数字[元素];
            let 键 = 映射[索引];
            if 元素.starts_with("声") || 元素.starts_with("韵") {
                let 新键位 = Mapped::Basic(self.棱镜.数字转键[&键].to_string());
                mapping.insert(元素.clone(), 新键位);
            } else {
                if 键 == 0 {
                    continue;
                }
                let 小码 = 映射[索引 + 1];
                let mut 编码 = self.棱镜.数字转键[&键].to_string();
                if 小码 != 空格 {
                    编码.push_str(&self.棱镜.数字转键[&小码].to_string());
                }
                let 新键位 = Mapped::Basic(编码);
                mapping.insert(元素.clone(), 新键位);
            }
        }
        新配置.form.mapping = mapping;
        to_string(&新配置).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 分析结果 {
    pub 重码项: Vec<(String, (Vec<String>, u64))>,
    pub 差指法: Vec<(String, String)>,
}

impl 冰雪清韵上下文 {
    fn 按介音归并(韵母: &str) -> String {
        let 归并后: &str = match 韵母 {
            "ia" | "ua" => "a",
            "ie" | "ve" => "e",
            "io" | "uo" => "o",
            "er" => "i",
            "uai" => "ai",
            "uei" => "ei",
            "iao" => "ao",
            "iou" => "ou",
            "ian" | "uan" | "van" => "an",
            "in" | "uen" | "vn" | "n" => "en",
            "iang" | "uang" => "ang",
            "ing" | "ueng" | "ng" => "eng",
            "iong" => "ong",
            _ => panic!("未知韵母: {}", 韵母),
        };
        format!("韵-{}", 归并后)
    }

    pub fn 新建(输入: 默认输入) -> Result<Self, 错误> {
        let 规则列表: Vec<规则输入> = from_str(&read_to_string("rules.yaml")?).unwrap();
        let 乱序安排: Vec<_> = 大集合.chars().map(|c| 字根安排::乱序 { 键位: c }).collect();
        let mut 决策空间 = 冰雪清韵决策空间 {
            声母: IndexMap::default(),
            韵母: IndexMap::default(),
            字根: IndexMap::default(),
            允许乱序: FxHashSet::default(),
        };
        let mut 初始决策 = 冰雪清韵决策 {
            声母: IndexMap::default(),
            韵母: IndexMap::default(),
            字根: IndexMap::default(),
        };
        let 布局 = 输入.配置.form.clone();
        let 映射 = 布局.mapping;
        let 可选映射 = 布局.optional.unwrap().mapping;
        let mut 元素转数字 = FxHashMap::default();
        let mut 数字转元素 = FxHashMap::default();
        let mut 键转数字 = FxHashMap::default();
        let mut 数字转键 = FxHashMap::default();
        let mut 数字 = 0;
        for c in 大集合.chars().chain(小集合.chars()) {
            数字 += 1;
            元素转数字.insert(c.to_string(), 数字);
            数字转元素.insert(数字, c.to_string());
            键转数字.insert(c, 数字 as u64);
            数字转键.insert(数字 as u64, c);
        }
        let 投影 = |编码: &Mapped| {
            let Mapped::Basic(s) = 编码 else {
                panic!("编码必须是基本类型");
            };
            s.to_string()
        };
        for 规则输入 {
            元素: 名称,
            规则,
            允许乱序,
        } in &规则列表
        {
            let 允许乱序 = 允许乱序.unwrap_or(false);
            let 编码 = &映射.get(名称).unwrap_or_else(|| &可选映射[名称]);
            let 元素 = 名称.clone();
            let 编码 = 投影(编码);
            if 元素.starts_with("声") || 元素.starts_with("韵") {
                数字 += 1;
                元素转数字.insert(元素.clone(), 数字);
                数字转元素.insert(数字, 元素.clone());
                if 元素.starts_with("声") {
                    初始决策
                        .声母
                        .insert(元素.clone(), 编码.chars().next().unwrap());
                    match 元素.as_str() {
                        "声-zh" | "声-ch" | "声-sh" | "声-0" => {
                            决策空间.声母.insert(元素.clone(), 大集合.chars().collect());
                        }
                        _ => {
                            决策空间
                                .声母
                                .insert(元素.clone(), vec![初始决策.声母[&元素]]);
                        }
                    }
                } else if 元素.starts_with("韵") {
                    let 键位 = 编码.chars().next().unwrap();
                    match 元素.as_str() {
                        "韵-a" | "韵-e" | "韵-i" | "韵-o" | "韵-u" => {
                            初始决策.韵母.insert(元素.clone(), 韵母安排::乱序 { 键位 });
                            决策空间
                                .韵母
                                .insert(元素.clone(), vec![初始决策.韵母[&元素].clone()]);
                        }
                        "韵-v" | "韵-ai" | "韵-ao" | "韵-ei" | "韵-ou" | "韵-an" | "韵-en"
                        | "韵-ang" | "韵-eng" | "韵-ong" => {
                            初始决策.韵母.insert(元素.clone(), 韵母安排::乱序 { 键位 });
                            let 可行键位 = 小集合
                                .chars()
                                .filter(|&c| c != '_')
                                .map(|键位| 韵母安排::乱序 { 键位 })
                                .collect();
                            决策空间.韵母.insert(元素.clone(), 可行键位);
                        }
                        x => {
                            let 韵母 = 冰雪清韵上下文::按介音归并(&x[4..]);
                            初始决策.韵母.insert(元素.clone(), 韵母安排::归并 { 韵母 });
                            决策空间
                                .韵母
                                .insert(元素.clone(), vec![初始决策.韵母[&元素].clone()]);
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
                决策空间.字根.insert(元素.clone(), 规则.to_vec());
                if 可选映射.contains_key(&元素) {
                    决策空间.字根.get_mut(&元素).unwrap().push(字根安排::未选取);
                }
                let mut 匹配 = false;
                for 安排 in 决策空间.字根[&元素].iter().chain(乱序安排.iter()) {
                    匹配 = match 安排 {
                        字根安排::读音 { 声母, 韵母 } => {
                            编码 == 投影(&映射[声母]) + 投影(&映射[韵母]).as_str()
                        }
                        字根安排::归并 { 字根 } => {
                            映射.contains_key(字根) && 编码 == 投影(&映射[字根])
                        }
                        字根安排::乱序 { 键位 } => {
                            (允许乱序 || "123456".contains(&元素)) && 编码 == 键位.to_string()
                        }
                        字根安排::未选取 => 编码 == "a",
                    };
                    if 匹配 {
                        初始决策.字根.insert(元素.clone(), 安排.clone());
                        break;
                    }
                }
                if 允许乱序 {
                    决策空间.允许乱序.insert(元素.clone());
                }
                if !匹配 {
                    panic!("字根 {元素:?} 的编码 {编码:?} 在规则中没有匹配到");
                }
            }
        }

        let 棱镜 = 棱镜 {
            键转数字,
            数字转键,
            元素转数字,
            数字转元素,
            进制,
        };
        let mut 词列表 = 输入.词列表.clone();
        词列表.sort_by_key(|x| Reverse(x.frequency));
        Ok(Self {
            配置: 输入.配置,
            棱镜,
            初始决策,
            决策空间,
            词列表,
            原始键位分布信息: 输入.原始键位分布信息,
            原始当量信息: 输入.原始当量信息,
        })
    }

    pub fn 生成码表(&self, 编码结果: &[编码信息]) -> Vec<码表项> {
        let mut 码表 = Vec::new();
        let 转编码 = |code: 编码| self.棱镜.数字转编码(code).iter().collect();
        for (序号, 可编码对象) in self.词列表.iter().enumerate() {
            let 码表项 = 码表项 {
                name: 可编码对象.name.clone(),
                full: 转编码(编码结果[序号].全码.原始编码),
                full_rank: 编码结果[序号].全码.原始编码候选位置,
                short: 转编码(编码结果[序号].简码.原始编码),
                short_rank: 编码结果[序号].简码.原始编码候选位置,
            };
            码表.push(码表项);
        }
        码表
    }

    // 分析前 3000 字中全码重码和简码差指法的情况
    pub fn 分析码表(&self, 码表: &[码表项], 路径: &PathBuf) {
        let 指法标记 = 指法标记::new();
        let mut 文件 = File::create(路径).unwrap();
        let mut 翻转码表: FxHashMap<String, (Vec<String>, u64)> = FxHashMap::default();
        for 码表项 in &码表[..3000] {
            let 记录 = 翻转码表
                .entry(码表项.full.clone())
                .or_insert_with(|| (vec![], 0));
            记录.0.push(码表项.name.clone());
            if 记录.0.len() == 2 {
                记录.1 = self
                    .词列表
                    .iter()
                    .find(|x| x.name == 码表项.name)
                    .unwrap()
                    .frequency;
            }
            for 键索引 in 0..(码表项.short.len() - 1) {
                let 组合 = (
                    码表项.short.chars().nth(键索引).unwrap(),
                    码表项.short.chars().nth(键索引 + 1).unwrap(),
                );
                if 指法标记.同指大跨排.contains(&组合) || 指法标记.错手.contains(&组合)
                {
                    writeln!(文件, "{} {}", 码表项.name, 码表项.short).unwrap();
                }
            }
        }
        let mut 重码项: Vec<_> = 翻转码表
            .into_iter()
            .filter(|(_, (names, _))| names.len() > 1)
            .collect();
        重码项.sort_by_key(|(_, (_, frequency))| Reverse(*frequency));
        for (full, (names, frequency)) in 重码项 {
            writeln!(文件, "{full} {names:?} ({frequency})").unwrap();
        }
    }
}
