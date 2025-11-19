use crate::qingyun::{
    context::冰雪清韵上下文, encoder::冰雪清韵编码器, 元素安排, 冰雪清韵决策, 冰雪清韵决策变化,
    冰雪清韵决策空间, 大集合, 所有汉字数, 转换, 进制, 音节信息, 频序, 频率,
};
use chai::{encoders::编码器, objectives::目标函数, 棱镜, 键位分布信息};
use rustc_hash::FxHashMap;
use serde::Serialize;
use std::{fmt::Display, iter::zip};

const 分级数: usize = 17;
const 分级大小: usize = 512;

#[derive(Debug, Clone, Serialize)]
pub struct 简体指标 {
    pub 分级选重数: [频序; 分级数],
    pub 选重率: 频率,
    pub 稳健选重率: 频率,
    // 简码指标仅限于简体
    pub 组合当量: 频率,
    pub 稳健组合当量: 频率,
    pub 形码分布: Vec<频率>,
    pub 形码分布偏差: 频率,
    pub 码长: 频率,
}

#[derive(Debug, Clone, Serialize)]
pub struct 简繁指标 {
    pub 选重数: 频序,
    pub 平方选重数: 频序,
    pub 最大选重数: u8,
    pub 选重率: 频率,
}

#[derive(Debug, Clone, Serialize)]
pub struct 繁体指标 {
    pub 分级选重数: [频序; 分级数],
    pub 选重率: 频率,
    pub 稳健选重率: 频率,
}

#[derive(Debug, Clone, Serialize)]
pub struct 正则化指标 {
    pub 字根组数: usize,
    pub 字根组数方差: f32,
    pub 字根数: usize,
    pub 字根难度: f32,
    pub 韵母难度: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct 冰雪清韵指标 {
    pub 简体: 简体指标,
    pub 繁体: 繁体指标,
    pub 简繁: 简繁指标,
    pub 正则化: 正则化指标,
    pub 音节熵: 频率,
    pub 双拼熵: 频率,
    pub 音码组合当量: 频率,
    pub 键转数字: FxHashMap<char, u64>,
}

const 键盘布局: [[char; 10]; 4] = [
    ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
    ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';'],
    ['z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/'],
    ['_', '\'', '-', '=', '[', ']', '\\', '`', ' ', ' '],
];

impl Display for 冰雪清韵指标 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "字根组数：{}；字根组数方差：{:.2}；字根数：{}；拆分难度：{:.2}；韵母难度：{:.2}；音节熵：{:.2}；双拼熵：{:.2}\n",
            self.正则化.字根组数,
            self.正则化.字根组数方差,
            self.正则化.字根数,
            self.正则化.字根难度,
            self.正则化.韵母难度,
            self.音节熵,
            self.双拼熵
        )?;
        write!(
            f,
            "简体选重数：{}；简体选重率：{:.0}μ；简体稳健选重率：{:.0}μ\n",
            self.简体.分级选重数.iter().sum::<频序>(),
            self.简体.选重率 * 1e6,
            self.简体.稳健选重率 * 1e6,
        )?;
        write!(f, "简体选重分布：")?;
        for 分级 in 0..分级数 {
            if 分级 != 分级数 - 1 {
                write!(
                    f,
                    "{} / {}；",
                    self.简体.分级选重数[分级],
                    (分级 + 1) * 分级大小
                )?;
            } else {
                write!(f, "{} / 其他；\n", self.简体.分级选重数[分级])?;
            }
        }
        write!(
            f,
            "繁体选重数：{}；繁体选重率：{:.0}μ；繁体稳健选重率：{:.0}μ\n",
            self.繁体.分级选重数.iter().sum::<频序>(),
            self.繁体.选重率 * 1e6,
            self.繁体.稳健选重率 * 1e6,
        )?;
        write!(f, "繁体选重分布：")?;
        for 分级 in 0..分级数 {
            if 分级 != 分级数 - 1 {
                write!(
                    f,
                    "{} / {}；",
                    self.繁体.分级选重数[分级],
                    (分级 + 1) * 分级大小
                )?;
            } else {
                write!(f, "{} / 其他；\n", self.繁体.分级选重数[分级])?;
            }
        }
        write!(
            f,
            "简繁选重数：{}；简繁选重率：{:.0}μ；简繁最大选重数：{}；简繁平方选重数：{}\n",
            self.简繁.选重数,
            self.简繁.选重率 * 1e6,
            self.简繁.最大选重数,
            self.简繁.平方选重数
        )?;
        write!(
            f,
            "码长：{:.4}；形码当量：{:.2}%；形码稳健当量：{:.2}%；音码当量：{:.2}%；形码分布偏差：{:.2}%；形码分布：",
            self.简体.码长,
            self.简体.组合当量 * 100.0,
            self.简体.稳健组合当量 * 100.0,
            self.音码组合当量 * 100.0,
            self.简体.形码分布偏差 * 100.0
        )?;
        for 行 in 键盘布局.iter() {
            if 行.iter().any(|x| self.键转数字.contains_key(x)) {
                f.write_str("\n")?;
                let mut buffer = vec![];
                for 键 in 行 {
                    if let Some(数字) = self.键转数字.get(键) {
                        let 频率 = self.简体.形码分布[*数字 as usize];
                        buffer.push(format!("{} {:5.2}%", 键, 频率 * 100.0));
                    }
                }
                f.write_str(&buffer.join(" | "))?;
            }
        }
        f.write_str("\n")
    }
}

