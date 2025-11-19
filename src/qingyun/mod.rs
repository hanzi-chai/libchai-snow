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
    'b', 'p', 'm', 'f', 'd', 't', 'n', 'l', 'g', 'k', 'h', 'j', 'q', 'x', 'z', 'c', 's', 'r', 'v',
    'w', 'y',
];
pub const 小集合: [char; 10] = ['_', 'e', 'i', 'o', 'u', 'a', ';', '/', ',', '.'];
pub const 进制: 键 = 32;
pub const 空格: 键 = 22;
pub const 特简码: [(char, char); 7] = [
    ('的', 'e'),
    ('是', 'i'),
    ('我', 'o'),
    ('不', 'u'),
    ('了', 'a'),
    ('在', ';'),
    ('和', '/'),
];
pub const 数字字根: [char; 10] = ['一', '二', '三', '四', '五', '六', '七', '八', '九', '十'];
pub const 笔画: [&str; 6] = ["1", "2", "3", "4", "5", "6"];
pub const 左手大码: [char; 13] = [
    'q', 'w', 'r', 't', 's', 'd', 'f', 'g', 'z', 'x', 'c', 'v', 'b',
];
pub const 右手大码: [char; 8] = ['y', 'p', 'h', 'j', 'k', 'l', 'n', 'm'];
pub const 不好的大集合键: [char; 5] = ['q', 'z', 'p', 'y', 'b'];
pub const 主根小码: [char; 5] = ['a', 'o', 'e', 'i', 'u'];

pub type 键 = u8;
pub type 双键 = (键, 键);
pub type 编码 = [u8; 4];
pub type 频率 = f32;
pub type 频序 = u32;
pub const 所有汉字数: usize = 20992;
pub const 常用简繁范围: usize = 8536;
pub const 无空格: bool = false;

trait 转换 {
    fn to_usize(&self) -> usize;

    fn 编码空间大小() -> usize;
}

impl 转换 for 编码 {
    fn to_usize(&self) -> usize {
        let k = 进制 as usize;
        let a = 空格 as usize;
        let result = self[0] as usize * a * a * k
            + self[1] as usize * a * k
            + self[2] as usize * k
            + self[3] as usize;
        result
    }

