use chai::{encoders::编码器, 元素, 元素映射, 棱镜, 编码信息, 错误};
use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde_yaml::from_str;
use std::{collections::HashMap, fs::read_to_string, iter::zip};

use crate::qingyun::{
    冰雪清韵上下文, 冰雪清韵决策, 冰雪清韵决策变化, 最大码长, 进制
};

type 块 = usize;
type 固定拆分 = Vec<[块; 4]>;
type 动态拆分 = Vec<Vec<[元素; 4]>>;

pub struct 冰雪清韵编码器 {
    pub 固定拆分: 固定拆分,
    pub 动态拆分: 动态拆分,
    pub 块转数字: FxHashMap<String, usize>,
    pub 数字转块: FxHashMap<usize, String>,
    pub 拆分序列: Vec<[元素; 4]>,
    pub 全码空间: Vec<u8>,
    pub 棱镜: 棱镜,
}

type 原始固定拆分 = HashMap<String, Vec<String>>;
type 原始动态拆分 = HashMap<String, Vec<Vec<String>>>;

#[derive(Deserialize)]
struct 拆分输入 {
    固定拆分: 原始固定拆分,
    动态拆分: 原始动态拆分,
}

impl 冰雪清韵编码器 {
    pub fn 新建(上下文: &冰雪清韵上下文) -> Result<Self, 错误> {
        let 编码空间大小 = 进制.pow(最大码长 as u32) as usize;
        let 全码空间 = vec![u8::default(); 编码空间大小];
        let 拆分输入: 拆分输入 = from_str(&read_to_string("dynamic_analysis.yaml")?).unwrap();
        let mut 动态拆分 = vec![vec![]];
        let mut 块转数字 = FxHashMap::default();
        let mut 数字转块 = FxHashMap::default();
        for (块, 原始拆分方式列表) in 拆分输入.动态拆分 {
            let 块序号 = 动态拆分.len();
            块转数字.insert(块.clone(), 块序号);
            数字转块.insert(块序号, 块.clone());
            let mut 拆分方式列表 = vec![];
            for 原始拆分方式 in &原始拆分方式列表 {
                let 拆分方式 = Self::对齐(
                    原始拆分方式
                        .iter()
                        .map(|字根| 上下文.棱镜.元素转数字[字根])
                        .collect(),
                );
                拆分方式列表.push(拆分方式);
            }
            动态拆分.push(拆分方式列表);
        }
        let 固定拆分: Vec<_> = 上下文
            .词列表
            .iter()
            .map(|词| {
                let 原始块列表 = &拆分输入.固定拆分[&词.name];
                Self::对齐(原始块列表.iter().map(|块| 块转数字[块]).collect())
            })
            .collect();
        let mut 拆分序列 = vec![<[元素; 4]>::default(); 上下文.词列表.len()];
        Self::构建拆分序列(
            &mut 拆分序列,
            &动态拆分,
            &固定拆分,
            &上下文.初始决策,
            &上下文.棱镜,
            &数字转块,
        );
        Ok(Self {
            动态拆分,
            固定拆分,
            块转数字,
            数字转块,
            拆分序列,
            全码空间,
            棱镜: 上下文.棱镜.clone(),
        })
    }

    fn 对齐(列表: Vec<元素>) -> [元素; 4] {
        [0, 1, 2, 3].map(|i| {
            if i == 3 && 列表.len() > 3 {
                列表[列表.len() - 1]
            } else if i < 列表.len() {
                列表[i]
            } else {
                0
            }
        })
    }

    fn 构建拆分序列(
        拆分序列: &mut Vec<[元素; 4]>,
        动态拆分: &动态拆分,
        固定拆分: &固定拆分,
        决策: &冰雪清韵决策,
        棱镜: &棱镜,
        数字转块: &FxHashMap<usize, String>,
    ) {
        let 映射 = 决策.线性化(&棱镜);
        let mut 当前拆分索引 = vec![0_usize; 动态拆分.len()];
        for (块序号, 拆分方式列表) in 动态拆分.iter().enumerate() {
            if 块序号 == 0 {
                continue;
            }
            let mut 找到 = false;
            for (拆分方式序号, 拆分方式) in 拆分方式列表.iter().enumerate() {
                if 拆分方式.iter().all(|x| *x == 0 || 映射[*x] != 0) {
                    当前拆分索引[块序号] = 拆分方式序号;
                    找到 = true;
                    break;
                }
            }
            if !找到 {
                panic!(
                    "未找到 {} 的映射: {:?}",
                    数字转块[&块序号],
                    拆分方式列表
                        .iter()
                        .map(|x| x.map(|y| {
                            if y == 0 {
                                "".to_string()
                            } else {
                                棱镜.数字转元素[&y].clone()
                            }
                        }))
                        .collect::<Vec<_>>()
                );
            }
        }
        for 词索引 in 0..拆分序列.len() {
            let mut 总序列: Vec<元素> = Default::default();
            for 块序号 in 固定拆分[词索引] {
                if 块序号 == 0 {
                    break;
                }
                for 元素 in 动态拆分[块序号][当前拆分索引[块序号]] {
                    if 元素 == 0 {
                        break;
                    }
                    总序列.push(元素);
                }
            }
            let mut 序列 = Self::对齐(总序列);
            if 序列[1] == 0 {
                序列[1] = 序列[0] + 1;
            } else if 序列[2] == 0 {
                序列[2] = 序列[1] + 1;
            } else if 序列[3] == 0 {
                序列[3] = 序列[2] + 1;
            }
            拆分序列[词索引] = 序列;
        }
    }

    pub fn 重置空间(&mut self) {
        self.全码空间.iter_mut().for_each(|x| {
            *x = 0;
        });
    }

    #[inline(always)]
    fn 全码规则(元素序列: &[元素; 4], 映射: &元素映射) -> u64 {
        映射[元素序列[0]]
            + (映射[元素序列[1]]) * 进制
            + (映射[元素序列[2]]) * 进制 * 进制
            + (映射[元素序列[3]]) * 进制 * 进制 * 进制
    }

    fn 输出全码(
        &mut self,
        编码结果: &mut [编码信息],
        决策: &冰雪清韵决策,
        _决策变化: &Option<冰雪清韵决策变化>,
    ) {
        let 映射 = 决策.线性化(&self.棱镜);
        for (词, 编码信息) in zip(&self.拆分序列, 编码结果.iter_mut()) {
            编码信息.全码.原始编码 = Self::全码规则(词, &映射);
        }

        for 编码信息 in 编码结果.iter_mut() {
            let 全码信息 = &mut 编码信息.全码;
            全码信息.原始编码候选位置 = self.全码空间[全码信息.原始编码 as usize];
            self.全码空间[全码信息.原始编码 as usize] += 1;
            全码信息.更新(全码信息.原始编码, 全码信息.原始编码候选位置 > 0);
        }
    }
}

impl 编码器 for 冰雪清韵编码器 {
    type 解类型 = 冰雪清韵决策;
    fn 编码(
        &mut self,
        决策: &冰雪清韵决策,
        决策变化: &Option<冰雪清韵决策变化>,
        输出: &mut [编码信息],
    ) {
        self.重置空间();
        self.输出全码(输出, 决策, 决策变化);
    }
}