pub struct 冰雪清韵目标函数 {
    pub 编码器: 冰雪清韵编码器,
    pub 当量信息: Vec<频率>,
    pub 键位分布信息: 键位分布信息,
    pub 棱镜: 棱镜,
    pub 决策空间: 冰雪清韵决策空间,
    pub 音节熵: f32,
    pub 简体总稳健频率: f32,
    pub 繁体总稳健频率: f32,
}

impl 冰雪清韵目标函数 {
    pub fn 新建(上下文: &冰雪清韵上下文, 编码器: 冰雪清韵编码器) -> Self {
        let 当量信息 = 上下文.预处理当量信息();
        let 键位分布信息 = 上下文.棱镜.预处理键位分布信息(&上下文.原始键位分布信息);
        let 简体总稳健频率 = 编码器
            .编码结果
            .iter()
            .filter_map(|x| {
                if x.简体 {
                    Some(x.简体指数频率)
                } else {
                    None
                }
            })
            .sum();
        let 繁体总稳健频率 = 编码器
            .编码结果
            .iter()
            .filter_map(|x| {
                if x.繁体 {
                    Some(x.繁体指数频率)
                } else {
                    None
                }
            })
            .sum();
        Self {
            编码器,
            当量信息,
            键位分布信息,
            棱镜: 上下文.棱镜.clone(),
            决策空间: 上下文.决策空间.clone(),
            音节熵: Self::计算音节熵(&上下文.拼音),
            简体总稳健频率,
            繁体总稳健频率,
        }
    }

    pub fn 计算音节熵(拼音: &[音节信息]) -> f32 {
        let mut 熵 = 0.0;
        for 音节 in 拼音 {
            if 音节.频率 > 0.0 {
                熵 -= 音节.频率 * 音节.频率.log2();
            }
        }
        熵
    }

    pub fn 计算音码指标(&self, 音码结果: &Vec<频率>) -> (f32, f32) {
        let mut 双拼熵 = 0.0;
        let mut 音码当量 = 0.0;
        for (编码, 频率) in 音码结果.iter().enumerate() {
            if *频率 > 0.0 {
                双拼熵 -= 频率 * 频率.log2();
                let 当量 = self.当量信息[编码];
                音码当量 += 频率 * 当量;
            }
        }
        (双拼熵, 音码当量)
    }

