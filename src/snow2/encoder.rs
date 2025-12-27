use crate::common::转换;
use crate::snow2::{
    冰雪二拼上下文, 冰雪二拼信息, 冰雪二拼决策, 线性化决策, 编码
};
use chai::encoders::编码器;
use chai::{元素, 编码信息};
use chai::{棱镜, 错误};
use rustc_hash::FxHashMap;
use std::iter::zip;

#[derive(Default, Clone, Debug)]
pub struct 冰雪二拼编码信息 {
    pub 全码: 编码,
    pub 频率: u64,
    pub 选重: bool,
}

pub struct 冰雪二拼编码器 {
    pub 进制: u64,
    pub 词列表: Vec<冰雪二拼信息>,
    pub 全码空间: Vec<u8>,
    pub 简码空间: Vec<u8>,
    pub 棱镜: 棱镜,
    pub 韵母声调映射: FxHashMap<元素, (String, usize)>,
    pub 编码结果: Vec<冰雪二拼编码信息>,
}

impl 冰雪二拼编码器 {
    pub fn 新建(上下文: &冰雪二拼上下文) -> Result<Self, 错误> {
        let 词列表 = 上下文.信息列表.clone();
        let 编码空间大小 = 编码::编码空间大小();
        let 全码空间 = vec![Default::default(); 编码空间大小];
        let 简码空间 = 全码空间.clone();
        let 编码器 = Self {
            进制: 上下文.棱镜.进制,
            词列表,
            全码空间,
            简码空间,
            棱镜: 上下文.棱镜.clone(),
            韵母声调映射: 上下文.韵母声调映射.clone(),
            编码结果: 上下文
                .信息列表
                .iter()
                .map(|词| 冰雪二拼编码信息 {
                    全码: 编码::default(),
                    频率: 词.频率,
                    选重: false,
                })
                .collect(),
        };
        Ok(编码器)
    }

    pub fn 重置空间(&mut self) {
        self.全码空间.iter_mut().for_each(|x| {
            *x = 0;
        });
        self.简码空间.iter_mut().for_each(|x| {
            *x = 0;
        });
    }

    #[inline(always)]
    fn 全码规则(词: &冰雪二拼信息, 映射: &线性化决策) -> 编码 {
        let 序列 = &词.序列;
        let mut 全码 = [映射[序列[0]].0, 映射[序列[1]].0, 0, 0, 0];
        let (字根一, 字根二) = (序列[2], 序列[3]);
        if 字根二 == 0 {
            if 词.独立一 {
                (全码[2], 全码[3]) = 映射[字根一];
            } else {
                全码[2] = 映射[字根一].0;
            }
        } else {
            let 编码一 = 映射[字根一];
            let 编码二 = 映射[字根二];
            if 词.独立一 && 编码一.1 != 0 {
                (全码[2], 全码[3]) = 编码一;
                全码[4] = 编码二.0;
            } else if 词.独立二 && 编码二.1 != 0 {
                全码[2] = 编码一.0;
                (全码[3], 全码[4]) = 编码二;
            } else {
                全码[2] = 编码一.0;
                全码[3] = 编码二.0;
            }
        }
        全码
    }

    fn 输出全码(&mut self, 映射: &线性化决策) {
        for (词, 编码信息) in zip(&self.词列表, self.编码结果.iter_mut()) {
            编码信息.全码 = 冰雪二拼编码器::全码规则(词, 映射);
            let hash = 编码信息.全码.hash();
            编码信息.选重 = self.全码空间[hash] > 0;
            self.全码空间[hash] += 1;
        }
    }

    // fn 输出简码(&mut self, 编码结果: &mut [编码信息]) {
    //     let 进制 = self.进制;
    //     let mut 索引 = 0;
    //     for 编码信息 in 编码结果.iter_mut() {
    //         let 全码原始 = 编码信息.全码.原始编码;
    //         let 简码信息 = &mut 编码信息.简码;
    //         let 一简原始 = 全码原始 % 进制;
    //         let 重数 = self.全码空间[一简原始 as usize] + self.简码空间[一简原始 as usize];
    //         if 重数 == 0 {
    //             // 简码信息.原始编码 = 一简原始;
    //             // 简码信息.原始编码候选位置 = self.简码空间[一简原始 as usize];
    //             self.简码空间[一简原始 as usize] += 1;
    //             let 一简实际 = 一简原始 + 空格 * 进制;
    //             简码信息.更新(一简实际, false);
    //             continue;
    //         }
    //         let 二简原始 = 全码原始 % (进制 * 进制);
    //         let 重数 = self.全码空间[二简原始 as usize] + self.简码空间[二简原始 as usize];
    //         if 重数 == 0 {
    //             // 简码信息.原始编码 = 二简原始;
    //             // 简码信息.原始编码候选位置 = self.简码空间[二简原始 as usize];
    //             self.简码空间[二简原始 as usize] += 1;
    //             let 二简实际 = 二简原始;
    //             简码信息.更新(二简实际, false);
    //             continue;
    //         }
    //         // 多字词以及没有简码的一字词
    //         let 全码是否重码 = self.简码空间[全码原始 as usize] > 0;
    //         简码信息.原始编码 = 全码原始;
    //         简码信息.原始编码候选位置 = self.简码空间[全码原始 as usize];
    //         self.简码空间[全码原始 as usize] += 1;
    //         简码信息.更新(全码原始, 全码是否重码);
    //         索引 += 1;
    //         if 索引 == self.一字数量 {
    //             break;
    //         }
    //     }
    // }
}

impl 编码器 for 冰雪二拼编码器 {
    type 决策 = 冰雪二拼决策;

    fn 编码(
        &mut self, 决策: &Self::决策, _变化: &Option<()>, _输出: &mut [编码信息]
    ) {
        self.重置空间();
        let 映射 = 决策.线性化(&self.棱镜, &self.韵母声调映射);
        self.输出全码(&映射);
        // self.输出简码();
    }
}
