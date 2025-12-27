use crate::common::键盘布局;
use crate::snow2::{冰雪二拼上下文, 冰雪二拼决策, 冰雪二拼字根安排};
use crate::snow2::encoder::冰雪二拼编码器;
use chai::encoders::编码器;
use chai::objectives::default::默认目标函数参数;
use chai::objectives::目标函数;
use chai::{编码, 部分编码信息, 错误};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::iter::zip;

const 分级数: usize = 8;
const 分级大小: usize = 1024;

#[derive(Clone, Serialize)]
pub struct 冰雪二拼指标 {
    一字全码选重率: f64,
    一字全码分级选重数: [i64; 分级数],
    一字简码选重率: f64,
    一字简码分级选重数: [i64; 分级数],
    多字全码选重率: f64,
    组合当量: f64,
    按键分布: HashMap<char, f64>,
    韵母组数: usize,
    字根数: usize,
    单编码字根组数: usize,
    双编码字根组数: usize,
    字根记忆量: f64,
}

impl Display for 冰雪二拼指标 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "韵母组数：{}；", self.韵母组数)?;
        write!(f, "字根数：{}；", self.字根数)?;
        write!(f, "单编码字根组数：{}；", self.单编码字根组数)?;
        write!(f, "双编码字根组数：{}；", self.双编码字根组数)?;
        write!(f, "字根记忆量：{:.2}；", self.字根记忆量)?;
        write!(f, "\n")?;
        write!(
            f,
            "一字全码选重率：{:.4}%；选重数：{}；",
            self.一字全码选重率 * 100.0,
            self.一字全码分级选重数.iter().sum::<i64>()
        )?;
        for 分级 in 0..分级数 {
            write!(
                f,
                "{} 选重：{}；",
                (分级 + 1) * 分级大小,
                self.一字全码分级选重数[分级]
            )?;
        }
        write!(f, "\n")?;
        write!(f, "一字简码选重率：{:.4}%；选重数：{}；",
            self.一字简码选重率 * 100.0,
            self.一字简码分级选重数.iter().sum::<i64>()
        )?;
        for 分级 in 0..分级数 {
            write!(
                f,
                "{} 选重：{}；",
                (分级 + 1) * 分级大小,
                self.一字简码分级选重数[分级]
            )?;
        }
        write!(f, "\n")?;
        write!(f, "多字全码选重率：{:.4}%；", self.多字全码选重率 * 100.0)?;
        write!(f, "组合当量：{:.2}；", self.组合当量)?;
        write!(f, "用指分布：")?;
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

#[derive(Debug, Clone)]
pub struct 冰雪二拼缓存 {
    进制: u64,
    一字总频数: i64,
    多字总频数: i64,
    总组合数: i64,
    按键数向量: Vec<i64>,
    总组合当量: f64,
    一字全码总选重频数: i64,
    一字全码分级选重个数: [i64; 分级数],
    一字简码总选重频数: i64,
    一字简码分级选重个数: [i64; 分级数],
    多字全码总选重频数: i64,
    长度分界点: [u64; 5],
}

#[derive(PartialEq, Clone, Copy)]
pub enum 编码类型 {
    一字全码,
    一字简码,
    多字全码,
}

impl 冰雪二拼缓存 {
    pub fn 新建(进制: u64) -> Self {
        let 长度分界点 = [0, 1, 2, 3, 4].map(|x| 进制.pow(x));
        Self {
            进制,
            一字总频数: 0,
            多字总频数: 0,
            总组合数: 0,
            按键数向量: vec![0; 进制 as usize],
            总组合当量: 0.0,
            一字全码总选重频数: 0,
            一字全码分级选重个数: [0; 分级数],
            一字简码总选重频数: 0,
            一字简码分级选重个数: [0; 分级数],
            多字全码总选重频数: 0,
            长度分界点,
        }
    }