    pub fn 难度指标(&self, 决策: &冰雪清韵决策) -> 正则化指标 {
        let mut 字根难度 = 0.0;
        let mut 字根数 = 0;
        let mut 字根组数 = 0;
        let mut 大集合上字根数量 = FxHashMap::default();
        for 键 in 大集合 {
            大集合上字根数量.insert(键, 0);
        }
        for 字根 in &self.决策空间.字根 {
            let 安排 = 决策.元素[*字根];
            if 安排 != 元素安排::未选取 {
                字根数 += 1;
                if !matches!(安排, 元素安排::归并(..) | 元素安排::归并韵母 { .. }) {
                    字根组数 += 1;
                }
                if let 元素安排::声母韵母 { 声母, .. } = 安排 {
                    let 元素安排::键位(键) = 决策.元素[声母] else {
                        unreachable!();
                    };
                    大集合上字根数量.entry(键).and_modify(|x| *x += 1);
                }
            }
            let 安排列表 = &self.决策空间.元素[*字根];
            let mut 分值 = 0.0;
            for 条件安排 in 安排列表 {
                if 条件安排.安排 == 安排 {
                    分值 = 条件安排.打分 as f32;
                    break;
                }
            }
            字根难度 += 分值;
        }
        let mut 韵母难度 = 0.0;
        for 韵母 in &self.决策空间.韵母 {
            let 安排 = 决策.元素[*韵母];
            let 安排列表 = &self.决策空间.元素[*韵母];
            let mut 分值 = 0.0;
            for 条件安排 in 安排列表 {
                if 条件安排.安排 == 安排 {
                    分值 = 条件安排.打分 as f32;
                    break;
                }
            }
            韵母难度 += 分值;
        }
        let 平均数量 = 大集合上字根数量.values().sum::<usize>() as f32 / 大集合.len() as f32;
        let 字根组数方差: f32 = 大集合上字根数量
            .values()
            .map(|&x| (x as f32 - 平均数量).powi(2))
            .sum();
        正则化指标 {
            字根组数,
            字根组数方差,
            字根数,
            字根难度,
            韵母难度,
        }
    }

    pub fn calculate_simplified(&mut self) -> 简体指标 {
        let mut 简体分级选重数 = [0; 分级数];
        let mut 简体稳健选重频率 = 0.0;
        let mut 简体选重率 = 0.0;
        let mut 总组合当量 = 0.0;
        let mut 总稳健组合数 = 0.0;
        let mut 总稳健组合当量 = 0.0;
        let mut 形码分布 = vec![0.0; 进制 as usize];
        let mut 码长 = 0.0;
        // 简体选重标记
        for 索引 in &self.编码器.简体顺序 {
            let 编码信息 = &mut self.编码器.编码结果[*索引];
            if self.编码器.简体空间[编码信息.计重索引] {
                简体选重率 += 编码信息.简体频率;
                简体稳健选重频率 += 编码信息.简体指数频率;
                let 分级 = 编码信息.简体频序 as usize / 分级大小;
                简体分级选重数[分级] += 1;
            }
            self.编码器.简体空间[编码信息.计重索引] = true;
            if 编码信息.简体频序 < 3000 {
                let 简码 = 编码信息.简体简码;
                形码分布[简码[0] as usize] += 编码信息.简体频率;
                形码分布[简码[1] as usize] += 编码信息.简体频率;
                形码分布[简码[2] as usize] += 编码信息.简体频率;
                形码分布[简码[3] as usize] += 编码信息.简体频率;
                let 编码长度 = if 简码[0] != 0 {
                    4
                } else if 简码[1] != 0 {
                    3
                } else if 简码[2] != 0 {
                    2
                } else {
                    1
                };
                码长 += 编码信息.简体频率 * 编码长度 as 频率;
                总组合当量 += 编码信息.简体频率 as 频率 * self.当量信息[简码.to_usize()];
                总稳健组合数 += 编码信息.简体指数频率 * (编码长度 - 1) as 频率;
                总稳健组合当量 += 编码信息.简体指数频率 * self.当量信息[简码.to_usize()];
            }
        }
        形码分布.iter_mut().for_each(|x| *x /= 码长);
        let mut 形码分布偏差 = 0.0;
        for (频率, 损失函数) in zip(&形码分布, &self.键位分布信息) {
            let 差距 = 频率 - 损失函数.ideal as 频率;
            if 差距 > 0.0 {
                形码分布偏差 += 损失函数.gt_penalty as 频率 * 差距;
            } else {
                形码分布偏差 -= 损失函数.lt_penalty as 频率 * 差距;
            }
        }
        let 简体稳健选重率 = 简体稳健选重频率 / self.简体总稳健频率;
        let 组合当量 = 总组合当量 / (码长 - 1.0);
        let 稳健组合当量 = 总稳健组合当量 / 总稳健组合数;
        简体指标 {
            分级选重数: 简体分级选重数,
            选重率: 简体选重率,
            稳健选重率: 简体稳健选重率,
            组合当量,
            稳健组合当量,
            形码分布,
            形码分布偏差,
            码长,
        }
    }

