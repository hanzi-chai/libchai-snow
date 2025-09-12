use chai::{encoders::编码器, 元素, 元素映射, 棱镜, 编码, 编码信息, 错误};
use core::panic;
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use std::{array::from_fn, cmp::Ordering, collections::BinaryHeap, iter::zip};

use crate::{
    qingyun::{
        context::冰雪清韵上下文, 优先简码, 冰雪清韵决策, 冰雪清韵决策变化, 冰雪清韵编码信息,
        动态拆分项, 固定拆分项, 大集合, 小集合, 最大码长, 特简码, 空格, 进制, 频率,
    },
    time_block,
};

pub struct 冰雪清韵编码器 {
    pub 固定拆分: Vec<固定拆分项>,
    pub 动态拆分: Vec<动态拆分项>,
    pub 块转数字: FxHashMap<String, usize>,
    pub 数字转块: FxHashMap<usize, String>,
    pub 优先简码: FxHashSet<usize>,
    pub 简体空间: Vec<bool>,
    pub 繁体空间: Vec<bool>,
    pub 简繁通打空间: Vec<bool>,
    pub 棱镜: 棱镜,
    pub 当量信息: Vec<f64>,
    pub 全部出简: bool,
    pub 繁体顺序: Vec<usize>,
    pub 简繁通打顺序: Vec<usize>,
}

const 最大备选长度: usize = 12;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct 队列 {
    pub 数据: [(usize, 频率); 最大备选长度],
    pub 当前索引: usize,
    pub 长度: usize,
    pub 二简: 编码,
}

impl 队列 {
    fn 入队(&mut self, 序号: usize, 频率: 频率) {
        if self.长度 < 最大备选长度 {
            self.数据[self.长度] = (序号, 频率);
            self.长度 += 1;
        }
    }

    fn 出队(&mut self) -> (usize, 频率) {
        let 数据 = self.数据[self.当前索引];
        self.当前索引 += 1;
        数据
    }

    fn 频率(&self) -> 频率 {
        if self.二简 == 0 {
            self.数据[self.当前索引].1
        } else {
            self.数据[self.当前索引].1 + self.数据[self.当前索引 + 1].1
        }
    }
}

impl Eq for 队列 {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialOrd for 队列 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.频率().partial_cmp(&other.频率())
    }
}

impl Ord for 队列 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.频率().partial_cmp(&other.频率()).unwrap()
    }
}

#[derive(Debug, Clone, Default)]
struct 出简子问题数据 {
    pub 三码全码队列: 队列,
    pub 四码全码队列: [队列; 21],
    pub 一简十重: Vec<编码>,
}

impl 冰雪清韵编码器 {
    pub fn 新建(上下文: &冰雪清韵上下文, 全部出简: bool) -> Result<Self, 错误> {
        let 当量信息 = 上下文
            .棱镜
            .预处理当量信息(&上下文.原始当量信息, 进制.pow(最大码长 as u32) as usize);
        let 编码空间大小 = 进制.pow(最大码长 as u32) as usize;
        let 简体空间 = vec![Default::default(); 编码空间大小];
        let 繁体空间 = vec![Default::default(); 编码空间大小];
        let 简繁通打空间 = vec![Default::default(); 编码空间大小];
        let 优先简码集合: FxHashSet<usize> = 上下文
            .固定拆分
            .iter()
            .enumerate()
            .filter_map(|(序号, 词)| {
                if 优先简码.contains(&词.词) {
                    Some(序号)
                } else {
                    None
                }
            })
            .collect();
        Ok(Self {
            动态拆分: 上下文.动态拆分.clone(),
            固定拆分: 上下文.固定拆分.clone(),
            块转数字: 上下文.块转数字.clone(),
            数字转块: 上下文.数字转块.clone(),
            简体空间,
            繁体空间,
            简繁通打空间,
            棱镜: 上下文.棱镜.clone(),
            当量信息,
            优先简码: 优先简码集合,
            全部出简,
            繁体顺序: 上下文.繁体顺序.clone(),
            简繁通打顺序: 上下文.简繁通打顺序.clone(),
        })
    }

