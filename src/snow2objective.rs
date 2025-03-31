use crate::{snow2encoder::双编码占位符, tree::字根树控制器};
use chai::{
    data::{元素映射, 数据, 正则化, 编码信息},
    objectives::{default::默认目标函数, metric::默认指标, 目标函数},
    错误,
};
use serde::Serialize;
use std::fmt::Display;

pub struct 冰雪双拼目标函数 {
    默认目标函数: 默认目标函数,
    字根树控制器: 字根树控制器,
    正则化: 正则化,
}

#[derive(Clone, Serialize)]
pub struct 冰雪双拼指标 {
    默认指标: 默认指标,
    字根数: usize,
    单编码字根组数: usize,
    双编码字根组数: usize,
    字根记忆量: f64,
}

impl Display for 冰雪双拼指标 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "字根数：{}；", self.字根数)?;
        write!(f, "单编码字根组数：{}；", self.单编码字根组数)?;
        write!(f, "双编码字根组数：{}；", self.双编码字根组数)?;
        write!(f, "字根记忆量：{:.2}；", self.字根记忆量)?;
        write!(f, "{}", self.默认指标)
    }
}

impl 冰雪双拼目标函数 {
    pub fn 新建(数据: &数据) -> Result<Self, 错误> {
        let 默认目标函数 = 默认目标函数::新建(数据)?;
        let 正则化 = 数据.正则化.clone();
        let 字根树控制器 = 字根树控制器::新建(数据);
        Ok(Self {
            默认目标函数,
            字根树控制器,
            正则化,
        })
    }
}

impl 目标函数 for 冰雪双拼目标函数 {
    type 目标值 = 冰雪双拼指标;
    fn 计算(
        &mut self, 编码结果: &mut [编码信息], 映射: &元素映射
    ) -> (冰雪双拼指标, f64) {
        let (默认指标, 损失函数) = self.默认目标函数.计算(编码结果, 映射);
        let 被选取的字根: Vec<_> = self
            .字根树控制器
            .父映射
            .keys()
            .filter(|&x| self.字根树控制器.查询字根是否被选取(映射, x))
            .cloned()
            .collect();
        let 字根数 = 被选取的字根.len();
        let mut 字根记忆量 = 0.0;
        let mut 单编码字根组数 = 0;
        let mut 双编码字根组数 = 0;
        for 字根 in 被选取的字根 {
            let 键 = 映射[字根];
            if 键 == 双编码占位符 {
                双编码字根组数 += 1;
                continue;
            }
            let 归并列表 = self.正则化.get(&字根).cloned().unwrap_or(vec![]);
            let mut 最大亲和度 = 0.0;
            for (目标元素, 亲和度) in 归并列表.iter() {
                if 映射[*目标元素] == 键 {
                    最大亲和度 = 亲和度.max(最大亲和度);
                }
            }
            字根记忆量 += 1.0 - 最大亲和度;
            if 最大亲和度 == 0.0 {
                单编码字根组数 += 1;
            }
        }
        let 归一化记忆量 = 字根记忆量 / 字根数 as f64;
        let 损失函数 = 损失函数 + 归一化记忆量 * 0.08 + 字根数 as f64 * 0.0003;
        let 指标 = 冰雪双拼指标 {
            默认指标,
            字根数,
            单编码字根组数,
            双编码字根组数,
            字根记忆量,
        };
        (指标, 损失函数)
    }
}
