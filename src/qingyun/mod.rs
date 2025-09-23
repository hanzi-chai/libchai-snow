use chai::{
    config::{Mapped, MappedKey},
    optimizers::解特征,
    元素, 棱镜,
};
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
pub const 进制: 编码 = 32;
pub const 一码掩码: 编码 = 31;
pub const 二码掩码: 编码 = 1023;
pub const 三码掩码: 编码 = 32767;
pub const 空格: 编码 = 22;
pub const 特简字: [char; 8] = [' ', '的', '是', '我', '不', '了', '在', '和'];
pub const 特简码: [char; 8] = [' ', 'e', 'i', 'o', 'u', 'a', ';', '/'];
pub const 优先简码: [char; 10] = ['一', '二', '三', '四', '五', '六', '七', '八', '九', '十'];
pub const 笔画: [&str; 5] = ["1", "2", "3", "4", "5"];
pub const 不好的大集合键: [char; 5] = ['q', 'z', 'p', 'y', 'b'];

pub type 编码 = u32;
pub type 频率 = f32;
pub type 频序 = u32;
pub const 所有汉字数: usize = 20992;
pub const 常用简繁范围: usize = 8536;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 冰雪清韵编码信息 {
    pub 简体频率: 频率,
    pub 简体频序: 频序,
    pub 繁体频率: 频率,
    pub 繁体频序: 频序,
    pub 通打频率: 频率,
    pub 全码: 编码,
    pub 简体简码: 编码,
    pub 简体选重: u8,
    pub 繁体选重: u8,
    pub 通打选重: u8,
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
    pub 简体频序: 频序,
    pub 繁体频率: 频率,
    pub 繁体频序: 频序,
    pub 通打频率: 频率,
    pub 字块: [块; 4],
    pub 通规: u8,
    pub gb2312: bool,
    pub 国字常用: bool,
    pub 陆标: bool,
}

#[derive(Deserialize)]
struct 原始固定拆分项 {
    pub 汉字: char,
    pub 通规: u8,
    pub gb2312: bool,
    pub 国字常用: bool,
    pub 频率: u64,
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
    pub 元素: Vec<元素安排>,
    pub 补码键: char,
}

impl 冰雪清韵决策 {
    pub fn 线性化(&self, 棱镜: &棱镜) -> Vec<编码> {
        let mut 映射 = vec![0_u32; self.元素.len()];
        for (元素, 安排) in self.元素.iter().enumerate() {
            match 安排 {
                元素安排::未选取 => {}
                元素安排::键位(键位) => {
                    映射[元素] = 棱镜.键转数字[键位] as u32;
                }
                元素安排::归并(元素1) => {
                    映射[元素] = 映射[*元素1];
                }
                元素安排::键位韵母 { 键位, 韵母 } => {
                    映射[元素] = 棱镜.键转数字[键位] as u32 + 映射[*韵母] * 进制;
                }
                元素安排::归并韵母 { 字根, 韵母 } => {
                    映射[元素] = 映射[*字根] % 进制 + 映射[*韵母] * 进制;
                }
                元素安排::声母韵母 { 声母, 韵母 } => {
                    映射[元素] = 映射[*声母] + 映射[*韵母] * 进制;
                }
            }
        }
        映射
    }

    fn 允许(&self, 条件安排: &条件元素安排) -> bool {
        for 条件 in &条件安排.条件列表 {
            if 条件.谓词 != (self.元素[条件.元素] == 条件.值) {
                return false;
            }
        }
        return true;
    }

    fn 打印(&self, 棱镜: &棱镜) {
        for (元素, 安排) in self.元素.iter().enumerate() {
            if 元素 > 0 {
                println!("元素 {}: {:?}", 棱镜.数字转元素[&元素], 安排);
            }
        }
    }
}
#[derive(Debug, Clone)]
pub struct 冰雪清韵决策空间 {
    pub 元素: Vec<Vec<条件元素安排>>,
    pub 声母: Vec<元素>,
    pub 韵母: Vec<元素>,
    pub 字根: Vec<元素>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "类型", rename_all = "snake_case")]