    fn 编码空间大小() -> usize {
        (进制 as usize).pow(3) * (空格 as usize)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 冰雪清韵编码信息 {
    // 常量
    pub 简体频率: 频率,
    pub 简体指数频率: 频率,
    pub 简体频序: 频序,
    pub 繁体频率: 频率,
    pub 繁体指数频率: 频率,
    pub 繁体频序: 频序,
    pub 通打频率: 频率,
    pub 简体: bool,
    pub 繁体: bool,
    pub 特简: bool,
    // 变量
    pub 全码: 编码,
    pub 计重全码: 编码,
    pub 计重索引: usize,
    pub 简体简码: 编码,
    pub 字根字: bool,
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
    pub 通规: bool,
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
    pub 第一主根: char,
    pub 第二主根: char,
}

pub type 映射 = Vec<双键>;

impl 冰雪清韵决策 {
    pub fn 线性化(&self, 棱镜: &棱镜) -> 映射 {
        let mut 映射 = vec![(0, 0); self.元素.len()];
        let 第一主根左手小码 = 棱镜.键转数字[&'e'] as 键;
        let 第一主根右手小码 = 棱镜.键转数字[&'i'] as 键;
        let 第二主根左手小码 = 棱镜.键转数字[&'a'] as 键;
        let 第二主根右手小码 = 棱镜.键转数字[&'o'] as 键;
        for (元素, 安排) in self.元素.iter().enumerate() {
            match 安排 {
                元素安排::未选取 => {}
                元素安排::键位(键位) => {
                    映射[元素] = (棱镜.键转数字[键位] as 键, 0);
                }
                元素安排::归并(元素1) => {
                    映射[元素] = 映射[*元素1];
                }
                元素安排::键位第一(键位) => {
                    let 小码 = if 左手大码.contains(键位) {
                        第一主根右手小码
                    } else {
                        第一主根左手小码
                    };
                    映射[元素] = (棱镜.键转数字[键位] as 键, 小码);
                }
                元素安排::键位第二(键位) => {
                    let 小码 = if 左手大码.contains(键位) {
                        第二主根右手小码
                    } else {
                        第二主根左手小码
                    };
                    映射[元素] = (棱镜.键转数字[键位] as 键, 小码);
                }
                元素安排::归并韵母 { 字根, 韵母 } => {
                    映射[元素] = (映射[*字根].0, 映射[*韵母].0);
                }
                元素安排::声母韵母 { 声母, 韵母 } => {
                    映射[元素] = (映射[*声母].0, 映射[*韵母].0);
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

    fn _打印(&self, 棱镜: &棱镜) {
        for (元素, 安排) in self.元素.iter().enumerate() {
            if 元素 > 0 {
                println!("元素 {:?}: {:?}", 棱镜.数字转元素[&元素], 安排);
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
    键位第一(char),
    键位第二(char),
    归并韵母 { 字根: 元素, 韵母: 元素 },
    声母韵母 { 声母: 元素, 韵母: 元素 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 条件元素安排 {
    pub 安排: 元素安排,
    pub 条件列表: Vec<条件>,
    pub 打分: f64,
}

impl From<元素安排> for 条件元素安排 {
    fn from(安排: 元素安排) -> Self {
        条件元素安排 {
            安排,
            条件列表: vec![],
            打分: 0.0,
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
                    element: element1, ..
                } = keys[1].clone()
                else {
                    unreachable!();
                };
                match first {
                    MappedKey::Ascii(key) => {
                        if element1 == "主根-1" {
                            元素安排::键位第一(key)
                        } else {
                            元素安排::键位第二(key)
                        }
                    }
                    MappedKey::Reference { element, .. } => {
                        if element.starts_with("声") {
                            元素安排::声母韵母 {
                                声母: 棱镜.元素转数字[&element],
                                韵母: 棱镜.元素转数字[&element1],
                            }
                        } else {
                            元素安排::归并韵母 {
                                字根: 棱镜.元素转数字[&element],
                                韵母: 棱镜.元素转数字[&element1],
                            }
                        }
                    }
                }
            }
            _ => {
                println!("无法从映射中恢复元素安排: {:?}", mapped);
                unreachable!()
            }
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
            元素安排::键位第一(键位) => Mapped::Advanced(vec![
                MappedKey::Ascii(*键位),
                MappedKey::Reference {
                    element: "主根-1".to_string(),
                    index: 0,
                },
            ]),
            元素安排::键位第二(键位) => Mapped::Advanced(vec![
                MappedKey::Ascii(*键位),
                MappedKey::Reference {
                    element: "主根-2".to_string(),
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
    pub 全局变化: bool,
    pub 移动字根: Vec<元素>,
    pub 增加字根: Vec<元素>,
    pub 减少字根: Vec<元素>,
}

impl 冰雪清韵决策变化 {
    pub fn 新建(
        全局变化: bool,
        移动字根: Vec<元素>,
        增加字根: Vec<元素>,
        减少字根: Vec<元素>,
    ) -> Self {
        冰雪清韵决策变化 {
            全局变化,
            移动字根,
            增加字根,
            减少字根,
        }
    }

    pub fn 无变化() -> Self {
        Self::新建(false, vec![], vec![], vec![])
    }

    pub fn 全局变化() -> Self {
        Self::新建(true, vec![], vec![], vec![])
    }
}

impl 解特征 for 冰雪清韵决策 {
    type 变化 = 冰雪清韵决策变化;

    fn 单位元() -> Self::变化 {
        冰雪清韵决策变化::无变化()
    }

    fn 除法(旧变化: &Self::变化, 新变化: &Self::变化) -> Self::变化 {
        let mut 移动字根 = 旧变化.移动字根.clone();
        let mut 增加字根 = 旧变化.减少字根.clone();
        let mut 减少字根 = 旧变化.增加字根.clone();
        for 元素 in &新变化.移动字根 {
            if !移动字根.contains(元素) {
                移动字根.push(*元素);
            }
        }
        for 元素 in &新变化.增加字根 {
            if !增加字根.contains(元素) {
                增加字根.push(*元素);
            }
        }
        for 元素 in &新变化.减少字根 {
            if !减少字根.contains(元素) {
                减少字根.push(*元素);
            }
        }
        冰雪清韵决策变化 {
            全局变化: 旧变化.全局变化 || 新变化.全局变化,
            移动字根,
            增加字根,
            减少字根,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 分析结果 {
    pub 重码项: Vec<(String, (Vec<String>, u64))>,
    pub 差指法: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 原始音节信息 {
    pub 拼音: String,
    pub 声母: String,
    pub 韵母: String,
    pub 频率: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 音节信息 {
    pub 声母: 元素,
    pub 韵母: 元素,
    pub 频率: 频率,
}
