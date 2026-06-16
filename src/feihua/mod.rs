pub mod encoder;
pub mod objective;
pub mod operators;
use crate::{
    common::转换, feihua::encoder::冰雪飞花编码信息, qingyun::context::写入文本文件
};
use chai::{
    config::{基本信息, 安排, 广义码位, 配置}, contexts::{
        default::默认决策变化, 上下文, 拓扑排序, 条件, 条件安排,
    }, formatted_local_now, interfaces::默认输入, optimizers::决策, 位图, 元素, 原始元素序列及条件列表, 原始可编码对象, 棱镜, 码表项, 错误
};
use indexmap::IndexMap;
use rustc_hash::FxHashMap;
use serde_yaml::to_string;
use std::{cmp::Reverse, io::Write};
use std::{fs::File, path::PathBuf};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum 冰雪飞花安排 {
    键位(键),
    归并(元素),
    未选取,
}

type 冰雪飞花条件安排 = 条件安排<冰雪飞花安排>;

impl 冰雪飞花安排 {
    pub fn from(mapped: &安排, 棱镜: &棱镜) -> Self {
        match mapped {
            安排::Basic(s) => {
                let 字母 = s.chars().next().unwrap();
                let 键 = 棱镜.键转数字[&字母] as 键;
                冰雪飞花安排::键位(键)
            }
            安排::Advanced(keys) => {
                let 广义码位::Ascii(字母) = keys[0] else {
                    panic!("Unexpected key type");
                };
                let 键 = 棱镜.键转数字[&字母] as 键;
                冰雪飞花安排::键位(键)
            }
            安排::Grouped { element } => 冰雪飞花安排::归并(棱镜.元素转数字[element]),
            安排::Unused(()) => 冰雪飞花安排::未选取,
        }
    }
}

#[derive(Clone, Debug)]
pub struct 冰雪飞花决策 {
    pub 元素: Vec<冰雪飞花安排>,
}

impl 决策 for 冰雪飞花决策 {
    type 变化 = 默认决策变化;

    fn 除法(旧变化: &Self::变化, 新变化: &Self::变化) -> Self::变化 {
        let mut res = 默认决策变化 {
            增加元素: 旧变化.减少元素.clone(),
            减少元素: 旧变化.增加元素.clone(),
            移动元素: 旧变化.移动元素.clone(),
        };
        for 元素 in &新变化.增加元素 {
            res.增加元素.push(*元素);
        }
        for 元素 in &新变化.减少元素 {
            res.减少元素.push(*元素);
        }
        for 元素 in &新变化.移动元素 {
            res.移动元素.push(*元素);
        }
        res
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
                冰雪飞花安排::未选取 => {
                    编码列表[i] = 0;
                }
            }
        }
        编码列表
    }

    pub fn 允许(&self, 条件安排: &冰雪飞花条件安排) -> bool {
        for 条件 in &条件安排.条件 {
            if 条件.谓词 != (self.元素[条件.元素] == 条件.值) {
                return false;
            }
        }
        return true;
    }
}

#[derive(Clone, Debug)]
pub struct 冰雪飞花决策空间 {
    pub 元素: Vec<Vec<条件安排<冰雪飞花安排>>>,
}

#[derive(Clone, Debug)]
pub struct 冰雪飞花可编码对象 {
    pub 词: String,
    pub 元素序列: [元素; 4],
    pub 全部元素序列: Vec<([元素; 4], 位图, 位图)>,
    pub 频率: u64,
    pub 原始顺序: usize,
}

#[derive(Clone, Debug)]
pub struct 冰雪飞花上下文 {
    pub 配置: 配置,
    pub 初始决策: 冰雪飞花决策,
    pub 决策空间: 冰雪飞花决策空间,
    pub 棱镜: 棱镜,
    pub 词列表: Vec<冰雪飞花可编码对象>,
    pub 元素图: FxHashMap<元素, Vec<元素>>,
}

impl 上下文 for 冰雪飞花上下文 {
    type 决策 = 冰雪飞花决策;