pub enum 元素安排 {
    未选取,
    键位(char),
    归并(元素),
    键位韵母 { 键位: char, 韵母: 元素 },
    归并韵母 { 字根: 元素, 韵母: 元素 },
    声母韵母 { 声母: 元素, 韵母: 元素 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 条件元素安排 {
    pub 安排: 元素安排,
    pub 条件列表: Vec<条件>,
}

impl From<元素安排> for 条件元素安排 {
    fn from(安排: 元素安排) -> Self {
        条件元素安排 {
            安排,
            条件列表: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 条件 {
    pub 元素: 元素,
    pub 谓词: bool,
    pub 值: 元素安排,
}

impl 元素安排 {
    fn from(mapped: &Mapped, 棱镜: &棱镜) -> Self {
        match mapped {
            Mapped::Unused(()) => 元素安排::未选取,
            Mapped::Grouped { element } => 元素安排::归并(棱镜.元素转数字[element]),
            Mapped::Advanced(keys) => {
                let first = keys[0].clone();
                let MappedKey::Reference {
                    element: 韵母, ..
                } = keys[1].clone()
                else {
                    unreachable!();
                };
                match first {
                    MappedKey::Ascii(key) => 元素安排::键位韵母 {
                        键位: key,
                        韵母: 棱镜.元素转数字[&韵母],
                    },
                    MappedKey::Reference { element, .. } => {
                        if element.starts_with("声") {
                            元素安排::声母韵母 {
                                声母: 棱镜.元素转数字[&element],
                                韵母: 棱镜.元素转数字[&韵母],
                            }
                        } else {
                            元素安排::归并韵母 {
                                字根: 棱镜.元素转数字[&element],
                                韵母: 棱镜.元素转数字[&韵母],
                            }
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn to_mapped(&self, 棱镜: &棱镜) -> Mapped {
        match self {
            元素安排::未选取 => Mapped::Unused(()),
            元素安排::键位(键位) => Mapped::Basic(键位.to_string()),
            元素安排::归并(字根) => Mapped::Grouped {
                element: 棱镜.数字转元素[&字根].clone(),
            },
            元素安排::归并韵母 { 字根, 韵母 } => Mapped::Advanced(vec![
                MappedKey::Reference {
                    element: 棱镜.数字转元素[&字根].clone(),
                    index: 0,
                },
                MappedKey::Reference {
                    element: 棱镜.数字转元素[&韵母].clone(),
                    index: 0,
                },
            ]),
            元素安排::键位韵母 { 键位, 韵母 } => Mapped::Advanced(vec![
                MappedKey::Ascii(*键位),
                MappedKey::Reference {
                    element: 棱镜.数字转元素[&韵母].clone(),
                    index: 0,
                },
            ]),
            元素安排::声母韵母 { 声母, 韵母 } => Mapped::Advanced(vec![
                MappedKey::Reference {
                    element: 棱镜.数字转元素[&声母].clone(),
                    index: 0,
                },
                MappedKey::Reference {
                    element: 棱镜.数字转元素[&韵母].clone(),
                    index: 0,
                },
            ]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct 冰雪清韵决策变化 {
    pub 变化元素: Vec<元素>,
    pub 拆分改变: bool,
}

impl 冰雪清韵决策变化 {
    pub fn 无变化() -> Self {
        冰雪清韵决策变化 {
            变化元素: vec![],
            拆分改变: false,
        }
    }

    pub fn 新建(变化元素: Vec<元素>, 拆分改变: bool) -> Self {
        冰雪清韵决策变化 {
            变化元素, 拆分改变
        }
    }
}

impl 解特征 for 冰雪清韵决策 {
    type 变化 = 冰雪清韵决策变化;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct 规则输入 {
    pub 元素: String,
    pub 规则: Vec<元素安排>,
    pub 允许乱序: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 分析结果 {
    pub 重码项: Vec<(String, (Vec<String>, u64))>,
    pub 差指法: Vec<(String, String)>,
}