    #[inline(always)]
    pub fn 处理(
        &mut self,
        类型: 编码类型,
        索引: usize,
        频率: u64,
        编码信息: &mut 部分编码信息,
        参数: &默认目标函数参数,
    ) {
        if !编码信息.有变化 {
            return;
        }
        编码信息.有变化 = false;
        self.增减(
            类型,
            索引,
            频率,
            编码信息.实际编码,
            编码信息.选重标记,
            参数,
            1,
        );
        if 编码信息.上一个实际编码 == 0 {
            return;
        }
        self.增减(
            类型,
            索引,
            频率,
            编码信息.上一个实际编码,
            编码信息.上一个选重标记,
            参数,
            -1,
        );
    }

    #[inline(always)]
    pub fn 增减(
        &mut self,
        类型: 编码类型,
        索引: usize,
        频数: u64,
        编码: 编码,
        选重标记: bool,
        参数: &默认目标函数参数,
        正负号: i64,
    ) {
        use 编码类型::*;
        let 有向频数 = 频数 as i64 * 正负号;
        if 类型 != 多字全码 {
            self.一字总频数 += 有向频数;
        } else {
            self.多字总频数 += 有向频数;
        }
        // 手感（使用多字全码和一字简码）
        if 类型 != 一字全码 {
            // 1. 按键分布
            let mut 剩余编码 = 编码;
            while 剩余编码 > 0 {
                let 键 = 剩余编码 % self.进制;
                self.按键数向量[键 as usize] += 有向频数;
                剩余编码 /= self.进制;
            }
            // 2. 组合当量
            let 编码长度 = self.长度分界点.iter().position(|&x| 编码 < x).unwrap() as i64;
            self.总组合数 += (编码长度 - 1) * 有向频数;
            self.总组合当量 += 参数.当量信息[编码 as usize] * 有向频数 as f64;
        }
        // 离散
        if 选重标记 {
            if 类型 != 多字全码 {
                if 类型 == 一字全码 {
                    self.一字全码总选重频数 += 有向频数;
                    self.一字全码分级选重个数[索引 / 分级大小] += 正负号;
                } else {
                    self.一字简码总选重频数 += 有向频数;
                    self.一字简码分级选重个数[索引 / 分级大小] += 正负号;
                }
            } else {
                self.多字全码总选重频数 += 有向频数;
            }
        }
    }

    pub fn 汇总(&self, 参数: &默认目标函数参数) -> (冰雪二拼指标, f64) {
        // 初始化返回值和标量化的损失函数
        let mut 损失函数 = 0.0;
        // 一、全局指标
        // 1. 按键分布
        let 总频率: i64 = self.按键数向量.iter().sum();
        let 分布: Vec<_> = self
            .按键数向量
            .iter()
            .map(|x| *x as f64 / 总频率 as f64)
            .collect();
        let mut 距离 = 0.0;
        for (frequency, loss) in zip(&分布, &参数.键位分布信息) {
            let diff = frequency - loss.ideal;
            if diff > 0.0 {
                距离 += loss.gt_penalty * diff;
            } else {
                距离 -= loss.lt_penalty * diff;
            }
        }
        let mut 按键分布 = HashMap::new();
        for (键, 频率) in 分布.iter().enumerate() {
            if let Some(键) = 参数.数字转键.get(&(键 as u64)) {
                按键分布.insert(*键, *频率);
            }
        }
        损失函数 += 距离 * 0.015;
        // 2. 组合当量
        let 组合当量 = self.总组合当量 / self.总组合数 as f64;
        损失函数 += 组合当量 * 0.05;
        // 3. 重码
        let 一字全码选重率 = self.一字全码总选重频数 as f64 / self.一字总频数 as f64;
        let 一字简码选重率 = self.一字简码总选重频数 as f64 / self.一字总频数 as f64;
        let 多字全码选重率 = self.多字全码总选重频数 as f64 / self.多字总频数 as f64;
        损失函数 += 多字全码选重率;
        for 分级 in 0..分级数 {
            let 一字全码分级选重率 = self.一字全码分级选重个数[分级] as f64 / 6000.0;
            let 一字简码分级选重率 = self.一字简码分级选重个数[分级] as f64 / 6000.0;
            损失函数 += 一字全码分级选重率 * 0.2;
            损失函数 += 一字简码分级选重率 * 1.5;
        }
        let 指标 = 冰雪二拼指标 {
            一字全码选重率,
            一字全码分级选重数: self.一字全码分级选重个数,
            一字简码选重率,
            一字简码分级选重数: self.一字简码分级选重个数,
            多字全码选重率,
            组合当量,
            按键分布,
            韵母组数: 0,
            字根数: 0,
            单编码字根组数: 0,
            双编码字根组数: 0,
            字根记忆量: 0.0,
        };
        (指标, 损失函数)
    }
}

