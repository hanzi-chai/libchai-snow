use crate::feihua::{冰雪飞花安排, 空格};
use crate::feihua::{encoder::冰雪飞花编码器, 冰雪飞花上下文, 冰雪飞花决策};
use chai::{contexts::default::默认决策变化, encoders::编码器};
use chai::objectives::目标函数;
use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Clone, Serialize)]
pub struct 字根数统计 {
    pub 声托字根数: u64,
    pub 韵托字根数: u64,
    pub 形托字根数: u64,
    pub 归并字根数: u64,
    pub 无理字根数: u64,
}

#[derive(Clone, Serialize)]
pub struct 冰雪飞花指标 {
    pub 一字全码选重率: f64,
    pub 一字全码选重数: u64,
    pub 一字全码指数选重率: f64,
    pub 字根数统计: 字根数统计,
    pub 复杂度: f64,
}

impl Display for 冰雪飞花指标 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "复杂度：{:.0}，一字全码选重率：{:.0}μ，一字全码指数选重率：{:.0}μ，一字全码选重数：{}\n",
            self.复杂度,
            self.一字全码选重率 * 1_000_000.0,
            self.一字全码指数选重率 * 1_000_000.0,
            self.一字全码选重数
        )?;
        write!(
            f,
            "声托字根数：{}，韵托字根数：{}，形托字根数：{}，归并字根数：{}，无理字根数：{}\n",
            self.字根数统计.声托字根数,
            self.字根数统计.韵托字根数,
            self.字根数统计.形托字根数,
            self.字根数统计.归并字根数,
            self.字根数统计.无理字根数
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

    fn 统计字根并计算复杂度(&self, 决策: &冰雪飞花决策) -> (字根数统计, f64) {
        let mut 复杂度 = 0.0;
        let 棱镜 = &self.编码器.棱镜;
        let mut 声托字根数 = 0;
        let mut 韵托字根数 = 0;
        let mut 形托字根数 = 0;
        let mut 归并字根数 = 0;
        let mut 无理字根数 = 0;
        for (元素, 安排) in 决策.元素.iter().enumerate() {
            if 元素 <= 空格.into() { continue; } // 跳过空格
            let 元素名称 = &棱镜.数字转元素[&元素];
            if 元素名称.contains("-") {
                continue; // 跳过「声-」、「韵-」和「形-」等元素
            }
            match 安排 {
                冰雪飞花安排::未选取 => {}
                冰雪飞花安排::键位(_) => {
                    复杂度 += 5.0;
                    无理字根数 += 1;
                }
                冰雪飞花安排::归并(归并元素) => {
                    let 归并元素名称 = &棱镜.数字转元素[归并元素];
                    if 归并元素名称.starts_with("声-") {
                        复杂度 += 0.0;
                        声托字根数 += 1;
                    } else if 归并元素名称.starts_with("韵-") {
                        复杂度 += 2.0;
                        韵托字根数 += 1;
                    } else if 归并元素名称.starts_with("形-") {
                        复杂度 += 2.0;
                        形托字根数 += 1;
                    } else {
                        复杂度 += 0.0; // 归并到其他字根
                        归并字根数 += 1;
                    }
                }
            }
        }
        let 字根数统计 = 字根数统计 {
            声托字根数,
            韵托字根数,
            形托字根数,
            归并字根数,
            无理字根数,
        };
        (字根数统计, 复杂度)
    }
}

impl 目标函数 for 冰雪飞花目标函数 {
    type 目标值 = 冰雪飞花指标;
    type 决策 = 冰雪飞花决策;

    fn 计算(
        &mut self,
        决策: &冰雪飞花决策,
        _变化: &Option<默认决策变化>,
    ) -> (Self::目标值, f64) {
        self.编码器.编码(决策, _变化, &mut []);
        let mut 一字总频率 = 0;
        let mut 一字全码选重频率 = 0;
        let mut 一字总指数频率 = 0.0;
        let mut 一字全码选重指数频率 = 0.0;
        let mut 一字全码选重数 = 0;
        for 编码信息 in &self.编码器.编码结果 {
            一字总频率 += 编码信息.频率;
            一字总指数频率 += 编码信息.指数频率;
            if 编码信息.选重 {
                一字全码选重频率 += 编码信息.频率;
                一字全码选重指数频率 += 编码信息.指数频率;
                一字全码选重数 += 1;
            }
        }
        let 一字全码选重率 = 一字全码选重频率 as f64 / 一字总频率 as f64;
        let 一字全码指数选重率 = 一字全码选重指数频率 / 一字总指数频率;
        let 一字全码静态选重率 = 一字全码选重数 as f64 / self.编码器.词信息.len() as f64;
        let (字根数统计, 复杂度) = self.统计字根并计算复杂度(决策);
        let 分数 = 一字全码静态选重率 + 复杂度 * 3e-4;
        let 指标 = 冰雪飞花指标 {
            一字全码选重率,
            一字全码指数选重率,
            一字全码选重数,
            字根数统计,
            复杂度,
        };
        (指标, 分数)
    }
}
