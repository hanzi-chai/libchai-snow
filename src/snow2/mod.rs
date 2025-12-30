pub mod encoder;
pub mod objective;
pub mod operators;
use crate::common::转换;
use chai::{
    config::{Mapped, 配置}, contexts::{上下文, 合并初始决策, 拓扑排序}, interfaces::默认输入, objectives::metric::键盘布局, optimizers::决策, 元素, 棱镜
};
use chrono::Local;
use indexmap::IndexMap;
use rustc_hash::FxHashMap;
use serde_yaml::to_string;

pub const 声调总数: usize = 5;
pub const 大: usize = 21;
pub const 小: usize = 7;
pub const 全: usize = 30;
pub const 进制: u64 = 32;
pub const 大集合: [char; 大] = [
    'b', 'p', 'm', 'f', 'd', 't', 'n', 'l', 'g', 'k', 'h', 'j', 'q', 'x', 'z', 'c', 's', 'r', 'w',
    'y', 'v',
];
pub const 小集合: [char; 小] = ['a', 'o', 'e', 'i', 'u', ';', '/'];
pub type 键 = u8;
pub type 编码 = [键; 5];
pub const 空格: 键 = 31;

impl 转换 for 编码 {
    fn hash(&self) -> usize {
        let [c1, c2, c3, c4, c5] = *self;
        let 音码 = (c1 as usize - 小 - 1) + (c2 as usize - 1) * 大;
        let 形码 = c3 as usize + (c4 as usize * 小) + (c5 as usize * 小 * 小);
        音码 + 大 * 全 * 形码
    }