    pub fn 构建拆分序列(
        &self, 决策: &冰雪清韵决策, 拆分序列: &mut [[元素; 4]]
    ) {
        let 映射 = 决策.线性化(&self.棱镜);
        let mut 当前拆分索引 = vec![0_usize; self.动态拆分.len()];
        for (块序号, 拆分方式列表) in self.动态拆分.iter().enumerate() {
            if 块序号 == usize::MAX {
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
                let 块 = &self.数字转块[&块序号];
                let 拆分方式 = 拆分方式列表.last().unwrap().map(|x| {
                    if x == 0 {
                        "".to_string()
                    } else {
                        self.棱镜.数字转元素[&x].clone()
                    }
                });
                panic!("未找到 {块:?} 的映射: {拆分方式:?}\n当前决策为: {决策:?}",);
            }
        }
        for (序列, 固定拆分项) in zip(拆分序列, &self.固定拆分) {
            *序列 = [0; 4];
            let mut index = 0;
            for 块序号 in 固定拆分项.字块 {
                if 块序号 == usize::MAX {
                    break;
                }
                for 元素 in self.动态拆分[块序号][当前拆分索引[块序号]] {
                    if 元素 == 0 {
                        break;
                    }
                    序列[index] = 元素;
                    if index <= 2 {
                        index += 1;
                    }
                }
            }
            if 序列[1] == 0 {
                序列[1] = 序列[0] + 1;
            } else if 序列[2] == 0 {
                序列[2] = 序列[1] + 1;
            } else if 序列[3] == 0 {
                序列[3] = 序列[2] + 1;
            }
            if !序列.iter().all(|x| *x == 0 || 映射[*x] != 0) {
                panic!("拆分序列 {序列:?} 中存在未映射的元素，当前决策为: {决策:?}",);
            }
        }
    }

