pub mod encoder;
pub mod objective;
pub mod operators;
use crate::{
    common::转换, feihua::encoder::冰雪飞花编码信息, qingyun::context::写入文本文件
};
use chai::{
    config::{Mapped, 配置},
    contexts::{上下文, 合并初始决策, 展开变量, 拓扑排序, 条件, 条件安排},
    interfaces::默认输入,
    optimizers::决策,
    元素, 棱镜, 码表项,
};
use chrono::Local;
use indexmap::IndexMap;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde_yaml::{from_str, to_string};
use std::fs::read_to_string;

pub const 大: usize = 20;
pub const 小: usize = 8;
pub const 全: usize = 30;
pub const 大集合: [char; 大] = [
    'b', 'p', 'm', 'f', 'd', 't', 'n', 'l', 'g', 'k', 'h', 'j', 'q', 'x', 'z', 'c', 's', 'r', 'w',
    'y',
];
pub const 小集合: [char; 小] = ['a', 'o', 'e', 'i', 'u', 'v', ';', '/'];
pub const 全集合: [char; 全] = [
    'b', 'p', 'm', 'f', 'd', 't', 'n', 'l', 'g', 'k', 'h', 'j', 'q', 'x', 'z', 'c', 's', 'r', 'w',
    'y', 'a', 'o', 'e', 'i', 'u', 'v', ';', '/', ',', '.',
];
pub type 键 = u8;
#[derive(Default, Copy, Clone, Debug)]
pub struct 编码([键; 4]);
pub const 空格: 键 = 31;

impl 转换 for 编码 {
    fn hash(&self) -> usize {
        let [c1, c2, c3, c4] = self.0;
        let 声母 = c1 as usize - 小 - 1;
        let 部首 = c2 as usize - 1;
        let 形码 = c3 as usize + (c4 as usize * 小);
        声母 + 部首 * 大 + 形码 * 大 * 全
    }