    fn 序列化(&self, 决策: &Self::决策) -> String {
        let mut 新配置 = self.配置.clone();
        let mut info = 新配置.info.clone().unwrap_or(基本信息 {
            name: None,
            description: None,
            version: None,
            author: None,
        });
        info.version = Some(formatted_local_now());
        新配置.info = Some(info);
        let mut mapping = IndexMap::new();
        for (序号, 安排) in 决策.元素.iter().enumerate() {
            if 序号 <= 空格 as usize {
                continue;
            }
            let 元素名称 = self.棱镜.数字转元素[&序号].clone();
            match 安排 {
                冰雪飞花安排::键位(键) => {
                    let 字母 = self.棱镜.数字转键[&(*键 as u64)];
                    mapping.insert(元素名称, 安排::Basic(字母.to_string()));
                }
                冰雪飞花安排::归并(元素) => {
                    let element = self.棱镜.数字转元素[&元素].clone();
                    mapping.insert(元素名称, 安排::Grouped { element });
                }
                冰雪飞花安排::未选取 => {}
            }
        }
        新配置.form.mapping = mapping;
        新配置.form.mapping_space = None;
        to_string(&新配置).unwrap()
    }
}

impl 冰雪飞花上下文 {
    pub fn 新建(输入: &默认输入) -> Self {
        let 布局 = 输入.配置.form.clone();
        let mut 原始决策 = 布局.mapping;
        let 原始决策空间 = 输入.配置.generated_mapping_space.clone().unwrap_or_default();
        for 元素名称 in 原始决策空间.keys() {
            if !原始决策.contains_key(元素名称) {
                原始决策.insert(元素名称.clone(), 安排::Unused(()));
            }
        }
        let (排序后元素名称列表, 原始元素图) = 拓扑排序(&原始决策空间).unwrap();
        let mut 元素图: FxHashMap<元素, Vec<_>> = FxHashMap::default();
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
        for 元素名称 in &排序后元素名称列表 {
            序号 += 1;
            元素转数字.insert(元素名称.clone(), 序号);
            数字转元素.insert(序号, 元素名称.clone());
        }
        let mut 可选元素位图索引 = FxHashMap::default();
        for v in 元素转数字.values() {
            可选元素位图索引.insert(*v, *v);
        }
        let 棱镜 = 棱镜 {
            进制: 32,
            元素转数字,
            数字转元素,
            键转数字,
            数字转键,
            可选元素位图索引,
        };
        let mut 初始决策 = 冰雪飞花决策 {
            元素: vec![冰雪飞花安排::键位(0); 棱镜.元素转数字.len() + 1],
        };
        let mut 决策空间 = 冰雪飞花决策空间 {
            元素: vec![vec![]; 棱镜.元素转数字.len() + 1],
        };
        for 元素名称 in &排序后元素名称列表 {
            let 原始安排 = &原始决策[元素名称];
            let 原始安排列表 = 原始决策空间[元素名称].clone();
            let 序号 = 棱镜.元素转数字[元素名称];
            let 安排 = 冰雪飞花安排::from(原始安排, &棱镜);
            let mut 条件安排列表 = vec![];
            for 条件安排 in 原始安排列表 {
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
            初始决策.元素[序号] = 安排;
            决策空间.元素[序号] = 条件安排列表;
            let 下游 = 原始元素图.get(元素名称).unwrap();
            let 下游编号: Vec<_> = 下游.iter().map(|x| 棱镜.元素转数字[x]).collect();
            元素图.insert(序号, 下游编号);
        }
        let 词列表 = Self::预处理词列表(&输入.词列表, &棱镜);
        Self {
            配置: 输入.配置.clone(),
            初始决策,
            决策空间,
            棱镜,
            词列表,
            元素图,
        }
    }

    pub fn 预处理词列表(
        词列表: &Vec<原始可编码对象>,
        棱镜: &棱镜,
    ) -> Vec<冰雪飞花可编码对象> {
        let mut 词信息列表 = vec![];
        for (原始顺序, 原始可编码对象) in 词列表.into_iter().enumerate() {
            let 原始可编码对象 {
                词,
                频率,
                全部元素序列: 原始全部元素序列,
                ..
            } = 原始可编码对象.clone();
            let mut 全部元素序列 = vec![];
            assert!(
                原始全部元素序列
                    .clone()
                    .unwrap()
                    .last()
                    .unwrap()
                    .条件列表
                    .is_empty(),
                "编码对象「{词}」的最后一个元素序列必须没有任何条件",
                词 = 词
            );
            for 原始元素序列及条件列表 {
                元素序列, 条件列表
            } in 原始全部元素序列.unwrap()
            {
                let 元素序列 = 棱镜.预处理元素序列(&词, &元素序列, 4).unwrap();
                let mut 新元素序列 = [0; 4];
                新元素序列.copy_from_slice(&元素序列[0..4]);
                let 负条件列表: Vec<_> = 条件列表
                    .iter()
                    .filter(|c| c.op == "不是")
                    .cloned()
                    .collect();
                let 正条件列表: Vec<_> =
                    条件列表.iter().filter(|c| c.op == "是").cloned().collect();
                let 全集合位图 = 位图::从条件列表创建(&负条件列表, 棱镜);
                let 小集合位图 = 位图::从条件列表创建(&正条件列表, 棱镜);
                全部元素序列.push((新元素序列, 全集合位图, 小集合位图));
            }
            词信息列表.push(冰雪飞花可编码对象 {
                词: 词.clone(),
                元素序列: 全部元素序列[0].0,
                全部元素序列,
                频率,
                原始顺序,
            });
        }
        词信息列表.sort_by_key(|x| Reverse(x.频率));
        词信息列表
    }

    pub fn 生成码表(&self, 编码结果: &Vec<冰雪飞花编码信息>) -> Vec<码表项> {
        let mut 码表: Vec<码表项> = Vec::new();
        let 转编码 = |code: 编码| {
            code.0
                .iter()
                .filter(|x| **x != 0)
                .map(|x| self.棱镜.数字转键[&(*x as u64)])
                .collect()
        };
        for (序号, 可编码对象) in self.词列表.iter().enumerate() {
            let 码表项 = 码表项 {
                词: 可编码对象.词.to_string(),
                全码: 转编码(编码结果[序号].全码),
                全码排名: 编码结果[序号].候选位置,
                简码: 转编码(编码结果[序号].简码),
                简码排名: 0,
            };
            码表.push(码表项);
        }
        码表
    }

    pub fn 输出码表(
        &self, 输出目录: &PathBuf, 码表: &Vec<码表项>
    ) -> Result<(), 错误> {
        let 码表路径 = 输出目录.join("code.txt");
        写入文本文件(码表路径, 码表);
        let mut 大竹码表 = vec![];
        for 码表项 in 码表 {
            大竹码表.push((format!("({})", 码表项.全码.clone()), 码表项.词.clone()));
        }
        let 大竹码表路径 = 输出目录.join("dazhu.txt");
        写入文本文件(大竹码表路径, &大竹码表);
        Ok(())
    }

    // 分析前 3000 字中全码重码和简码差指法的情况
    pub fn 分析码表(
        &self,
        编码结果: &[冰雪飞花编码信息],
        码表: &[码表项],
        路径: &PathBuf,
    ) -> Result<(), 错误> {
        let mut 文件 = File::create(路径).unwrap();
        // 全码 -> 词列表的映射
        let mut 翻转码表 = FxHashMap::default();
        let 一字总频率: u64 = 编码结果.iter().map(|x| x.频率).sum();
        for (序号, 码表项) in 码表.iter().enumerate() {
            let 百万分之频率 = (编码结果[序号].频率 as f64 / 一字总频率 as f64) * 1_000_000.0;
            翻转码表
                .entry(码表项.全码.clone())
                .or_insert_with(|| vec![])
                .push((码表项.词.clone(), 百万分之频率 as u64));
        }
        let mut 重码 = vec![];
        for 码表项 in 码表.iter().take(4000) {
            let 是重码 = 码表项.全码排名 != 0;
            if 是重码 {
                let 完整重码组 = 翻转码表[&码表项.全码].clone();
                let 位置 = 完整重码组
                    .iter()
                    .position(|(x, _)| x == &码表项.词)
                    .unwrap();
                let 百万分之频率 = 完整重码组[位置].1;
                重码.push((
                    码表项.词.clone(),
                    码表项.全码.clone(),
                    百万分之频率,
                    完整重码组,
                ));
            }
        }
        writeln!(文件, "# 前 4000 中重码\n")?;
        for (name, code, frequency, names) in 重码 {
            writeln!(文件, "- {name} {code} {frequency}μ：{names:?}")?;
        }
        Ok(())
    }
}
