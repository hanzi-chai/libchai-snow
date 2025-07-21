use chai::{encoders::编码器, 元素, 元素映射, 棱镜, 编码信息, 错误};
use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde_yaml::from_str;
use std::{collections::HashMap, fs::read_to_string, iter::zip};

use crate::qingyun::{
    冰雪清韵上下文, 冰雪清韵决策, 冰雪清韵决策变化, 字根安排, 小集合, 最大码长, 空格, 进制,
};

type 块 = usize;
type 固定拆分 = Vec<[块; 4]>;
type 动态拆分 = Vec<Vec<[元素; 4]>>;

pub struct 冰雪清韵编码器 {
    pub 固定拆分: 固定拆分,
    pub 动态拆分: 动态拆分,
    pub 块转数字: FxHashMap<String, usize>,
    pub 数字转块: FxHashMap<usize, String>,
    pub 字根首笔: FxHashMap<元素, 元素>,
    pub 编码空间: Vec<u8>,
    pub 棱镜: 棱镜,
    pub 当量信息: Vec<f64>,
}

type 原始固定拆分 = HashMap<String, Vec<String>>;
type 原始动态拆分 = HashMap<String, Vec<Vec<String>>>;

#[derive(Deserialize)]
struct 拆分输入 {
    固定拆分: 原始固定拆分,
    动态拆分: 原始动态拆分,
    字根笔画: FxHashMap<String, Vec<usize>>,
}

impl 冰雪清韵编码器 {
    pub fn 新建(上下文: &冰雪清韵上下文) -> Result<Self, 错误> {
        let 当量信息 = 上下文
            .棱镜
            .预处理当量信息(&上下文.原始当量信息, 进制.pow(最大码长 as u32) as usize);
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
            // 检查原始拆分方式列表的最后一项都是必选字根
            let 最后一项 = 原始拆分方式列表.last().unwrap();
            if !最后一项
                .iter()
                .all(|x| !上下文.决策空间.字根[x].contains(&字根安排::未选取))
            {
                panic!("动态拆分方式列表的最后一项必须都是必选字根, {块:?}, {原始拆分方式列表:?}");
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
        let mut 字根首笔 = FxHashMap::default();
        for (字根, 笔画) in 拆分输入.字根笔画 {
            字根首笔.insert(
                上下文.棱镜.元素转数字[&字根],
                上下文.棱镜.元素转数字[&笔画.iter().next().unwrap().to_string()],
            );
        }
        Ok(Self {
            动态拆分,
            固定拆分,
            块转数字,
            数字转块,
            字根首笔,
            编码空间: 全码空间,
            棱镜: 上下文.棱镜.clone(),
            当量信息,
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

    pub fn 构建拆分序列(
        &self, 决策: &冰雪清韵决策, 拆分序列: &mut [[元素; 4]]
    ) {
        let 映射 = 决策.线性化(&self.棱镜);
        let mut 当前拆分索引 = vec![0_usize; self.动态拆分.len()];
        for (块序号, 拆分方式列表) in self.动态拆分.iter().enumerate() {
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
        for (序列, 固定拆分内容) in zip(拆分序列, &self.固定拆分) {
            *序列 = [0, 0, 0, 0];
            let mut index = 0;
            for 块序号 in *固定拆分内容 {
                if 块序号 == 0 {
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
                序列[1] = 序列[0];
                序列[2] = 序列[0] + 1;
                序列[0] = self.字根首笔[&序列[0]]
            } else if 序列[2] == 0 {
                序列[2] = 序列[1] + 1;
            } else if 序列[3] == 0 {
                序列[3] = 序列[2] + 1;
            }
        }
    }

    pub fn 重置空间(&mut self) {
        self.编码空间.iter_mut().for_each(|x| {
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
        拆分序列: &[[元素; 4]],
    ) {
        let 映射 = 决策.线性化(&self.棱镜);
        for (序列, 编码信息) in zip(拆分序列, 编码结果.iter_mut()) {
            let 全码信息 = &mut 编码信息.全码;
            全码信息.原始编码 = Self::全码规则(序列, &映射);
            全码信息.原始编码候选位置 = self.编码空间[全码信息.原始编码 as usize];
            self.编码空间[全码信息.原始编码 as usize] += 1;
            全码信息.选重标记 = 全码信息.原始编码候选位置 > 0;
        }
    }

    fn 输出简码(&mut self, 编码结果: &mut [编码信息]) {
        let 特简码 = [' ', 'e', 'i', 'o', 'u', 'a', ';', '/'];
        let 非空格小集合键: Vec<_> = 小集合
            .chars()
            .filter(|x| *x != '_')
            .map(|c| self.棱镜.键转数字[&c])
            .collect();
        for (序号, 编码信息) in 编码结果.iter_mut().enumerate() {
            let 全码 = 编码信息.全码.原始编码;
            if 序号 >= 1500 {
                编码信息.简码.原始编码 = 全码;
                continue;
            }
            // 零级简码
            if 编码信息.词长 > 0 {
                编码信息.简码.原始编码 = self.棱镜.键转数字[&特简码[编码信息.词长 as usize]];
                continue;
            }
            // 一级简码（空格）
            let 空格一简 = 全码 % 进制 + 空格 * 进制;
            if self.编码空间[空格一简 as usize] == 0 {
                编码信息.简码.原始编码 = 空格一简;
                self.编码空间[空格一简 as usize] += 1;
                continue;
            }
            // 一级简码（非空格）
            let mut 普通一简 = 0;
            let mut 最小当量 = f64::MAX;
            for 键 in &非空格小集合键 {
                let 简码 = 全码 % 进制 + 键 * 进制;
                let 当量 = self.当量信息[简码 as usize];
                if self.编码空间[简码 as usize] == 0 && 当量 < 最小当量 {
                    普通一简 = 简码;
                    最小当量 = 当量;
                }
            }
            if 普通一简 > 0 {
                编码信息.简码.原始编码 = 普通一简;
                self.编码空间[普通一简 as usize] += 1;
                continue;
            }
            // 二级简码
            let 空格二简 = 全码 % (进制 * 进制) + 空格 * 进制 * 进制;
            if self.编码空间[空格二简 as usize] == 0 {
                编码信息.简码.原始编码 = 空格二简;
                self.编码空间[空格二简 as usize] += 1;
                continue;
            }
            // 无简码
            编码信息.简码.原始编码 = 全码;
        }
    }

    pub fn 动态编码(
        &mut self,
        决策: &冰雪清韵决策,
        拆分序列: &[[元素; 4]],
        输出: &mut [编码信息],
    ) {
        self.重置空间();
        self.输出全码(输出, 决策, 拆分序列);
        self.输出简码(输出);
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