    fn 编码空间大小() -> usize {
        大 * 全 * (1 + 小 + 小 * 小)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum 冰雪飞花安排 {
    键位(键),
    归并(元素),
}

impl 冰雪飞花安排 {
    pub fn from(mapped: &Mapped, 棱镜: &棱镜) -> Self {
        match mapped {
            Mapped::Basic(s) => {
                let 字母 = s.chars().next().unwrap();
                let 键 = 棱镜.键转数字[&字母] as 键;
                冰雪飞花安排::键位(键)
            }
            Mapped::Grouped { element } => {
                冰雪飞花安排::归并(棱镜.元素转数字[element])
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct 冰雪飞花决策 {
    pub 元素: Vec<冰雪飞花安排>,
}

impl 决策 for 冰雪飞花决策 {
    type 变化 = ();

    fn 除法(_旧变化: &Self::变化, _新变化: &Self::变化) -> Self::变化 {
        ()
    }
}

pub type 线性化决策 = Vec<键>;

impl 冰雪飞花决策 {
    pub fn 线性化(&self, _上下文: &棱镜) -> 线性化决策 {
        let mut 编码列表 = vec![0; self.元素.len()];
        for (i, 元素安排) in self.元素.iter().enumerate() {
            match 元素安排 {
                冰雪飞花安排::键位(键) => {
                    编码列表[i] = *键;
                }
                冰雪飞花安排::归并(元素) => {
                    编码列表[i] = 编码列表[*元素];
                }
            }
        }
        编码列表
    }
}

#[derive(Clone, Debug)]
pub struct 冰雪飞花决策空间 {
    pub 元素空间: Vec<Vec<条件安排<冰雪飞花安排>>>,
}

#[derive(Clone, Debug)]
pub struct 冰雪飞花汉字信息 {
    pub 汉字: char,
    pub 频率: u64,
    pub 声母: 元素,
    // 0 表示没有部首
    pub 部首: 元素,
    pub 字块: [usize; 2],
}

pub type 动态拆分项 = Vec<[usize; 3]>;

#[derive(Clone, Debug)]
pub struct 冰雪飞花上下文 {
    pub 配置: 配置,
    pub 初始决策: 冰雪飞花决策,
    pub 决策空间: 冰雪飞花决策空间,
    pub 棱镜: 棱镜,
    pub 信息列表: Vec<冰雪飞花汉字信息>,
    pub 动态拆分: Vec<动态拆分项>,
}

impl 上下文 for 冰雪飞花上下文 {
    type 决策 = 冰雪飞花决策;

    fn 序列化(&self, 决策: &Self::决策) -> String {
        let mut 新配置 = self.配置.clone();
        新配置.info.as_mut().unwrap().version =
            Some(format!("{}", Local::now().format("%Y-%m-%d+%H:%M:%S")));
        let mut mapping = IndexMap::new();
        for (序号, 元素安排) in 决策.元素.iter().enumerate() {
            if 序号 <= 空格 as usize {
                continue;
            }
            let 元素名称 = self.棱镜.数字转元素[&序号].clone();
            match 元素安排 {
                冰雪飞花安排::键位(键) => {
                    let 字母 = self.棱镜.数字转键[&(*键 as u64)];
                    mapping.insert(元素名称, Mapped::Basic(字母.to_string()));
                }
                冰雪飞花安排::归并(元素) => {
                    let element = self.棱镜.数字转元素[&元素].clone();
                    mapping.insert(元素名称, Mapped::Grouped { element });
                }
            }
        }
        新配置.form.mapping = mapping;
        to_string(&新配置).unwrap()
    }
}

#[derive(Deserialize)]
struct 拆分输入 {
    汉字信息: 原始汉字信息,
    动态拆分: 原始动态拆分,
}

#[derive(Deserialize)]
struct 原始读音 {
    频率: u64,
    声: String,
}

#[derive(Deserialize)]
struct 原始汉字信息项 {
    pub 汉字: char,
    pub 读音: Vec<原始读音>,
    pub 部首: Option<String>,
    pub 字块: Vec<String>,
}

type 原始汉字信息 = Vec<原始汉字信息项>;
type 原始动态拆分 = FxHashMap<String, Vec<Vec<String>>>;

impl 冰雪飞花上下文 {
    pub fn 新建(输入: &默认输入) -> Self {
        let 布局 = 输入.配置.form.clone();
        let 原始决策 = 布局.mapping;
        let mut 原始决策空间 = 布局.mapping_space.unwrap_or_default();
        let 原始变量映射 = 布局.mapping_variables.unwrap_or_default();
        合并初始决策(&mut 原始决策空间, &原始决策);
        展开变量(&mut 原始决策空间, &原始变量映射);
        let (所有元素, _) = 拓扑排序(&原始决策空间).unwrap();
        let mut 元素转数字 = FxHashMap::default();
        let mut 数字转元素 = FxHashMap::default();
        let mut 键转数字 = FxHashMap::default();
        let mut 数字转键 = FxHashMap::default();
        let mut 序号 = 0;
        for 键 in 小集合
            .into_iter()
            .chain(大集合.into_iter())
            .chain([',', '.', '_'].into_iter())
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
        let 棱镜 = 棱镜 {
            进制: 32 as u64,
            元素转数字,
            数字转元素,
            键转数字,
            数字转键,
        };
        let mut 初始决策 = 冰雪飞花决策 {
            元素: vec![冰雪飞花安排::键位(0); 棱镜.元素转数字.len() + 1],
        };
        for (元素名称, 安排) in &原始决策 {
            let 序号 = 棱镜.元素转数字[元素名称];
            let 安排 = 冰雪飞花安排::from(安排, &棱镜);
            初始决策.元素[序号] = 安排;
        }
        let mut 决策空间 = 冰雪飞花决策空间 {
            元素空间: vec![vec![]; 棱镜.元素转数字.len() + 1],
        };
        for (元素名称, 安排列表) in &原始决策空间 {
            let 序号 = 棱镜.元素转数字[元素名称];
            let mut 条件安排列表 = vec![];
            for 条件安排 in 安排列表 {
                let 安排 = 冰雪飞花安排::from(&条件安排.value, &棱镜);
                let mut 条件列表 = vec![];
                for 条件 in 条件安排.condition.as_ref().unwrap_or(&vec![]) {
                    条件列表.push(条件 {
                        元素: 棱镜.元素转数字[&条件.element],
                        谓词: 条件.op == "是",
                        值: 冰雪飞花安排::from(&条件.value, &棱镜),
                    });
                }
                条件安排列表.push(条件安排 {
                    条件: 条件列表,
                    安排: 安排,
                    分数: 条件安排.score,
                });
            }
            决策空间.元素空间[序号] = 条件安排列表;
        }
        let 拆分输入: 拆分输入 =
            from_str(&read_to_string("feihua/dynamic_analysis.yaml").unwrap()).unwrap();
        let mut 动态拆分 = vec![];
        let mut 块转数字 = FxHashMap::default();
        let mut 数字转块 = FxHashMap::default();
        for (块, 原始拆分方式列表) in 拆分输入.动态拆分 {
            let 块序号 = 动态拆分.len();
            块转数字.insert(块.clone(), 块序号);
            数字转块.insert(块序号, 块.clone());
            let mut 拆分方式列表 = vec![];
            for 原始拆分方式 in &原始拆分方式列表 {
                let mut 拆分方式 = [0; 3];
                for (索引, 字根) in 原始拆分方式.iter().enumerate().take(3) {
                    let 字根序号 = 棱镜.元素转数字[字根];
                    拆分方式[索引] = 字根序号;
                }
                拆分方式列表.push(拆分方式);
            }
            动态拆分.push(拆分方式列表);
        }
        let mut 信息列表 = vec![];
        for 原始信息 in 拆分输入.汉字信息 {
            let mut 字块 = [usize::MAX; 2];
            for (索引, 块) in 原始信息.字块.iter().enumerate().take(2) {
                let 块序号 = 块转数字[块];
                字块[索引] = 块序号;
            }
            let 部首 = if let Some(部首名称) = &原始信息.部首 {
                棱镜.元素转数字[部首名称]
            } else {
                0
            };
            let mut freq = FxHashMap::default();
            for 读音 in &原始信息.读音 {
                let 声母 = 棱镜.元素转数字[&读音.声];
                freq.entry(声母)
                    .and_modify(|f| *f += 读音.频率)
                    .or_insert(读音.频率);
            }
            for (声母, 频率) in freq {
                信息列表.push(冰雪飞花汉字信息 {
                    汉字: 原始信息.汉字,
                    频率,
                    声母,
                    部首,
                    字块,
                });
            }
        }
        信息列表.sort_by_key(|x| std::cmp::Reverse(x.频率));
        Self {
            配置: 输入.配置.clone(),
            初始决策,
            决策空间,
            棱镜,
            信息列表,
            动态拆分,
        }
    }

    pub fn 生成码表(&self, 编码结果: &Vec<冰雪飞花编码信息>) {
        let mut 码表: Vec<码表项> = Vec::new();
        let 转编码 = |code: 编码| {
            code.0
                .iter()
                .filter(|x| **x != 0)
                .map(|x| self.棱镜.数字转键[&(*x as u64)])
                .collect()
        };
        for (序号, 可编码对象) in self.信息列表.iter().enumerate() {
            let 码表项 = 码表项 {
                name: 可编码对象.汉字.to_string(),
                full: 转编码(编码结果[序号].全码),
                full_rank: 编码结果[序号].候选位置,
                short: 转编码(编码结果[序号].简码),
                short_rank: 0,
            };
            码表.push(码表项);
        }
        码表.sort_by_key(|x| (x.name.chars().next().unwrap(), x.full.clone()));
        写入文本文件("feihua/code.txt".into(), 码表);
    }
}
