use crate::dual::构建双编码映射;
use chai::data::{元素, 元素映射, 可编码对象, 数据, 编码信息, 键};
use chai::encoders::编码器;
use chai::错误;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::iter::zip;

pub const 双编码占位符: 键 = 31;

pub struct 冰雪双拼编码器 {
    pub 进制: u64,
    pub 编码结果: Vec<编码信息>,
    pub 词列表: Vec<可编码对象>,
    pub 全码空间: Vec<u8>,
    pub 简码空间: Vec<u8>,
    pub 包含元素的词映射: Vec<Vec<usize>>,
    pub 双编码映射: HashMap<元素, (元素, 元素)>,
    pub 数字转元素: FxHashMap<元素, String>,
}

impl 冰雪双拼编码器 {
    pub fn 新建(数据: &数据) -> Result<Self, 错误> {
        let 最大码长 = 4;
        let 词列表 = 数据.词列表.clone();
        let 编码输出 = 词列表.iter().map(编码信息::new).collect();
        let 编码空间大小 = 数据.进制.pow(最大码长 as u32) as usize;
        let 全码空间 = vec![u8::default(); 编码空间大小];
        let 简码空间 = 全码空间.clone();
        let mut 包含元素的词映射 = vec![vec![]; 数据.初始映射.len()];
        for (索引, 词) in 词列表.iter().enumerate() {
            for 元素 in &词.元素序列 {
                包含元素的词映射[*元素].push(索引);
            }
        }
        let 双编码映射 = 构建双编码映射(数据);
        let 编码器 = Self {
            进制: 数据.进制,
            编码结果: 编码输出,
            词列表,
            全码空间,
            简码空间,
            包含元素的词映射,
            双编码映射,
            数字转元素: 数据.数字转元素.clone(),
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
        let 音码 = 映射[元素序列[0]] + 映射[元素序列[1]] * 进制;
        let 字根一 = 元素序列[2];
        let mut 字根二 = *元素序列.get(3).unwrap_or(&0);
        if 映射[字根二] == 双编码占位符 {
            字根二 = 双编码映射[&字根二].0;
        }
        let 形码 = if 映射[字根一] == 双编码占位符 {
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
    }

    fn 输出全码(&mut self, 映射: &元素映射, 移动的元素: &Option<Vec<元素>>) {
        let 编码结果 = &mut self.编码结果;
        let 进制 = self.进制;
        if let Some(移动的元素) = 移动的元素 {
            for 元素 in 移动的元素 {
                for 索引 in &self.包含元素的词映射[*元素] {
                    let 词 = &self.词列表[*索引];
                    let 全码 =
                        冰雪双拼编码器::全码规则(词, 映射, 进制, &self.双编码映射);
                    编码结果[*索引].全码.写入编码(全码);
                }
            }
        } else {
            for (词, 编码信息) in zip(&self.词列表, 编码结果.iter_mut()) {
                let 全码 = 冰雪双拼编码器::全码规则(词, 映射, 进制, &self.双编码映射);
                编码信息.全码.写入编码(全码);
            }
        }

        for 编码信息 in 编码结果.iter_mut() {
            let 全码信息 = &mut 编码信息.全码;
            let 是否重码 = self.全码空间[全码信息.实际编码 as usize] > 0;
            全码信息.写入选重(是否重码);
            self.全码空间[全码信息.实际编码 as usize] += 1;
        }
    }

    fn 输出简码(&mut self) {
        let 编码结果 = &mut self.编码结果;
        let 进制 = self.进制;
        for (编码信息, 词) in zip(编码结果.iter_mut(), &self.词列表) {
            let 全码 = &编码信息.全码.实际编码;
            let 简码信息 = &mut 编码信息.简码;
            if 词.词长 == 1 {
                let 一简 = 全码 % 进制;
                let 重数 = self.全码空间[一简 as usize] + self.简码空间[一简 as usize];
                if 重数 == 0 {
                    简码信息.写入(一简, false);
                    self.简码空间[一简 as usize] += 1;
                    continue;
                }
                let 二简 = 全码 % (进制 * 进制);
                let 重数 = self.全码空间[二简 as usize] + self.简码空间[二简 as usize];
                if 重数 == 0 {
                    简码信息.写入(二简, false);
                    self.简码空间[二简 as usize] += 1;
                    continue;
                }
            }
            let 全码是否重码 = self.简码空间[*全码 as usize] > 0;
            简码信息.写入(*全码, 全码是否重码);
            self.简码空间[*全码 as usize] += 1;
        }
    }
}

impl 编码器 for 冰雪双拼编码器 {
    fn 编码(
        &mut self,
        映射: &元素映射,
        移动的元素: &Option<Vec<元素>>,
    ) -> &mut Vec<编码信息> {
        self.重置空间();
        self.输出全码(映射, 移动的元素);
        self.输出简码();
        // 如果需要输出实际的编码信息，取消注释以下代码
        // for 编码信息 in &mut self.编码结果 {
        //     编码信息.全码.原始编码 = 编码信息.全码.实际编码;
        //     编码信息.全码.原始编码候选位置 = 编码信息.全码.选重标记 as u8;
        //     编码信息.简码.原始编码 = 编码信息.简码.实际编码;
        //     编码信息.简码.原始编码候选位置 = 编码信息.简码.选重标记 as u8;
        // }
        &mut self.编码结果
    }
}
