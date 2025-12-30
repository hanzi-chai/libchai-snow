use crate::feihua::{encoder::冰雪飞花编码器, 冰雪飞花上下文, 冰雪飞花决策};
use chai::encoders::编码器;
use chai::objectives::目标函数;
use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Clone, Serialize)]
pub struct 冰雪飞花指标 {
    pub 一字全码选重率: f64,
    pub 一字全码选重数: u64,
}

impl Display for 冰雪飞花指标 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "一字全码选重率：{:.2}%，一字全码选重数：{}\n",
            self.一字全码选重率 * 100.0,
            self.一字全码选重数
        )
    }
}

pub struct 冰雪飞花目标函数 {
    pub 编码器: 冰雪飞花编码器,
}

impl 冰雪飞花目标函数 {
    pub fn 新建(_上下文: &冰雪飞花上下文, 编码器: 冰雪飞花编码器) -> Self {
        Self { 编码器 }
    }
}

impl 目标函数 for 冰雪飞花目标函数 {
    type 目标值 = 冰雪飞花指标;
    type 决策 = 冰雪飞花决策;

    fn 计算(
        &mut self,
        决策: &crate::feihua::冰雪飞花决策,
        _变化: &Option<()>,
    ) -> (Self::目标值, f64) {
        self.编码器.编码(决策, _变化, &mut []);
        let mut 一字全码总频率 = 0;
        let mut 一字全码选重频率 = 0;
        let mut 一字全码选重数 = 0;
        for 编码信息 in &self.编码器.编码结果 {
            一字全码总频率 += 编码信息.频率;
            if 编码信息.选重 {
                一字全码选重频率 += 编码信息.频率;
                一字全码选重数 += 1;
            }
        }
        let 一字全码选重率 = 一字全码选重频率 as f64 / 一字全码总频率 as f64;
        let 一字全码静态选重率 = 一字全码选重数 as f64 / self.编码器.汉字信息.len() as f64;
        let 分数 = 一字全码选重率 + 0.1 * 一字全码静态选重率;
        let 指标 = 冰雪飞花指标 {
            一字全码选重率,
            一字全码选重数,
        };
        (指标, 分数)
    }
}
