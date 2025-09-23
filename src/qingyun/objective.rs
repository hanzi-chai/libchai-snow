use crate::{
    qingyun::{
        context::冰雪清韵上下文, encoder::冰雪清韵编码器, 元素安排, 冰雪清韵决策, 冰雪清韵决策变化,
        冰雪清韵决策空间, 冰雪清韵编码信息, 所有汉字数, 最大码长, 特简字, 进制, 频序, 频率,
    },
    time_block,
};
use chai::{objectives::目标函数, 元素, 棱镜, 键位分布信息};
use rustc_hash::FxHashMap;
use serde::Serialize;
use std::{fmt::Display, iter::zip};

const 分级数: usize = 10;
const 分级大小: [频序; 分级数] = [
    500,
    1000,
    1500,
    2000,
    2500,
    3000,
    4000,
    5000,
    6000,
    频序::MAX,
];

#[derive(Debug, Clone, Serialize)]
pub struct 冰雪清韵指标 {
    pub 字根数: usize,
    pub 简体分级选重数: [频序; 分级数],
    pub 简体选重率: 频率,
    pub 简体稳健选重率: 频率,
    pub 繁体分级选重数: [频序; 分级数],
    pub 繁体选重率: 频率,
    pub 繁体稳健选重率: 频率,
    pub 通打选重数: 频序,
    pub 通打平方选重数: 频序,
    pub 通打最大选重数: u8,
    pub 通打选重率: 频率,
    // 简码指标仅限于简体
    pub 组合当量: 频率,
    pub 稳健组合当量: 频率,
    pub 按键分布: FxHashMap<char, 频率>,
    pub 按键分布偏差: 频率,
    pub 码长: 频率,
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
            "简体选重数：{}；简体选重率：{:.0}μ；简体稳健选重率：{:.0}μ\n",
            self.简体分级选重数.iter().sum::<频序>(),
            self.简体选重率 * 1e6,
            self.简体稳健选重率 * 1e6,
        )?;
        write!(f, "简体选重分布：")?;
        for (分级, 大小) in 分级大小.iter().enumerate() {
            if 大小 != &频序::MAX {
                write!(f, "{} / {}；", self.简体分级选重数[分级], 大小)?;
            } else {
                write!(f, "{} / 其他；\n", self.简体分级选重数[分级])?;
            }
        }
        write!(
            f,
            "繁体选重数：{}；繁体选重率：{:.0}μ；繁体稳健选重率：{:.0}μ\n",
            self.繁体分级选重数.iter().sum::<频序>(),
            self.繁体选重率 * 1e6,
            self.繁体稳健选重率 * 1e6,
        )?;
        write!(f, "繁体选重分布：")?;
        for (分级, 大小) in 分级大小.iter().enumerate() {
            if 大小 != &频序::MAX {
                write!(f, "{} / {}；", self.繁体分级选重数[分级], 大小)?;
            } else {
                write!(f, "{} / 其他；\n", self.繁体分级选重数[分级])?;
            }
        }
        write!(
            f,
            "通打选重数：{}；通打选重率：{:.0}μ；通打最大选重数：{}；通打平方选重数：{}\n",
            self.通打选重数,
            self.通打选重率 * 1e6,
            self.通打最大选重数,
            self.通打平方选重数
        )?;
        write!(
            f,
            "字根数：{}；码长：{:.4}；组合当量：{:.2}%；稳健组合当量：{:.2}%；按键分布偏差：{:.2}%；用指分布：",
            self.字根数,
            self.码长,
            self.组合当量 * 100.0,
            self.稳健组合当量 * 100.0,
            self.按键分布偏差 * 100.0
        )?;
        for 行 in 键盘布局.iter() {
            if 行.iter().any(|x| self.按键分布.contains_key(x)) {
                f.write_str("\n")?;
                let mut buffer = vec![];
                for 键 in 行 {
                    if let Some(频率) = self.按键分布.get(键) {
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
    pub 编码结果: Vec<冰雪清韵编码信息>,
    pub 编码结果缓冲: Vec<冰雪清韵编码信息>,
    pub 拆分序列: Vec<[元素; 4]>,
    pub 拆分序列缓冲: Vec<[元素; 4]>,
    pub 当量信息: Vec<频率>,
    pub 键位分布信息: 键位分布信息,
    pub 棱镜: 棱镜,
    pub 决策空间: 冰雪清韵决策空间,
}

impl 冰雪清韵目标函数 {
    pub fn 新建(上下文: &冰雪清韵上下文, 编码器: 冰雪清韵编码器) -> Self {
        let 当量信息 = 上下文
            .棱镜
            .预处理当量信息(&上下文.原始当量信息, 进制.pow(最大码长 as u32) as usize)
            .iter()
            .map(|&x| x as 频率)
            .collect();
        let 键位分布信息 = 上下文.棱镜.预处理键位分布信息(&上下文.原始键位分布信息);
        let 拆分序列 = vec![<[元素; 4]>::default(); 上下文.固定拆分.len()];
        let 拆分序列缓冲 = 拆分序列.clone();
        let 编码结果: Vec<_> = 上下文
            .固定拆分
            .iter()
            .map(|x| 冰雪清韵编码信息 {
                简体: x.gb2312,
                繁体: x.国字常用 || x.陆标,
                简体频率: x.简体频率,
                简体频序: x.简体频序,
                繁体频率: x.繁体频率,
                繁体频序: x.繁体频序,
                通打频率: x.通打频率,
                全码: Default::default(),
                简体简码: Default::default(),
                简体选重: 0,
                繁体选重: 0,
                通打选重: 0,
                完成出简: false,
                特简: 特简字.iter().position(|&c| c == x.词).unwrap_or(0) as u8,
            })
            .collect();
        let 编码结果缓冲 = 编码结果.clone();
        Self {
            编码器,
            编码结果,
            编码结果缓冲,
            拆分序列,
            拆分序列缓冲,
            当量信息,
            键位分布信息,
            棱镜: 上下文.棱镜.clone(),
            决策空间: 上下文.决策空间.clone(),
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
        time_block!("备份", {
            self.编码结果缓冲.clone_from(&self.编码结果);
            self.拆分序列缓冲.clone_from(&self.拆分序列);
        });
        time_block!("构建拆分序列", {
            if let Some(变化) = 变化 {
                if 变化.拆分改变 {
                    self.编码器.构建拆分序列(解, &mut self.拆分序列缓冲);
                }
            } else {
                self.编码器.构建拆分序列(解, &mut self.拆分序列缓冲);
            }
        });
        self.编码器
            .动态编码(解, &self.拆分序列缓冲, &mut self.编码结果缓冲);
        let (指标, 目标函数值) = time_block!("统计", {
            let 长度分界点 = [0, 1, 2, 3, 4].map(|x| 进制.pow(x));
            let mut 简体分级选重数 = [0; 分级数];
            let mut 简体总稳健频率 = 0.0;
            let mut 简体稳健选重频率 = 0.0;
            let mut 简体选重率 = 0.0;
            let mut 繁体分级选重数 = [0; 分级数];
            let mut 繁体总稳健频率 = 0.0;
            let mut 繁体稳健选重频率 = 0.0;
            let mut 繁体选重率 = 0.0;
            let mut 通打选重率 = 0.0;
            let mut 通打选重数 = 0;
            let mut 通打平方选重数 = 0;
            let mut 通打最大选重数 = 0;
            let mut 总组合当量 = 0.0;
            let mut 总稳健组合数 = 0.0;
            let mut 总稳健组合当量 = 0.0;
            let mut 按键数向量 = vec![0.0; 进制 as usize];
            let mut 码长 = 0.0;
            for 编码信息 in &self.编码结果缓冲 {
                if 编码信息.通打选重 > 0 {
                    通打选重率 += 编码信息.通打频率;
                    通打选重数 += 1;
                    通打平方选重数 += (2 * 编码信息.通打选重 - 1) as 频序;
                    if 编码信息.通打选重 > 通打最大选重数 {
                        通打最大选重数 = 编码信息.通打选重;
                    }
                }
                if 编码信息.简体 {
                    let 简体指数频率 = ((编码信息.简体频序 as 频率).min(6000.0) / -1500.0).exp();
                    简体总稳健频率 += 简体指数频率;
                    if 编码信息.简体选重 > 0 {
                        简体选重率 += 编码信息.简体频率;
                        简体稳健选重频率 += 简体指数频率;
                        let 分级 = 分级大小
                            .iter()
                            .position(|&x| 编码信息.简体频序 < x)
                            .unwrap();
                        简体分级选重数[分级] += 1;
                    }
                    if 编码信息.简体频序 < 3000 {
                        let 简码 = 编码信息.简体简码;
                        let 编码长度 = 长度分界点.iter().position(|&x| 简码 < x).unwrap();
                        码长 += 编码信息.简体频率 * 编码长度 as 频率;
                        总组合当量 += 编码信息.简体频率 as 频率 * self.当量信息[简码 as usize];
                        let 稳健频率 = ((编码信息.简体频序 as 频率).min(6000.0) / -1500.0).exp();
                        总稳健组合数 += 稳健频率 * (编码长度 - 1) as 频率;
                        总稳健组合当量 += 稳健频率 * self.当量信息[简码 as usize];
                        let mut 剩余编码 = 简码;
                        while 剩余编码 > 0 {
                            let 键 = 剩余编码 % 进制;
                            按键数向量[键 as usize] += 编码信息.简体频率;
                            剩余编码 /= 进制;
                        }
                    }
                }
                if 编码信息.繁体 {
                    let 繁体指数频率 = ((编码信息.繁体频序 as 频率).min(6000.0) / -1000.0).exp();
                    繁体总稳健频率 += 繁体指数频率;
                    if 编码信息.繁体选重 > 0 {
                        繁体选重率 += 编码信息.繁体频率;
                        繁体稳健选重频率 += 繁体指数频率;
                        let 分级 = 分级大小
                            .iter()
                            .position(|&x| 编码信息.繁体频序 < x)
                            .unwrap();
                        繁体分级选重数[分级] += 1;
                    }
                }
            }

            let 字根数 = self
                .决策空间
                .字根
                .iter()
                .filter(|&&x| 解.元素[x] != 元素安排::未选取)
                .count();
            let 分布: Vec<_> = 按键数向量.iter().map(|x| *x as 频率 / 码长).collect();
            let mut 按键分布偏差 = 0.0;
            for (frequency, loss) in zip(&分布, &self.键位分布信息) {
                let diff = frequency - loss.ideal as 频率;
                if diff > 0.0 {
                    按键分布偏差 += loss.gt_penalty as 频率 * diff;
                } else {
                    按键分布偏差 -= loss.lt_penalty as 频率 * diff;
                }
            }
            let mut 按键分布 = FxHashMap::default();
            for (键, 频率) in 按键数向量.iter().enumerate() {
                if let Some(键) = self.棱镜.数字转键.get(&(键 as u64)) {
                    按键分布.insert(*键, *频率 / 码长);
                }
            }
            let 简体稳健选重率 = 简体稳健选重频率 / 简体总稳健频率;
            let 繁体稳健选重率 = 繁体稳健选重频率 / 繁体总稳健频率;
            let 通打平方静态选重率 = 通打平方选重数 as 频率 / 所有汉字数 as 频率;
            let 组合当量 = 总组合当量 / (码长 - 1.0);
            let 稳健组合当量 = 总稳健组合当量 / 总稳健组合数;
            let 指标 = 冰雪清韵指标 {
                字根数,
                简体分级选重数,
                简体选重率,
                简体稳健选重率,
                繁体分级选重数,
                繁体选重率,
                繁体稳健选重率,
                通打选重率,
                通打选重数,
                通打最大选重数,
                通打平方选重数,
                组合当量,
                稳健组合当量,
                按键分布,
                码长,
                按键分布偏差,
            };
            let 目标函数值 = 简体稳健选重率 * 100.0
                + 繁体稳健选重率 * 20.0
                + 通打选重率 * 20.0
                + 通打平方静态选重率 * 2.0
                + 稳健组合当量 * 8.0
                + 按键分布偏差 * 3.0
                + (码长 - 2.4) * 2.0;

            (指标, 目标函数值)
        });
        if 变化.is_none() {
            self.编码结果.clone_from(&self.编码结果缓冲);
            self.拆分序列.clone_from(&self.拆分序列缓冲);
        }
        (指标, 目标函数值.into())
    }

    fn 接受新解(&mut self) {
        self.编码结果.clone_from(&self.编码结果缓冲);
        self.拆分序列.clone_from(&self.拆分序列缓冲);
    }
}
