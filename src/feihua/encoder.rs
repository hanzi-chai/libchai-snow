use crate::{
    common::转换,
    feihua::{
        冰雪飞花上下文, 冰雪飞花决策, 冰雪飞花可编码对象, 小集合, 线性化决策, 编码
    },
};
use chai::{contexts::default::默认决策变化, encoders::编码器, 位图, 棱镜, 编码信息};
use std::iter::zip;

pub struct 冰雪飞花编码信息 {
    pub 全码: 编码,
    pub 简码: 编码,
    pub 频率: u64,
    pub 指数频率: f64,
    pub 选重: bool,
    pub 候选位置: u8,
}

pub struct 冰雪飞花编码器 {
    pub 词信息: Vec<冰雪飞花可编码对象>,
    pub 棱镜: 棱镜,
    pub 编码结果: Vec<冰雪飞花编码信息>,
    pub 编码空间: Vec<u8>,
}

impl 冰雪飞花编码器 {
    pub fn 新建(上下文: &冰雪飞花上下文) -> Self {
        let 编码结果 = 上下文
            .词列表
            .iter()
            .enumerate()
            .map(|(索引, 词)| 冰雪飞花编码信息 {
                全码: 编码::default(),
                简码: 编码::default(),
                频率: 词.频率,
                指数频率: ((索引.min(6000) as f64) / -2000.0).exp(),
                选重: false,
                候选位置: 0,
            })
            .collect();
        Self {
            词信息: 上下文.词列表.clone(),
            棱镜: 上下文.棱镜.clone(),
            编码空间: vec![0; 编码::编码空间大小()],
            编码结果,
        }
    }
}

impl 冰雪飞花编码器 {
    fn 刷新元素序列表(&mut self, 映射: &线性化决策) {
        let mut 全集合位图 = 位图::new();
        let mut 小集合位图 = 位图::new();
        for (元素, 键位) in 映射.iter().enumerate() {
            if *键位 != 0 {
                全集合位图.insert(元素);
                if *键位 <= 小集合.len() as u8 {
                    小集合位图.insert(元素);
                }
            }
        }
        for 词信息 in &mut self.词信息 {
            词信息.元素序列 = 词信息
                .全部元素序列
                .iter()
                .find(|(_, 全, 小)| 全.subset(&全集合位图) && 小.subset(&小集合位图))
                .unwrap()
                .0;
        }
    }

    pub fn 重置空间(&mut self) {
        self.编码空间.iter_mut().for_each(|x| {
            *x = 0;
        });
    }

    pub fn 生成全码(&mut self, 映射: &线性化决策) {
        for (输出, 可编码对象) in zip(&mut self.编码结果, &self.词信息) {
            let a = &可编码对象.元素序列;
            输出.全码 = 编码([映射[a[0]], 映射[a[1]], 映射[a[2]], 映射[a[3]]]);
            let hash = 输出.全码.hash();
            if hash >= self.编码空间.len() {
                panic!("词「{}」编码：{:?} 超出编码空间范围，当前序列：{:?}", 可编码对象.词, 输出.全码, a);
            }
            输出.选重 = self.编码空间[hash] > 0;
            输出.候选位置 = self.编码空间[hash];
            self.编码空间[hash] += 1;
        }
    }
}

impl 编码器 for 冰雪飞花编码器 {
    type 决策 = 冰雪飞花决策;

    fn 编码(
        &mut self, 决策: &冰雪飞花决策, _变化: &Option<默认决策变化>, _: &mut [编码信息]
    ) {
        self.重置空间();
        let 线性化决策 = 决策.线性化(&self.棱镜);
        self.刷新元素序列表(&线性化决策);
        self.生成全码(&线性化决策);
    }
}