    pub fn 重置空间(&mut self) {
        self.简体空间.iter_mut().for_each(|x| {
            *x = false;
        });
        self.繁体空间.iter_mut().for_each(|x| {
            *x = false;
        });
        self.简繁通打空间.iter_mut().for_each(|x| {
            *x = false;
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
        编码结果: &mut [冰雪清韵编码信息],
        映射: &元素映射,
        拆分序列: &[[元素; 4]],
        决策: &冰雪清韵决策,
    ) {
        let mut index = 0;
        // 生成全码并统计简体选重标记
        for (序列, 编码信息) in zip(拆分序列, 编码结果.iter_mut()) {
            编码信息.全码 = Self::全码规则(序列, &映射);
            let 第一码 = 编码信息.全码 % 进制;
            let 第二码 = (编码信息.全码 / 进制) % 进制;
            assert!(
                第一码 != 0 && 第二码 != 0,
                "全码不能为零，当前序列: {:?}，当前序列: {:?}，映射后：{:?}，当前决策：{:?}",
                self.固定拆分[index],
                序列.map(|x| self.棱镜.数字转元素[&x].clone()),
                序列.map(|x| 映射[x]),
                决策
            );
            index += 1;
            // 在生成全码时，只对非字根字统计重码
            if 编码信息.简体 {
                if 序列[2] != 0 {
                    编码信息.简体选重 = self.简体空间[编码信息.全码 as usize];
                    self.简体空间[编码信息.全码 as usize] = true;
                } else {
                    编码信息.简体选重 = false;
                }
            }
        }
        // 繁体选重标记
        for 索引 in &self.繁体顺序 {
            let 编码信息 = &mut 编码结果[*索引];
            if 编码信息.全码 < 进制 * 进制 {
                编码信息.繁体选重 = false;
            } else {
                编码信息.繁体选重 = self.繁体空间[编码信息.全码 as usize];
                self.繁体空间[编码信息.全码 as usize] = true;
            }
        }
        // 简繁通打选重标记
        for 索引 in &self.简繁通打顺序 {
            let 编码信息 = &mut 编码结果[*索引];
            if 编码信息.全码 < 进制 * 进制 {
                编码信息.简繁通打选重 = false;
            } else {
                编码信息.简繁通打选重 = self.简繁通打空间[编码信息.全码 as usize];
                self.简繁通打空间[编码信息.全码 as usize] = true;
            }
        }
    }

    fn 输出优先简码(&mut self, 编码结果: &mut [冰雪清韵编码信息]) {
        // 输出优先字根
        for 序号 in self.优先简码.iter() {
            let 编码信息 = &mut 编码结果[*序号];
            // 特简码
            编码信息.简体简码 = 编码信息.全码;
            编码信息.完成出简 = true;
            if self.简体空间[编码信息.全码 as usize] {
                编码信息.简体选重 = true;
            }
            self.简体空间[编码信息.全码 as usize] = true;
        }
        const 字根字优先位置: usize = 1000;
        // 让前 1000 的字根占据两键字的位置
        for (序号, 编码信息) in 编码结果.iter_mut().enumerate() {
            if 序号 >= 字根字优先位置 {
                break;
            }
            // 特简码
            if 编码信息.特简 > 0 {
                编码信息.简体简码 = self.棱镜.键转数字[&特简码[编码信息.特简 as usize]];
                编码信息.完成出简 = true;
                continue;
            }
            // 二码字根字
            if 编码信息.全码 < 进制 * 进制 {
                if !self.简体空间[编码信息.全码 as usize] {
                    编码信息.简体简码 = 编码信息.全码;
                    编码信息.完成出简 = true;
                    self.简体空间[编码信息.全码 as usize] = true;
                }
                continue;
            }
        }
    }

    // fn 输出简码(&mut self, 编码结果: &mut [冰雪清韵编码信息], 决策: &冰雪清韵决策) {
    //     let 非空格小集合键: Vec<_> = 小集合
    //         .chars()
    //         .filter(|x| *x != '_')
    //         .map(|c| self.棱镜.键转数字[&c])
    //         .collect();
    //     let 不太好的组合 = ["p,", "p.", "p/", "y,", "y.", "y/", "ce", "nu", "mu"].map(|s| {
    //         let c1 = s.chars().next().unwrap();
    //         let c2 = s.chars().nth(1).unwrap();
    //         (self.棱镜.键转数字[&c1], self.棱镜.键转数字[&c2])
    //     });
    //     for (序号, 编码信息) in 编码结果.iter_mut().enumerate() {
    //         // 跳过已经处理的优先简码
    //         if 编码信息.完成出简 {
    //             编码信息.完成出简 = false;
    //             continue;
    //         }
    //         let 全码 = 编码信息.全码.原始编码;
    //         // 字根字的简码：包括 ab, ?ab, ??ab 三种情况
    //         if 编码信息.全码.原始编码候选位置 == u8::MAX {
    //             if self.编码空间[全码 as usize] == 0 {
    //                 // ab
    //                 编码信息.简码.原始编码 = 全码;
    //                 self.编码空间[全码 as usize] += 1;
    //             } else {
    //                 let 补一码 = self.棱镜.键转数字[&决策.补码键] + 全码 * 进制;
    //                 if self.编码空间[补一码 as usize] == 0 {
    //                     // ?ab
    //                     编码信息.简码.原始编码 = 补一码;
    //                     编码信息.全码.原始编码 = 补一码;
    //                     self.编码空间[补一码 as usize] += 1;
    //                 } else {
    //                     // ??ab
    //                     let 补二码 = self.棱镜.键转数字[&决策.补码键] + 补一码 * 进制;
    //                     编码信息.简码.原始编码 = 补二码;
    //                     编码信息.全码.原始编码 = 补二码;
    //                     if self.编码空间[补二码 as usize] != 0 {
    //                         编码信息.全码.选重标记 = true;
    //                     }
    //                     self.编码空间[补二码 as usize] += 1;
    //                 }
    //             }
    //             continue;
    //         }
    //         // 非字根字的简码：包括一级简码（空格）、一级简码（非空格）、二级简码、全码四种情况
    //         if 序号 >= 1500 && !self.全部出简 {
    //             编码信息.简码.原始编码 = 全码;
    //             continue;
    //         }
    //         // 一级简码（空格）
    //         let 空格一简 = 全码 % 进制 + 空格 * 进制;
    //         if self.编码空间[空格一简 as usize] == 0 {
    //             编码信息.简码.原始编码 = 空格一简;
    //             self.编码空间[空格一简 as usize] += 1;
    //             continue;
    //         }
    //         // 一级简码（非空格）
    //         let mut 普通一简 = 0;
    //         let mut 最小当量 = f64::MAX;
    //         let 第一码 = 全码 % 进制;
    //         for 键 in &非空格小集合键 {
    //             // 避免主动生成不太好的组合
    //             if 不太好的组合.contains(&(第一码, *键)) {
    //                 continue;
    //             }
    //             let 简码 = 第一码 + 键 * 进制;
    //             let 当量 = self.当量信息[简码 as usize];
    //             if self.编码空间[简码 as usize] == 0 && 当量 < 最小当量 {
    //                 普通一简 = 简码;
    //                 最小当量 = 当量;
    //             }
    //         }
    //         if 普通一简 > 0 {
    //             编码信息.简码.原始编码 = 普通一简;
    //             self.编码空间[普通一简 as usize] += 1;
    //             continue;
    //         }
    //         if 全码 > 进制 * 进制 * 进制 {
    //             // 二级简码
    //             let 空格二简 = 全码 % (进制 * 进制) + 空格 * 进制 * 进制;
    //             if self.编码空间[空格二简 as usize] == 0 {
    //                 编码信息.简码.原始编码 = 空格二简;
    //                 self.编码空间[空格二简 as usize] += 1;
    //                 continue;
    //             }
    //         }
    //         if self.全部出简 && 全码 > 进制 * 进制 * 进制 {
    //             // 三级简码
    //             let 空格三简 = 全码 % (进制 * 进制 * 进制) + 空格 * 进制 * 进制 * 进制;
    //             if self.编码空间[空格三简 as usize] == 0 {
    //                 编码信息.简码.原始编码 = 空格三简;
    //                 self.编码空间[空格三简 as usize] += 1;
    //                 continue;
    //             }
    //         }
    //         // 无简码
    //         编码信息.简码.原始编码 = 全码;
    //     }
    // }

    fn 输出简码2(
        &mut self, 编码结果: &mut [冰雪清韵编码信息], 决策: &冰雪清韵决策
    ) {
        let mut 字根字队列 = vec![];
        let 不太好的组合: FxHashSet<_> = vec!["p,", "p.", "p/", "y,", "y.", "y/", "ce", "nu", "mu"]
            .iter()
            .map(|s| {
                let c1 = s.chars().next().unwrap();
                let c2 = s.chars().nth(1).unwrap();
                self.棱镜.键转数字[&c1] + self.棱镜.键转数字[&c2] * 进制
            })
            .collect();
        let mut 子问题列表: Vec<_> = 大集合
            .map(|c| {
                let 第一码 = self.棱镜.键转数字[&c];
                let 一简十重: Vec<_> = 小集合
                    .map(|x| 第一码 + self.棱镜.键转数字[&x] * 进制)
                    .into_iter()
                    .filter(|一级简码| !不太好的组合.contains(一级简码))
                    .sorted_by(|&x, &y| {
                        self.当量信息[x as usize]
                            .partial_cmp(&self.当量信息[y as usize])
                            .unwrap()
                    })
                    .collect();
                let 二简列表: Vec<_> = 大集合
                    .map(|x| 第一码 + self.棱镜.键转数字[&x] * 进制 + 空格 * 进制 * 进制)
                    .into_iter()
                    .collect();
                return 出简子问题数据 {
                    三码全码队列: 队列 {
                        数据: [(0, 0.0); 最大备选长度],
                        当前索引: 0,
                        长度: 0,
                        二简: 0,
                    },
                    四码全码队列: from_fn(|x| 队列 {
                        数据: [(0, 0.0); 最大备选长度],
                        当前索引: 0,
                        长度: 0,
                        二简: 二简列表[x],
                    }),
                    一简十重,
                };
            })
            .into_iter()
            .collect();
        const 声码位移: usize = 1;
        for (序号, 编码信息) in 编码结果.iter_mut().enumerate() {
            if !编码信息.简体 {
                continue;
            }
            // 跳过已经处理的优先简码
            if 编码信息.完成出简 {
                编码信息.完成出简 = false;
                if 编码信息.简体简码 > 进制 {
                    let 第一码 = (编码信息.简体简码 % 进制) as usize - 声码位移;
                    子问题列表[第一码]
                        .一简十重
                        .retain(|&x| x != 编码信息.简体简码);
                }
                continue;
            } else if 编码信息.全码 < 进制 * 进制 {
                // 字根字
                字根字队列.push(序号);
            } else if 编码信息.全码 < 进制 * 进制 * 进制 {
                // 二根字
                let 第一码 = (编码信息.全码 % 进制) as usize - 声码位移;
                子问题列表[第一码]
                    .三码全码队列
                    .入队(序号, 编码信息.简体频率);
            } else if 编码信息.全码 < 进制 * 进制 * 进制 * 进制 {
                // 三根以上字
                let 第一码 = (编码信息.全码 % 进制) as usize - 声码位移;
                let 第二码 = ((编码信息.全码 / 进制) % 进制) as usize - 声码位移;
                子问题列表[第一码].四码全码队列[第二码].入队(序号, 编码信息.简体频率);
            }
            编码信息.简体简码 = 编码信息.全码;
        }
        for 子问题 in 子问题列表.iter_mut() {
            self.求解子问题(子问题, 编码结果);
        }
        // 最后处理字根字
        for 序号 in 字根字队列 {
            let 编码信息 = &mut 编码结果[序号];
            if self.固定拆分[序号].通规 == 0 {
                // 非通规字直接打 ??ab
                let 补二码 = self.棱镜.键转数字[&决策.补码键]
                    + self.棱镜.键转数字[&决策.补码键] * 进制
                    + 编码信息.全码 * 进制 * 进制;
                编码信息.简体简码 = 补二码;
                编码信息.全码 = 补二码;
                if self.简体空间[补二码 as usize] {
                    编码信息.简体选重 = true;
                }
                self.简体空间[补二码 as usize] = true;
            } else {
                // 通规字可以是 ab 或 ?ab
                if !self.简体空间[编码信息.全码 as usize] {
                    // ab
                    编码信息.简体简码 = 编码信息.全码;
                    self.简体空间[编码信息.全码 as usize] = true;
                } else {
                    // ?ab
                    let 补一码 = self.棱镜.键转数字[&决策.补码键] + 编码信息.全码 * 进制;
                    编码信息.简体简码 = 补一码;
                    编码信息.全码 = 补一码;
                    if self.简体空间[补一码 as usize] {
                        编码信息.简体选重 = true;
                    }
                    self.简体空间[补一码 as usize] = true;
                }
            }
        }
    }

    fn 求解子问题(
        &mut self, 子问题: &mut 出简子问题数据, 编码结果: &mut [冰雪清韵编码信息]
    ) {
        let mut 总队列 = BinaryHeap::from(子问题.四码全码队列);
        总队列.push(子问题.三码全码队列);
        while !子问题.一简十重.is_empty() {
            let 一级简码 = 子问题.一简十重.remove(0);
            let mut 队列 = 总队列.pop().unwrap();
            let (序号, _) = 队列.出队();
            编码结果[序号].简体简码 = 一级简码;
            self.简体空间[一级简码 as usize] = true;
            总队列.push(队列);
        }
        // 输出二级简码
        for 队列 in 总队列 {
            if 队列.二简 != 0 {
                let (序号, _) = 队列.数据[队列.当前索引];
                if 序号 != 0 {
                    编码结果[序号].简体简码 = 队列.二简;
                    self.简体空间[队列.二简 as usize] = true;
                }
            }
        }
    }

    // fn 输出简码3(
    //     &mut self, 编码结果: &mut [编码信息], 映射: &元素映射, 决策: &冰雪清韵决策
    // ) {
    //     let 非空格小集合键: Vec<_> = 小集合
    //         .chars()
    //         .filter(|x| *x != '_')
    //         .map(|c| self.棱镜.键转数字[&c])
    //         .collect();
    //     let 不太好的组合 = ["p,", "p.", "p/", "y,", "y.", "y/", "ce", "nu", "mu"].map(|s| {
    //         let c1 = s.chars().next().unwrap();
    //         let c2 = s.chars().nth(1).unwrap();
    //         (self.棱镜.键转数字[&c1], self.棱镜.键转数字[&c2])
    //     });
    //     for (序号, 编码信息) in 编码结果.iter_mut().enumerate() {
    //         // 跳过已经处理的优先简码
    //         if 编码信息.简码.有变化 {
    //             编码信息.简码.有变化 = false;
    //             continue;
    //         }
    //         let 全码 = 编码信息.全码.原始编码;
    //         // 字根字的简码：包括 ab, ?ab, ??ab 三种情况
    //         if 编码信息.全码.原始编码候选位置 == u8::MAX {
    //             if self.编码空间[全码 as usize] == 0 {
    //                 // ab
    //                 编码信息.简码.原始编码 = 全码;
    //                 self.编码空间[全码 as usize] += 1;
    //             } else {
    //                 let 补一码 = self.棱镜.键转数字[&决策.补码键] + 全码 * 进制;
    //                 if self.编码空间[补一码 as usize] == 0 {
    //                     // ?ab
    //                     编码信息.简码.原始编码 = 补一码;
    //                     编码信息.全码.原始编码 = 补一码;
    //                     self.编码空间[补一码 as usize] += 1;
    //                 } else {
    //                     // ??ab
    //                     let 补二码 = self.棱镜.键转数字[&决策.补码键] + 补一码 * 进制;
    //                     编码信息.简码.原始编码 = 补二码;
    //                     编码信息.全码.原始编码 = 补二码;
    //                     if self.编码空间[补二码 as usize] != 0 {
    //                         编码信息.全码.选重标记 = true;
    //                     }
    //                     self.编码空间[补二码 as usize] += 1;
    //                 }
    //             }
    //             continue;
    //         }
    //         // 非字根字的简码：包括一级简码（空格）、一级简码（非空格）、二级简码、全码四种情况
    //         if 序号 >= 1500 && !self.全部出简 {
    //             编码信息.简码.原始编码 = 全码;
    //             continue;
    //         }
    //         // 拼音码
    //         let (声, 韵) = self.固定拆分[序号].声韵;
    //         let (声键, 韵键) = (映射[声 as usize], 映射[韵 as usize]);
    //         // 避免主动生成不太好的组合
    //         if !不太好的组合.contains(&(声 as u64, 韵 as u64)) {
    //             let 拼音码 = 声键 + 韵键 * 进制;
    //             if self.编码空间[拼音码 as usize] == 0 {
    //                 编码信息.简码.原始编码 = 拼音码;
    //                 self.编码空间[拼音码 as usize] += 1;
    //                 continue;
    //             }
    //         }
    //         // 一级简码（空格）
    //         let 空格一简 = 全码 % 进制 + 空格 * 进制;
    //         if self.编码空间[空格一简 as usize] == 0 {
    //             编码信息.简码.原始编码 = 空格一简;
    //             self.编码空间[空格一简 as usize] += 1;
    //             continue;
    //         }
    //         if 全码 > 进制 * 进制 * 进制 {
    //             // 二级简码
    //             let 空格二简 = 全码 % (进制 * 进制) + 空格 * 进制 * 进制;
    //             if self.编码空间[空格二简 as usize] == 0 {
    //                 编码信息.简码.原始编码 = 空格二简;
    //                 self.编码空间[空格二简 as usize] += 1;
    //                 continue;
    //             }
    //         }
    //         if self.全部出简 && 全码 > 进制 * 进制 * 进制 {
    //             // 三级简码
    //             let 空格三简 = 全码 % (进制 * 进制 * 进制) + 空格 * 进制 * 进制 * 进制;
    //             if self.编码空间[空格三简 as usize] == 0 {
    //                 编码信息.简码.原始编码 = 空格三简;
    //                 self.编码空间[空格三简 as usize] += 1;
    //                 continue;
    //             }
    //         }
    //         // 无简码
    //         编码信息.简码.原始编码 = 全码;
    //     }
    // }

    pub fn 动态编码(
        &mut self,
        决策: &冰雪清韵决策,
        拆分序列: &[[元素; 4]],
        输出: &mut [冰雪清韵编码信息],
    ) {
        let 映射 = 决策.线性化(&self.棱镜);
        time_block!("重置空间", { self.重置空间() });
        time_block!("输出全码", {
            self.输出全码(输出, &映射, 拆分序列, 决策)
        });
        time_block!("输出优先简码", { self.输出优先简码(输出) });
        time_block!("输出简码", { self.输出简码2(输出, 决策) });
    }
}

impl 编码器 for 冰雪清韵编码器 {
    type 解类型 = 冰雪清韵决策;
    fn 编码(
        &mut self,
        _决策: &冰雪清韵决策,
        _决策变化: &Option<冰雪清韵决策变化>,
        _输出: &mut [编码信息],
    ) {
        self.重置空间();
    }
}
