use chai::{
    config::{Mapped, MappedKey},
    optimizers::解特征,
    元素, 元素映射, 棱镜, 编码,
};
use indexmap::IndexMap;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
pub mod context;
pub mod encoder;
pub mod objective;
pub mod operators;

pub const 大集合: [char; 21] = [
    'b', 'p', 'm', 'f', 'd', 't', 'n', 'l', 'g', 'k', 'h', 'j', 'q', 'x', 'z', 'c', 's', 'r', 'w',
    'y', 'v',
];
pub const 小集合: [char; 10] = ['_', 'e', 'i', 'o', 'u', 'a', ';', '/', ',', '.'];
pub const 最大码长: u64 = 4;
pub const 进制: u64 = 32;
pub const 空格: u64 = 22;
pub const 特简字: [char; 8] = [' ', '的', '是', '我', '不', '了', '在', '和'];
pub const 特简码: [char; 8] = [' ', 'e', 'i', 'o', 'u', 'a', ';', '/'];
pub const 优先简码: [char; 10] = ['一', '二', '三', '四', '五', '六', '七', '八', '九', '十'];
pub const 笔画: [&str; 5] = ["1", "2", "3", "4", "5"];

type 频率 = f64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 冰雪清韵编码信息 {
    pub 简体频率: 频率,
    pub 繁体频率: 频率,
    pub 混合频率: 频率,
    pub 全码: 编码,
    pub 简体简码: 编码,
    pub 简体选重: bool,
    pub 繁体选重: bool,
    pub 简繁通打选重: bool,
    pub 特简: u8,
    pub 完成出简: bool,
    pub 简体: bool,
    pub 繁体: bool,
}

pub type 块 = usize;
pub type 动态拆分项 = Vec<[元素; 4]>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 固定拆分项 {
    pub 词: char,
    pub 简体频率: 频率,
    pub 繁体频率: 频率,
    pub 混合频率: 频率,
    pub 声韵: (u8, u8),
    pub 字块: [块; 4],
    pub 通规: u8,
    pub gb2312: bool,
    pub 国字常用: bool,
}

#[derive(Deserialize)]
struct 带频读音 {
    pub 频率: u32,
    pub 声: String,
    pub 韵: String,
}

#[derive(Deserialize)]
struct 原始固定拆分项 {
    pub 汉字: String,
    pub 通规: u8,
    pub gb2312: bool,
    pub 国字常用: bool,
    pub 频率: u64,
    pub 读音: Vec<带频读音>,
    pub 拆分: Vec<String>,
}

type 原始固定拆分 = Vec<原始固定拆分项>;
type 原始动态拆分 = FxHashMap<String, Vec<Vec<String>>>;

#[derive(Deserialize)]
struct 拆分输入 {
    固定拆分: 原始固定拆分,
    动态拆分: 原始动态拆分,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 冰雪清韵决策 {
    pub 声母: IndexMap<String, 声母安排>,
    pub 韵母: IndexMap<String, 韵母安排>,
    pub 字根: IndexMap<String, 字根安排>,
    pub 补码键: char,
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
                字根安排::乱序 { 键位, 韵母 } => {
                    let 韵母索引 = 棱镜.元素转数字[韵母];
                    映射[索引] = 棱镜.键转数字[键位];
                    映射[索引 + 1] = 映射[韵母索引];
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
                    assert!(映射[索引] != 0 && 映射[索引 + 1] != 0, "{元素:?} 归并 {字根:?} 映射错误，当前决策: {self:?}");
                }
                字根安排::半归并 { 字根, 韵母 } => {
                    let 字根索引 = 棱镜.元素转数字[字根];
                    let 韵母索引 = 棱镜.元素转数字[韵母];
                    映射[索引] = 映射[字根索引];
                    映射[索引 + 1] = 映射[韵母索引];
                    assert!(映射[索引] != 0, "{元素:?} 归并 {字根:?} 映射错误，当前决策: {self:?}");
                }
            }
        }
        映射
    }

    fn 允许(&self, 条件安排: &条件字根安排) -> bool {
        for 条件 in &条件安排.条件列表 {
            if 条件.谓词 != (self.字根[&条件.元素] == 条件.值) {
                return false;
            }
        }
        return true;
    }

}
#[derive(Debug, Clone)]
pub struct 冰雪清韵决策空间 {
    pub 声母: IndexMap<String, Vec<声母安排>>,
    pub 韵母: IndexMap<String, Vec<韵母安排>>,
    pub 字根: IndexMap<String, Vec<条件字根安排>>,
}

pub type 声母安排 = char;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum 韵母安排 {
    乱序 { 键位: char },
    归并 { 韵母: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "类型", rename_all = "snake_case")]
pub enum 字根安排 {
    未选取,
    归并 { 字根: String },
    半归并 { 字根: String, 韵母: String },
    乱序 { 键位: char, 韵母: String },
    读音 { 声母: String, 韵母: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 条件字根安排 {
    pub 安排: 字根安排,
    pub 条件列表: Vec<条件>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 条件 {
    pub 元素: String,
    pub 谓词: bool,
    pub 值: 字根安排,
}

impl From<&Mapped> for 字根安排 {
    fn from(mapped: &Mapped) -> Self {
        match mapped {
            Mapped::Unused(()) => 字根安排::未选取,
            Mapped::Grouped { element } => 字根安排::归并 {
                字根: element.to_string(),
            },
            Mapped::Advanced(keys) => {
                let first = keys[0].clone();
                let MappedKey::Reference {
                    element: 韵母, ..
                } = keys[1].clone()
                else {
                    unreachable!();
                };
                match first {
                    MappedKey::Ascii(key) => 字根安排::乱序 {
                        键位: key, 韵母
                    },
                    MappedKey::Reference { element, .. } => {
                        if element.starts_with("声") {
                            字根安排::读音 {
                                声母: element,
                                韵母,
                            }
                        } else {
                            字根安排::半归并 {
                                字根: element,
                                韵母,
                            }
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

impl From<字根安排> for Mapped {
    fn from(安排: 字根安排) -> Self {
        match 安排 {
            字根安排::未选取 => Mapped::Unused(()),
            字根安排::归并 { 字根 } => Mapped::Grouped { element: 字根 },
            字根安排::半归并 { 字根, 韵母 } => Mapped::Advanced(vec![
                MappedKey::Reference {
                    element: 字根,
                    index: 0,
                },
                MappedKey::Reference {
                    element: 韵母,
                    index: 0,
                },
            ]),
            字根安排::乱序 { 键位, 韵母 } => Mapped::Advanced(vec![
                MappedKey::Ascii(键位),
                MappedKey::Reference {
                    element: 韵母,
                    index: 0,
                },
            ]),
            字根安排::读音 { 声母, 韵母 } => Mapped::Advanced(vec![
                MappedKey::Reference {
                    element: 声母,
                    index: 0,
                },
                MappedKey::Reference {
                    element: 韵母,
                    index: 0,
                },
            ]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct 冰雪清韵决策变化 {
    pub 变化元素: Vec<String>,
    pub 拆分改变: bool,
}

impl 冰雪清韵决策变化 {
    pub fn 无变化() -> Self {
        冰雪清韵决策变化 {
            变化元素: vec![],
            拆分改变: false
        }
    }

    pub fn 新建(变化元素: Vec<String>, 拆分改变: bool) -> Self {
        冰雪清韵决策变化 {
            变化元素,
            拆分改变,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 分析结果 {
    pub 重码项: Vec<(String, (Vec<String>, u64))>,
    pub 差指法: Vec<(String, String)>,
}
