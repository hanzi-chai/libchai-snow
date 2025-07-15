pub mod encoder;
pub mod objective;
pub mod operators;
use chai::{
    config::{Mapped, 配置},
    contexts::上下文,
    interfaces::默认输入,
    optimizers::解特征,
    元素映射, 原始可编码对象, 棱镜, 码表项, 编码, 编码信息, 错误,
};
use core::panic;
use indexmap::IndexMap;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_yaml::{from_str, to_string};
use std::fs::read_to_string;

pub const 大集合: &str = "bpmfdtnlgkhjqxzcsrwyv";
pub const 小集合: &str = "aeiou;,./_";
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
pub struct 冰雪清韵决策变化 {}

impl 解特征 for 冰雪清韵决策 {
    type 变化 = 冰雪清韵决策变化;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct 规则输入 {
    pub 名称: String,
    pub 规则: Vec<字根安排>,
}

impl 上下文 for 冰雪清韵上下文 {
    type 解类型 = 冰雪清韵决策;

    fn 序列化(&self, 解: &冰雪清韵决策) -> String {
        // let 新配置 = self.配置.clone();
        // let 映射 = 解.线性化(&self.棱镜);
        to_string(&解).unwrap()
        // write(
        //     "debug.txt",
        //     映射
        //         .iter()
        //         .enumerate()
        //         .map(|(i, x)| {
        //             format!(
        //                 "{}: {x}",
        //                 棱镜.数字转元素.get(&i).unwrap_or(&"".to_string())
        //             )
        //         })
        //         .collect::<Vec<_>>()
        //         .join("\n"),
        // )?;
    }
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
        let mut 决策空间 = 冰雪清韵决策空间 {
            声母: IndexMap::default(),
            韵母: IndexMap::default(),
            字根: IndexMap::default(),
        };
        let mut 决策 = 冰雪清韵决策 {
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
        for 规则输入 { 名称, 规则 } in &规则列表 {
            let 编码 = &映射.get(名称).unwrap_or_else(|| &可选映射[名称]);
            let 元素 = 名称.clone();
            let 编码 = 投影(编码);
            if 元素.starts_with("声") || 元素.starts_with("韵") {
                数字 += 1;
                元素转数字.insert(元素.clone(), 数字);
                数字转元素.insert(数字, 元素.clone());
                if 元素.starts_with("声") {
                    决策.声母.insert(元素.clone(), 编码.chars().next().unwrap());
                    match 元素.as_str() {
                        "声-zh" | "声-ch" | "声-sh" | "声-0" => {
                            决策空间.声母.insert(元素.clone(), 大集合.chars().collect());
                        }
                        _ => {
                            决策空间.声母.insert(元素.clone(), vec![决策.声母[&元素]]);
                        }
                    }
                } else if 元素.starts_with("韵") {
                    let 键位 = 编码.chars().next().unwrap();
                    match 元素.as_str() {
                        "韵-a" | "韵-e" | "韵-i" | "韵-o" | "韵-u" => {
                            决策.韵母.insert(元素.clone(), 韵母安排::乱序 { 键位 });
                            决策空间
                                .韵母
                                .insert(元素.clone(), vec![决策.韵母[&元素].clone()]);
                        }
                        "韵-v" | "韵-ai" | "韵-ao" | "韵-ei" | "韵-ou" | "韵-an" | "韵-en"
                        | "韵-ang" | "韵-eng" | "韵-ong" => {
                            决策.韵母.insert(元素.clone(), 韵母安排::乱序 { 键位 });
                            let 可行键位 = 小集合
                                .chars()
                                .filter(|&c| c != '_')
                                .map(|键位| 韵母安排::乱序 { 键位 })
                                .collect();
                            决策空间.韵母.insert(元素.clone(), 可行键位);
                        }
                        x => {
                            let 韵母 = 冰雪清韵上下文::按介音归并(&x[4..]);
                            决策.韵母.insert(元素.clone(), 韵母安排::归并 { 韵母 });
                            决策空间
                                .韵母
                                .insert(元素.clone(), vec![决策.韵母[&元素].clone()]);
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
                for 安排 in 决策空间.字根[&元素].iter() {
                    匹配 = match 安排 {
                        字根安排::未选取 => 编码 == "a",
                        字根安排::乱序 { .. } => false,
                        字根安排::读音 { 声母, 韵母 } => {
                            编码 == 投影(&映射[声母]) + 投影(&映射[韵母]).as_str()
                        }
                        字根安排::归并 { 字根 } => {
                            映射.contains_key(字根) && 编码 == 投影(&映射[字根])
                        }
                    };
                    if 匹配 {
                        决策.字根.insert(元素.clone(), 安排.clone());
                        break;
                    }
                }
                if !匹配 {
                    panic!("字根 {} 的编码 {} 在规则中没有匹配到", 元素, 编码);
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
        Ok(Self {
            配置: 输入.配置,
            棱镜,
            初始决策: 决策,
            决策空间,
            词列表: 输入.词列表,
        })
    }

    pub fn 生成码表(&self, 编码结果: &[编码信息]) -> Vec<码表项> {
        let mut 码表: Vec<(usize, 码表项)> = Vec::new();
        let 转编码 = |code: 编码| self.棱镜.数字转编码(code).iter().collect();
        for (序号, 可编码对象) in self.词列表.iter().enumerate() {
            let 码表项 = 码表项 {
                name: 可编码对象.name.clone(),
                full: 转编码(编码结果[序号].全码.原始编码),
                full_rank: 编码结果[序号].全码.原始编码候选位置,
                short: 转编码(编码结果[序号].简码.原始编码),
                short_rank: 编码结果[序号].简码.原始编码候选位置,
            };
            码表.push((0, 码表项));
        }
        码表.sort_by_key(|x| x.0);
        码表.into_iter().map(|x| x.1).collect()
    }
}