pub struct 冰雪二拼目标函数 {
    参数: 默认目标函数参数,
    缓存: 冰雪二拼缓存,
    编码器: 冰雪二拼编码器,
}

impl 冰雪二拼目标函数 {
    pub fn 新建(
        上下文: &冰雪二拼上下文, 编码器: 冰雪二拼编码器
    ) -> Result<Self, 错误> {
        // let 键位分布信息 = 上下文.键位分布信息.clone();
        // let 当量信息 = 上下文.当量信息.clone();
        // let 指法计数 = 上下文.棱镜.预处理指法标记(上下文.get_space());
        let 参数 = 默认目标函数参数 {
            键位分布信息: Default::default(),
            当量信息: Default::default(),
            指法计数: Default::default(),
            数字转键: 上下文.棱镜.数字转键.clone(),
            正则化强度: 1.0,
        };
        Ok(Self {
            参数,
            编码器,
            缓存: 冰雪二拼缓存::新建(上下文.棱镜.进制),
        })
    }
}

impl 目标函数 for 冰雪二拼目标函数 {
    type 目标值 = 冰雪二拼指标;
    type 决策 = 冰雪二拼决策;

    fn 计算(&mut self, 决策: &Self::决策, 变化: &Option<()>) -> (Self::目标值, f64) {
        self.编码器.编码(决策, 变化, &mut []);
        let mut 一字总频率 = 0;
        let mut 一字选重频率 = 0;
        let mut 一字全码分级选重数 = [0; 分级数];

        for (索引, 编码信息) in self.编码器.编码结果.iter_mut().enumerate() {
            let 频率 = 编码信息.频率;
            一字总频率 += 频率;
            if 编码信息.选重 {
                一字选重频率 += 频率;
                let 分级 = 索引 / 分级大小;
                一字全码分级选重数[分级] += 1;
            }

            // let 编码信息 {
            //     频率, 全码, 简码,
            // ..
            // } = 编码信息;
            // if 编码信息.词长 == 1 {
            //     self.缓存缓冲.处理(一字全码, 索引, *频率, 全码, &self.参数);
            //     self.缓存缓冲.处理(一字简码, 索引, *频率, 简码, &self.参数);
            // } else {
            //     self.缓存缓冲.处理(多字全码, 索引, *频率, 全码, &self.参数);
            // }
        }
        let 损失函数 = 一字全码分级选重数.iter().sum::<i64>() as f64 * 0.1;
        let mut 单编码字根组数 = 0;
        let mut 双编码字根组数 = 0;
        for 安排 in 决策.字根.values() {
            match 安排 {
                冰雪二拼字根安排::主根(_) => {
                    单编码字根组数 += 1;
                }
                冰雪二拼字根安排::副根(_, _) => {
                    双编码字根组数 += 1;
                }
                _ => {}
            }
        }
        let 指标 = 冰雪二拼指标 {
            一字全码选重率: 一字选重频率 as f64 / 一字总频率 as f64,
            一字全码分级选重数,
            一字简码选重率: 0.0,
            一字简码分级选重数: [0; 分级数],
            多字全码选重率: 0.0,
            组合当量: 0.0,
            按键分布: HashMap::new(),
            韵母组数: 0,
            字根数: 决策.字根.len(),
            单编码字根组数,
            双编码字根组数,
            字根记忆量: 0.0,
        };
        return (指标, 损失函数);
    }
}