    fn 编码空间大小() -> usize {
        大 * 全 * (1 + 小 + 小 * 小 + 小 * 小 * 小)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum 冰雪二拼字根安排 {
    主根(键),
    副根(键, 键),
    归并(元素),
    未选取,
}

impl 冰雪二拼字根安排 {
    fn from(mapped: &Mapped, 棱镜: &棱镜) -> Self {
        match mapped {
            Mapped::Basic(s) => {
                let 字符列表: Vec<char> = s.chars().collect();
                if 字符列表.len() == 1 {
                    冰雪二拼字根安排::主根(棱镜.键转数字[&字符列表[0]] as 键)
                } else if 字符列表.len() == 2 {
                    冰雪二拼字根安排::副根(
                        棱镜.键转数字[&字符列表[0]] as 键,
                        棱镜.键转数字[&字符列表[1]] as 键,
                    )
                } else {
                    冰雪二拼字根安排::未选取
                }
            }
            Mapped::Grouped { element } => {
                冰雪二拼字根安排::归并(棱镜.元素转数字[element])
            }
            _ => 冰雪二拼字根安排::未选取,
        }
    }
}

#[derive(Clone, Debug)]
pub struct 冰雪二拼决策 {
    // 声母的行数
    pub 声母: FxHashMap<元素, 键>,
    // 韵母的列数
    pub 韵母: FxHashMap<String, usize>,
    // 声调的行数
    pub 声调: [usize; 声调总数],
    // 字根
    pub 字根: FxHashMap<元素, 冰雪二拼字根安排>,
}

pub type 线性化决策 = Vec<(键, 键)>;

impl 决策 for 冰雪二拼决策 {
    type 变化 = ();

    fn 除法(_旧变化: &Self::变化, _新变化: &Self::变化) -> Self::变化 {
        ()
    }
}

impl 冰雪二拼决策 {
    pub fn 线性化(
        &self,
        棱镜: &棱镜,
        韵母声调映射: &FxHashMap<元素, (String, usize)>,
    ) -> Vec<(键, 键)> {
        let mut 编码列表 = vec![(0, 0); 棱镜.元素转数字.len() + 1];
        for (&元素, &键) in &self.声母 {
            编码列表[元素] = (键, 0);
        }
        for (元素, (不带调韵母, 声调)) in 韵母声调映射 {
            let 列号 = self.韵母[不带调韵母];
            let 行号 = self.声调[*声调];
            let 键 = 键盘布局[行号][列号];
            编码列表[*元素] = (棱镜.键转数字[&键] as 键, 0);
        }
        for (&元素, 安排) in &self.字根 {
            match 安排 {
                冰雪二拼字根安排::主根(c) => {
                    编码列表[元素] = (*c, 0);
                }
                冰雪二拼字根安排::副根(c1, c2) => {
                    编码列表[元素] = (*c1, *c2);
                }
                冰雪二拼字根安排::归并(归并元素) => {
                    编码列表[元素] = 编码列表[*归并元素];
                }
                _ => {}
            }
        }
        编码列表
    }
}

#[derive(Clone, Debug)]
pub struct 冰雪二拼上下文 {
    pub 配置: 配置,
    pub 初始决策: 冰雪二拼决策,
    pub 声母列表: Vec<元素>,
    pub 韵母列表: Vec<元素>,
    // 带调韵母转换为韵母和声调
    pub 韵母声调映射: FxHashMap<元素, (String, usize)>,
    pub 字根列表: Vec<元素>,
    pub 信息列表: Vec<冰雪二拼信息>,
    pub 棱镜: 棱镜,
}

impl 上下文 for 冰雪二拼上下文 {
    type 决策 = 冰雪二拼决策;

    fn 序列化(&self, 决策: &Self::决策) -> String {
        let mut 新配置 = self.配置.clone();
        新配置.info.as_mut().unwrap().version =
            Some(format!("{}", Local::now().format("%Y-%m-%d+%H:%M:%S")));
        let mut mapping = IndexMap::new();
        for (元素, 键) in &决策.声母 {
            let 元素名称 = self.棱镜.数字转元素[&元素].clone();
            let 字母 = self.棱镜.数字转键[&(*键 as u64)];
            mapping.insert(元素名称, Mapped::Basic(字母.to_string()));
        }
        for (元素, (不带调韵母, 声调)) in &self.韵母声调映射 {
            let 元素名称 = self.棱镜.数字转元素[&元素].clone();
            let 列号 = 决策.韵母[不带调韵母];
            let 行号 = 决策.声调[*声调];
            let 韵母字母 = 键盘布局[行号][列号];
            mapping.insert(元素名称, Mapped::Basic(韵母字母.to_string()));
        }
        for (元素, 安排) in &决策.字根 {
            let 元素名称 = self.棱镜.数字转元素[&元素].clone();
            match 安排 {
                冰雪二拼字根安排::主根(键) => {
                    let c = self.棱镜.数字转键[&(*键 as u64)];
                    mapping.insert(元素名称, Mapped::Basic(c.to_string()));
                }
                冰雪二拼字根安排::副根(键一, 键二) => {
                    let c1 = self.棱镜.数字转键[&(*键一 as u64)];
                    let c2 = self.棱镜.数字转键[&(*键二 as u64)];
                    mapping.insert(元素名称, Mapped::Basic([c1, c2].iter().collect()));
                }
                冰雪二拼字根安排::归并(s) => {
                    let element = self.棱镜.数字转元素[s].clone();
                    mapping.insert(元素名称, Mapped::Grouped { element });
                }
                _ => {}
            }
        }
        新配置.form.mapping = mapping;
        to_string(&新配置).unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct 冰雪二拼信息 {
    pub 词: char,
    pub 频率: u64,
    pub 序列: [元素; 4],
    pub 独立一: bool,
    pub 独立二: bool,
}

impl 冰雪二拼上下文 {
    pub fn 新建(输入: &默认输入) -> Self {
        let 布局 = 输入.配置.form.clone();
        let 原始决策 = 布局.mapping;
        let mut 原始决策空间 = 布局.mapping_space.unwrap_or_default();
        合并初始决策(&mut 原始决策空间, &原始决策);
        let (所有元素, _) = 拓扑排序(&原始决策空间).unwrap();
        let mut 元素转数字 = FxHashMap::default();
        let mut 数字转元素 = FxHashMap::default();
        let mut 键转数字 = FxHashMap::default();
        let mut 数字转键 = FxHashMap::default();
        let mut 序号 = 0;
        for 键 in 小集合
            .into_iter()
            .chain(大集合.into_iter())
            .chain([',', '.'].into_iter())
        {
            序号 += 1;
            元素转数字.insert(键.to_string(), 序号);
            数字转元素.insert(序号, 键.to_string());
            键转数字.insert(键, 序号 as u64);
            数字转键.insert(序号 as u64, 键);
        }
        for 元素名称 in 所有元素 {
            序号 += 1;
            元素转数字.insert(元素名称.clone(), 序号);
            数字转元素.insert(序号, 元素名称.clone());
        }
        let mut 声母列表 = vec![];
        let mut 韵母列表 = vec![];
        let mut 字根列表 = vec![];
        let mut 韵母声调映射 = FxHashMap::default();
        let mut 初始决策 = 冰雪二拼决策 {
            声母: FxHashMap::default(),
            韵母: FxHashMap::default(),
            声调: [usize::MAX; 声调总数],
            字根: FxHashMap::default(),
        };
        let 棱镜 = 棱镜 {
            进制,
            元素转数字,
            数字转元素,
            键转数字,
            数字转键,
        };
        for (元素名称, 安排) in &原始决策 {
            let 序号 = 棱镜.元素转数字[元素名称];
            if 元素名称.starts_with("冰声") {
                let Mapped::Basic(编码) = 安排 else {
                    unreachable!()
                };
                let 字母 = 编码.chars().next().unwrap();
                let 键 = 棱镜.键转数字[&字母] as 键;
                初始决策.声母.insert(序号, 键);
                声母列表.push(序号);
            } else if 元素名称.starts_with("冰韵") {
                let Mapped::Basic(编码) = 安排 else {
                    continue;
                };
                let 字母 = 编码.chars().next().unwrap();
                let 行号 = 键盘布局.iter().position(|x| x.contains(&字母)).unwrap();
                let 列号 = 键盘布局[行号].iter().position(|&x| x == 字母).unwrap();
                韵母列表.push(序号);
                let 字符列表: Vec<char> = 元素名称.chars().collect();
                let 声调 = 字符列表[字符列表.len() - 1].to_digit(10).unwrap() - 1;
                let 无声调韵母: String = 字符列表[..(字符列表.len() - 1)].iter().collect();
                韵母声调映射.insert(序号, (无声调韵母.clone(), 声调 as usize));
                if !初始决策.韵母.contains_key(&无声调韵母) {
                    初始决策.韵母.insert(无声调韵母.clone(), 列号);
                }
                if 初始决策.声调[声调 as usize] == usize::MAX {
                    初始决策.声调[声调 as usize] = 行号;
                }
            } else {
                字根列表.push(序号);
                let 安排 = 冰雪二拼字根安排::from(安排, &棱镜);
                初始决策.字根.insert(序号, 安排);
            }
        }
        let mut 信息列表 = vec![];
        for 原始信息 in &输入.词列表 {
            let 字符列表: Vec<char> = 原始信息.name.chars().collect();
            let 原始序列: Vec<_> = 原始信息
                .sequence
                .split(' ')
                .map(|x| x.to_string())
                .collect();
            let mut 序列 = [0; 4];
            for (i, 元素名称) in 原始序列
                .iter()
                .filter(|x| !["q", "w", "e", "r"].contains(&x.as_str()))
                .enumerate()
            {
                let 元素编号 = 棱镜.元素转数字.get(元素名称).unwrap();
                序列[i] = *元素编号;
            }
            let (独立一, 独立二) = match 原始序列[原始序列.len() - 1].as_str() {
                "q" => (true, true),
                "w" => (false, true),
                "e" => (true, false),
                "r" => (false, false),
                _ => unreachable!(),
            };
            信息列表.push(冰雪二拼信息 {
                词: 字符列表[0],
                序列,
                独立一,
                独立二,
                频率: 原始信息.frequency,
            });
        }
        信息列表.sort_by(|a, b| b.频率.cmp(&a.频率));
        Self {
            配置: 输入.配置.clone(),
            声母列表,
            韵母列表,
            字根列表,
            韵母声调映射,
            初始决策,
            信息列表,
            棱镜,
        }
    }
}
