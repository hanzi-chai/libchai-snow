use super::冰雪二拼元素分类;
use crate::common::dual::构建双编码映射;
use chai::contexts::default::默认上下文;
use chai::encoders::编码器;
use chai::错误;
use chai::{元素, 元素映射, 可编码对象, 编码信息, 键};
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::iter::zip;

pub const 空格: 键 = 31;

pub struct 冰雪二拼编码器 {
    pub 进制: u64,
    pub 词列表: Vec<可编码对象>,
    pub 全码空间: Vec<u8>,
    pub 简码空间: Vec<u8>,
    pub 包含元素的词映射: Vec<Vec<usize>>,
    pub 双编码映射: HashMap<元素, (元素, 元素)>,
    pub 数字转元素: FxHashMap<元素, String>,
    pub 元素分类: 冰雪二拼元素分类,
    pub 一字数量: usize,
}

impl 冰雪二拼编码器 {
    pub fn 新建(数据: &默认上下文) -> Result<Self, 错误> {
        let 最大码长 = 4;
        let 词列表 = 数据.词列表.clone();
        let 编码空间大小 = 数据.棱镜.进制.pow(最大码长 as u32) as usize;
        let 全码空间 = vec![u8::default(); 编码空间大小];
        let 简码空间 = 全码空间.clone();
        let mut 包含元素的词映射 = vec![vec![]; 数据.初始映射.len()];
        for (索引, 词) in 词列表.iter().enumerate() {
            for 元素 in &词.元素序列 {
                包含元素的词映射[*元素].push(索引);
            }
        }
        let 一字数量 = 词列表
            .iter()
            .position(|x| x.词长 > 1)
            .unwrap_or(词列表.len());
        let 双编码映射 = 构建双编码映射(数据);
        let 编码器 = Self {
            进制: 数据.棱镜.进制,
            词列表,
            全码空间,
            简码空间,
            包含元素的词映射,
            双编码映射,
            数字转元素: 数据.棱镜.数字转元素.clone(),
            元素分类: 冰雪二拼元素分类::新建(数据),
            一字数量,
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
    fn 全码规则(
        词: &可编码对象,
        映射: &元素映射,
        进制: u64,
        双编码映射: &HashMap<元素, (元素, 元素)>,
    ) -> u64 {
        let 元素序列 = &词.元素序列;
        if 词.词长 == 1 {
            let 音码 = 映射[元素序列[0]] + 映射[元素序列[1]] * 进制;
            let 字根一 = 元素序列[2];
            let mut 字根二 = *元素序列.get(3).unwrap_or(&0);
            if 映射[字根二] == 空格 {
                字根二 = 双编码映射[&字根二].0;
            }
            let 形码 = if 映射[字根一] == 空格 {
                let (子字根一, 子字根二) = 双编码映射[&字根一];
                if 元素序列.len() >= 4 && 元素序列[4] == 2 {
                    映射[子字根一] + 映射[字根二] * 进制
                } else {
                    映射[子字根一] + 映射[子字根二] * 进制
                }
            } else {
                let mut tmp = 映射[字根一];
                if 元素序列.len() >= 4 {
                    tmp += 映射[字根二] * 进制;
                }
                tmp
            };
            音码 + 形码 * (进制 * 进制)
        } else {
            映射[元素序列[0]]
                + 映射[元素序列[1]] * 进制
                + 映射[元素序列[2]] * (进制 * 进制)
                + if 元素序列.len() == 4 {
                    映射[元素序列[3]] * (进制 * 进制 * 进制)
                } else {
                    0
                }
        }
    }

    fn 输出全码(
        &mut self,
        编码结果: &mut [编码信息],
        映射: &元素映射,
        移动的元素: &Option<Vec<元素>>,
        仅形码改变: bool,
    ) {
        let 进制 = self.进制;
        if let Some(移动的元素) = 移动的元素 {
            for 元素 in 移动的元素 {
                for 索引 in &self.包含元素的词映射[*元素] {
                    let 词 = &self.词列表[*索引];
                    编码结果[*索引].全码.原始编码 =
                        冰雪二拼编码器::全码规则(词, 映射, 进制, &self.双编码映射);
                }
            }
        } else {
            for (词, 编码信息) in zip(&self.词列表, 编码结果.iter_mut()) {
                编码信息.全码.原始编码 =
                    冰雪二拼编码器::全码规则(词, 映射, 进制, &self.双编码映射);
            }
        }

        for (索引, 编码信息) in 编码结果.iter_mut().enumerate() {
            let 全码信息 = &mut 编码信息.全码;
            全码信息.原始编码候选位置 = self.全码空间[全码信息.原始编码 as usize];
            self.全码空间[全码信息.原始编码 as usize] += 1;
            全码信息.更新(全码信息.原始编码, 全码信息.原始编码候选位置 > 0);
            if 仅形码改变 && 索引 == self.一字数量 {
                break;
            }
        }
    }

    fn 输出简码(&mut self, 编码结果: &mut [编码信息]) {
        let 进制 = self.进制;
        let mut 索引 = 0;
        for 编码信息 in 编码结果.iter_mut() {
            let 全码原始 = 编码信息.全码.原始编码;
            let 简码信息 = &mut 编码信息.简码;
            let 一简原始 = 全码原始 % 进制;
            let 重数 = self.全码空间[一简原始 as usize] + self.简码空间[一简原始 as usize];
            if 重数 == 0 {
                // 简码信息.原始编码 = 一简原始;
                // 简码信息.原始编码候选位置 = self.简码空间[一简原始 as usize];
                self.简码空间[一简原始 as usize] += 1;
                let 一简实际 = 一简原始 + 空格 * 进制;
                简码信息.更新(一简实际, false);
                continue;
            }
            let 二简原始 = 全码原始 % (进制 * 进制);
            let 重数 = self.全码空间[二简原始 as usize] + self.简码空间[二简原始 as usize];
            if 重数 == 0 {
                // 简码信息.原始编码 = 二简原始;
                // 简码信息.原始编码候选位置 = self.简码空间[二简原始 as usize];
                self.简码空间[二简原始 as usize] += 1;
                let 二简实际 = 二简原始;
                简码信息.更新(二简实际, false);
                continue;
            }
            // 多字词以及没有简码的一字词
            let 全码是否重码 = self.简码空间[全码原始 as usize] > 0;
            简码信息.原始编码 = 全码原始;
            简码信息.原始编码候选位置 = self.简码空间[全码原始 as usize];
            self.简码空间[全码原始 as usize] += 1;
            简码信息.更新(全码原始, 全码是否重码);
            索引 += 1;
            if 索引 == self.一字数量 {
                break;
            }
        }
    }
}

impl 编码器 for 冰雪二拼编码器 {
    type 解类型 = 元素映射;
    fn 编码(
        &mut self, 映射: &元素映射, 移动的元素: &Option<Vec<元素>>, 输出: &mut [编码信息]
    ) {
        self.重置空间();
        let 仅形码改变 = if let Some(移动的元素) = 移动的元素 {
            移动的元素
                .iter()
                .all(|x| self.元素分类.字根列表.contains(x))
        } else {
            false
        };
        self.输出全码(输出, 映射, 移动的元素, 仅形码改变);
        self.输出简码(输出);
    }
}
