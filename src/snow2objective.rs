use crate::{snow2encoder::空格, tree::字根树控制器, 冰雪双拼元素分类};
use chai::{
    data::{元素映射, 数据, 正则化, 编码信息},
    objectives::{default::默认目标函数, metric::默认指标, 目标函数},
    错误,
};
use serde::Serialize;
use std::{collections::HashSet, fmt::Display};

pub struct 冰雪双拼目标函数 {
    默认目标函数: 默认目标函数,
    正则化: 正则化,
    元素分类: 冰雪双拼元素分类,
}

#[derive(Clone, Serialize)]
pub struct 冰雪双拼指标 {
    默认指标: 默认指标,
    乱序声母数: usize,
    韵母组数: usize,
}

impl Display for 冰雪双拼指标 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "乱序声母数：{}；", self.乱序声母数)?;
        write!(f, "韵母组数：{}；", self.韵母组数)?;
        write!(f, "{}", self.默认指标)
    }
}

impl 冰雪双拼目标函数 {
    pub fn 新建(数据: &数据) -> Result<Self, 错误> {
        let 默认目标函数 = 默认目标函数::新建(数据)?;
        let 正则化 = 数据.正则化.clone();
        let 元素分类 = 冰雪双拼元素分类::新建(数据);
        Ok(Self {
            默认目标函数,
            正则化,
            元素分类,
        })
    }
}

impl 目标函数 for 冰雪双拼目标函数 {
    type 目标值 = 冰雪双拼指标;
    fn 计算(
        &mut self, 编码结果: &mut [编码信息], 映射: &元素映射
    ) -> (Self::目标值, f64) {
        let (mut 默认指标, 默认损失函数) = self.默认目标函数.计算(编码结果, 映射);
        let mut 乱序声母数 = 0;
        let mut 总频率 = 0;
        let mut 多字频率 = 0;
        for 编码信息 in 编码结果 {
            if 编码信息.词长 != 0 {
                多字频率 += 编码信息.频率;
            }
            总频率 += 编码信息.频率;
        }
        if let Some(words_full) = &mut 默认指标.words_full {
            if let Some(duplication) = words_full.duplication {
                words_full.duplication = Some(duplication * (总频率 as f64 / 多字频率 as f64));
            }
        }
        for 声母 in &self.元素分类.声母列表 {
            if let Some(键位列表) = self.正则化.get(声母) {
                let 键盘键位 = 键位列表[0].0;
                let 实际键位 = 映射[*声母] as usize;
                if 实际键位 != 键盘键位 {
                    乱序声母数 += 1;
                }
            }
        }
        let mut 韵母组数 = 0;
        for 韵部 in &self.元素分类.韵部列表 {
            let 阴平韵母 = 韵部[0];
            let 被归并 = self
                .正则化
                .get(&阴平韵母)
                .unwrap_or(&vec![])
                .iter()
                .any(|(韵母, _)| 映射[*韵母] == 映射[阴平韵母]);
            if !被归并 {
                韵母组数 += 1;
            }
        }
        let 损失函数 = 默认损失函数 + 韵母组数 as f64 * 0.00005;
        return (
            冰雪双拼指标 {
                默认指标,
                乱序声母数,
                韵母组数,
            },
            损失函数,
        );
    }
}

pub struct 冰雪双拼形码目标函数 {
    默认目标函数: 默认目标函数,
    字根树控制器: 字根树控制器,
    正则化: 正则化,
}

#[derive(Clone, Serialize)]
pub struct 冰雪双拼形码指标 {
    默认指标: 默认指标,
    字根数: usize,
    单编码字根组数: usize,
    双编码字根组数: usize,
    字根记忆量: f64,
}

impl Display for 冰雪双拼形码指标 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "字根数：{}；", self.字根数)?;
        write!(f, "单编码字根组数：{}；", self.单编码字根组数)?;
        write!(f, "双编码字根组数：{}；", self.双编码字根组数)?;
        write!(f, "字根记忆量：{:.2}；", self.字根记忆量)?;
        write!(f, "{}", self.默认指标)
    }
}

impl 冰雪双拼形码目标函数 {
    pub fn 新建(数据: &数据) -> Result<Self, 错误> {
        let 默认目标函数 = 默认目标函数::新建(数据)?;
        let mut 正则化 = 数据.正则化.clone();
        let 字根树控制器 = 字根树控制器::新建(数据);
        for (字根, 父字根) in 字根树控制器.父映射.iter() {
            let mut 祖先字根亲和列表 = vec![];
            let mut 指针 = 父字根;
            let mut 亲和度 = 1.0;
            while 字根树控制器.父映射.contains_key(指针) {
                指针 = &字根树控制器.父映射[指针];
                亲和度 -= 0.2; // 每向上追溯一层，亲和度降低 0.2
                祖先字根亲和列表.push((*指针, 亲和度));
            }
            正则化
                .entry(*字根)
                .or_insert_with(|| vec![]) // 如果没有，就初始化为空向量
                .extend(祖先字根亲和列表);
        }
        Ok(Self {
            默认目标函数,
            字根树控制器,
            正则化,
        })
    }
}

impl 目标函数 for 冰雪双拼形码目标函数 {
    type 目标值 = 冰雪双拼形码指标;
    fn 计算(
        &mut self,
        编码结果: &mut [编码信息],
        映射: &元素映射,
    ) -> (冰雪双拼形码指标, f64) {
        let (默认指标, 损失函数) = self.默认目标函数.计算(编码结果, 映射);
        let 被选取的字根: HashSet<_> = self
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
            if 键 == 空格 {
                双编码字根组数 += 1;
                continue;
            }
            let 归并列表 = self.正则化.get(&字根).cloned().unwrap_or(vec![]);
            let mut 最大亲和度 = 0.0;
            for (目标元素, 亲和度) in 归并列表.iter() {
                if 映射[*目标元素] == 键
                    && self.字根树控制器.查询字根是否被选取(映射, 目标元素)
                {
                    最大亲和度 = 亲和度.max(最大亲和度);
                }
            }
            字根记忆量 += 1.0 - 最大亲和度;
            if 最大亲和度 == 0.0 {
                单编码字根组数 += 1;
            }
        }
        let 归一化记忆量 = 字根记忆量 / 字根数 as f64;
        let 损失函数 = 损失函数 + 归一化记忆量 * 0.07 + 字根数 as f64 * 0.0003;
        let 指标 = 冰雪双拼形码指标 {
            默认指标,
            字根数,
            单编码字根组数,
            双编码字根组数,
            字根记忆量,
        };
        (指标, 损失函数)
    }
}
