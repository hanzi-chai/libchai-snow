use crate::qingyun::{
    encoder::冰雪清韵编码器, 冰雪清韵决策, 冰雪清韵决策变化
};
use chai::{encoders::编码器, objectives::目标函数, 编码信息, 部分编码信息};
use serde::Serialize;
use std::fmt::Display;

#[derive(Debug, Clone, Serialize)]
pub struct 冰雪清韵指标 {
    pub 重码数: usize,
}

impl Display for 冰雪清韵指标 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "重码数: {}\n", self.重码数)
    }
}

pub struct 冰雪清韵目标函数 {
    pub 编码器: 冰雪清韵编码器,
    pub 编码结果: Vec<编码信息>,
    pub 编码结果缓冲: Vec<编码信息>,
}

impl 冰雪清韵目标函数 {
    pub fn 新建(编码器: 冰雪清韵编码器) -> Self {
        let 编码结果: Vec<_> = 编码器
            .拆分序列
            .iter()
            .map(|_| 编码信息 {
                词长: 1,
                频率: 1,
                全码: 部分编码信息::default(),
                简码: 部分编码信息::default(),
            })
            .collect();
        let 编码结果缓冲 = 编码结果.clone();
        Self {
            编码器,
            编码结果,
            编码结果缓冲,
        }
    }
}

impl 目标函数 for 冰雪清韵目标函数 {
    type 目标值 = 冰雪清韵指标;
    type 解类型 = 冰雪清韵决策;

    /// 计算各个部分编码的指标，然后将它们合并成一个指标输出
    fn 计算(
        &mut self,
        解: &冰雪清韵决策,
        变化: &Option<冰雪清韵决策变化>,
    ) -> (冰雪清韵指标, f64) {
        self.编码结果缓冲.clone_from(&self.编码结果);
        self.编码器.编码(解, 变化, &mut self.编码结果缓冲);
        let mut 重码数 = 0;
        for 编码信息 in self.编码结果缓冲.iter_mut() {
            if 编码信息.全码.选重标记 {
                重码数 += 1;
            }
        }

        if 变化.is_none() {
            self.编码结果.clone_from(&self.编码结果缓冲);
        }

        let 目标函数 = 重码数 as f64 / self.编码结果.len() as f64;
        let 指标 = 冰雪清韵指标 { 重码数 };
        (指标, 目标函数)
    }

    fn 接受新解(&mut self) {
        self.编码结果.clone_from(&self.编码结果缓冲);
    }
}
