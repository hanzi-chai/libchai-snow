use crate::{
    qingyun::{
        context::冰雪清韵上下文, encoder::冰雪清韵编码器, 元素安排, 冰雪清韵决策, 冰雪清韵决策变化,
        冰雪清韵决策空间, 冰雪清韵编码信息, 最大码长, 特简字, 进制,
    },
    time_block,
};
use chai::{objectives::目标函数, 元素, 棱镜, 键位分布信息};
use rustc_hash::FxHashMap;
use serde::Serialize;
use std::{fmt::Display, iter::zip};

const 分级数: usize = 10;
const 分级大小: [usize; 分级数] = [
    500,
    1000,
    1500,
    2000,
    2500,
    3000,
    4000,
    5000,
    6000,
    usize::MAX,
];

type 分段线性函数 = Vec<(usize, f64)>;

pub fn 线性插值(x: usize, 分段函数: &分段线性函数) -> f64 {
    let i = 分段函数.iter().position(|&(x1, _)| x1 > x).unwrap();
    if i == 0 {
        分段函数[0].1
    } else {
        let (x1, y1) = 分段函数[i - 1];
        let (x2, y2) = 分段函数[i];
        y1 + (y2 - y1) * (x - x1) as f64 / (x2 - x1) as f64
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct 冰雪清韵指标 {
    pub 字根数: usize,
    pub 简体选重数: u64,
    pub 简体分级选重数: [u64; 分级数],
    pub 简体选重率: f64,
    pub 简体稳健选重率: f64,
    pub 繁体选重数: u64,
    pub 繁体选重率: f64,
    pub 简繁通打选重率: f64,
    // 以下简码指标都是简体，不算繁体
    pub 组合当量: f64,
    pub 稳健组合当量: f64,
    pub 按键分布: FxHashMap<char, f64>,
    pub 按键分布偏差: f64,
    pub 码长: f64,
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
            "字根数：{}；码长：{:.4}；简体选重数：{}；简体选重率：{:.2}%；简体稳健选重率：{:.2}%\n",
            self.字根数,
            self.码长,
            self.简体选重数,
            self.简体选重率 * 100.0,
            self.简体稳健选重率 * 100.0,
        )?;
        for (分级, 大小) in 分级大小.iter().enumerate() {
            if 大小 != &usize::MAX {
                write!(f, "{} / {}；", self.简体分级选重数[分级], 大小)?;
            } else {
                write!(f, "{} / 其他；\n", self.简体分级选重数[分级])?;
            }
        }
        write!(
            f,
            "繁体选重数：{}；繁体选重率：{:.2}%；简繁通打选重率：{:.2}%；\n",
            self.繁体选重数,
            self.繁体选重率 * 100.0,
            self.简繁通打选重率 * 100.0
        )?;
        write!(
            f,
            "组合当量：{:.2}%；稳健组合当量：{:.2}%；按键分布偏差：{:.2}%；用指分布：",
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
    pub 当量信息: Vec<f64>,
    pub 键位分布信息: 键位分布信息,
    pub 棱镜: 棱镜,
    pub 决策空间: 冰雪清韵决策空间,
}

impl 冰雪清韵目标函数 {
    pub fn 新建(上下文: &冰雪清韵上下文, 编码器: 冰雪清韵编码器) -> Self {
        let 当量信息 = 上下文
            .棱镜
            .预处理当量信息(&上下文.原始当量信息, 进制.pow(最大码长 as u32) as usize);
        let 键位分布信息 = 上下文.棱镜.预处理键位分布信息(&上下文.原始键位分布信息);
        let 拆分序列 = vec![<[元素; 4]>::default(); 上下文.固定拆分.len()];
        let 拆分序列缓冲 = 拆分序列.clone();
        let 编码结果: Vec<_> = 上下文
            .固定拆分
            .iter()
            .map(|x| 冰雪清韵编码信息 {
                简体: x.gb2312,
                繁体: x.国字常用,
                简体频率: x.简体频率,
                繁体频率: x.繁体频率,
                混合频率: x.混合频率,
                全码: Default::default(),
                简体简码: Default::default(),
                简体选重: false,
                繁体选重: false,
                简繁通打选重: false,
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
            let mut 总稳健频率 = 0.0;
            let mut 简体选重率 = 0.0;
            let mut 繁体选重率 = 0.0;
            let mut 繁体选重数 = 0;
            let mut 简繁通打选重率 = 0.0;
            let mut 稳健选重频率 = 0.0;
            let mut 总组合当量 = 0.0;
            let mut 总稳健组合数 = 0.0;
            let mut 总稳健组合当量 = 0.0;
            let mut 按键数向量 = vec![0.0; 进制 as usize];
            let mut 码长 = 0.0;
            for (序号, 编码信息) in self.编码结果缓冲.iter_mut().enumerate() {
                if 编码信息.繁体选重 {
                    繁体选重率 += 编码信息.繁体频率;
                    繁体选重数 += 1;
                }
                if 编码信息.简繁通打选重 {
                    简繁通打选重率 += 编码信息.混合频率;
                }
                if 编码信息.简体 {
                    let 稳健频率 = (-(序号.min(6000) as f64) / 1500.0).exp();
                    if 编码信息.简体选重 {
                        简体选重率 += 编码信息.简体频率;
                        稳健选重频率 += 稳健频率;
                        let 分级 = 分级大小.iter().position(|&x| 序号 < x).unwrap();
                        简体分级选重数[分级] += 1;
                    }
                    let 简码 = 编码信息.简体简码;
                    let 编码长度 = 长度分界点.iter().position(|&x| 简码 < x).unwrap();
                    总稳健频率 += 稳健频率;
                    码长 += 编码信息.简体频率 * 编码长度 as f64;
                    总组合当量 += 编码信息.简体频率 as f64 * self.当量信息[简码 as usize];
                    总稳健组合数 += 稳健频率 * (编码长度 - 1) as f64;
                    总稳健组合当量 += 稳健频率 * self.当量信息[简码 as usize];
                    let mut 剩余编码 = 简码;
                    while 剩余编码 > 0 {
                        let 键 = 剩余编码 % 进制;
                        按键数向量[键 as usize] += 编码信息.简体频率;
                        剩余编码 /= 进制;
                    }
                }
            }

            let 字根数 = self
                .决策空间
                .字根
                .iter()
                .filter(|&&x| 解.元素[x] != 元素安排::未选取)
                .count();
            let 分布: Vec<_> = 按键数向量.iter().map(|x| *x as f64 / 码长 as f64).collect();
            let mut 按键分布偏差 = 0.0;
            for (frequency, loss) in zip(&分布, &self.键位分布信息) {
                let diff = frequency - loss.ideal;
                if diff > 0.0 {
                    按键分布偏差 += loss.gt_penalty * diff;
                } else {
                    按键分布偏差 -= loss.lt_penalty * diff;
                }
            }
            let mut 按键分布 = FxHashMap::default();
            for (键, 频率) in 按键数向量.iter().enumerate() {
                if let Some(键) = self.棱镜.数字转键.get(&(键 as u64)) {
                    按键分布.insert(*键, *频率 as f64 / 码长 as f64);
                }
            }
            let 简体选重数 = 简体分级选重数.iter().sum();
            let 简体稳健选重率 = 稳健选重频率 / 总稳健频率;
            let 组合当量 = 总组合当量 / (码长 - 1.0);
            let 稳健组合当量 = 总稳健组合当量 / 总稳健组合数;
            let 指标 = 冰雪清韵指标 {
                字根数,
                简体选重数,
                简体分级选重数,
                简体选重率,
                简体稳健选重率,
                繁体选重数,
                繁体选重率,
                简繁通打选重率,
                组合当量,
                稳健组合当量,
                按键分布,
                码长,
                按键分布偏差,
            };
            let 目标函数值 = 简体稳健选重率 * 100.0
                + 稳健组合当量 * 5.0
                + 按键分布偏差 * 1.0
                + (码长 - 2.4) * 5.0
                + 繁体选重率 * 30.0
                + 简繁通打选重率 * 10.0;

            (指标, 目标函数值)
        });
        if 变化.is_none() {
            self.编码结果.clone_from(&self.编码结果缓冲);
            self.拆分序列.clone_from(&self.拆分序列缓冲);
        }
        (指标, 目标函数值)
    }

    fn 接受新解(&mut self) {
        self.编码结果.clone_from(&self.编码结果缓冲);
        self.拆分序列.clone_from(&self.拆分序列缓冲);
    }
}