    pub fn calculate_traditional(&mut self) -> 繁体指标 {
        let mut 繁体选重率 = 0.0;
        let mut 繁体分级选重数 = [0; 分级数];
        let mut 繁体稳健选重频率 = 0.0;
        // 繁体选重标记
        for 索引 in &self.编码器.繁体顺序 {
            let 编码信息 = &mut self.编码器.编码结果[*索引];
            if self.编码器.繁体空间[编码信息.计重索引] {
                繁体选重率 += 编码信息.繁体频率;
                繁体稳健选重频率 += 编码信息.繁体指数频率;
                let 分级 = 编码信息.繁体频序 as usize / 分级大小;
                繁体分级选重数[分级] += 1;
            }
            self.编码器.繁体空间[编码信息.计重索引] = true;
        }

        let 繁体稳健选重率 = 繁体稳健选重频率 / self.繁体总稳健频率;
        繁体指标 {
            分级选重数: 繁体分级选重数,
            选重率: 繁体选重率,
            稳健选重率: 繁体稳健选重率,
        }
    }

    pub fn calculate_combined(&mut self) -> 简繁指标 {
        let mut 通打选重率 = 0.0;
        let mut 通打选重数 = 0;
        let mut 通打平方选重数 = 0;
        let mut 通打最大选重数 = 0;

        // 简繁选重标记
        for 编码信息 in self.编码器.编码结果.iter_mut() {
            let 通打选重 = self.编码器.通打空间[编码信息.计重索引];
            if 通打选重 > 0 {
                通打选重率 += 编码信息.通打频率;
                通打选重数 += 1;
                通打平方选重数 += (2 * 通打选重 - 1) as 频序;
                if 通打选重 > 通打最大选重数 {
                    通打最大选重数 = 通打选重;
                }
            }
            self.编码器.通打空间[编码信息.计重索引] =
                self.编码器.通打空间[编码信息.计重索引].wrapping_add(1);
        }

        简繁指标 {
            选重数: 通打选重数,
            选重率: 通打选重率,
            最大选重数: 通打最大选重数,
            平方选重数: 通打平方选重数,
        }
    }

    pub fn calculate(&mut self, 解: &冰雪清韵决策) -> (冰雪清韵指标, f32) {
        let 简体指标 = self.calculate_simplified();
        let 繁体指标 = self.calculate_traditional();
        let 简繁指标 = self.calculate_combined();
        let (双拼熵, 音码组合当量) = self.计算音码指标(&self.编码器.音码空间);
        let 正则化指标 = self.难度指标(解);
        let 目标函数值 = 简体指标.稳健选重率 * 100.0
            + 简体指标.稳健组合当量 * 7.0
            + 简体指标.形码分布偏差 * 3.0
            + (简体指标.码长 - 2.5) * 1.5
            + 繁体指标.稳健选重率 * 20.0
            + 简繁指标.选重率 * 20.0
            + (简繁指标.平方选重数 as f32 / 所有汉字数 as f32) * 2.0
            + 音码组合当量 * 1.0
            + 正则化指标.字根难度 * 0.0012
            + 正则化指标.韵母难度 * 0.012
            + 正则化指标.字根组数方差 * 0.0003
            + (7.0 - 双拼熵) * 0.5;

        let 指标 = 冰雪清韵指标 {
            正则化: 正则化指标,
            简体: 简体指标,
            繁体: 繁体指标,
            简繁: 简繁指标,
            音节熵: self.音节熵,
            双拼熵,
            音码组合当量,
            键转数字: self.棱镜.键转数字.clone(),
        };
        (指标, 目标函数值)
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
        self.编码器.编码(解, 变化, &mut vec![]);
        let (指标, 目标函数值) = self.calculate(解);
        (指标, 目标函数值.into())
    }
}
