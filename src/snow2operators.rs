//! 冰雪双拼的优化问题。
//!

use crate::dual::构建双编码映射;
use crate::snow2encoder::双编码占位符;
use crate::tree::字根树控制器;
use crate::冰雪双拼元素分类;
use chai::data::{元素, 元素映射, 数据, 键};
use chai::operators::变异;
use rand::seq::{IteratorRandom, SliceRandom};
use rand::{random, thread_rng};
use rustc_hash::FxHashMap;
use std::collections::HashMap;

pub struct 冰雪双拼操作 {
    元素分类: 冰雪双拼元素分类,
    字根树控制器: 字根树控制器,
    键转数字: FxHashMap<char, u64>,
    双编码映射: HashMap<元素, (元素, 元素)>,
}

#[derive(PartialEq)]
pub enum 策略 {
    产生,
    湮灭,
    移动,
}

const 大集合: &str = "bpmfdtnlzcsrgkhjqx";
const 小集合: &str = "awevyuio;/";
const 全集合: &str = "qazwsxedcrfvtgbyhnujmik,ol.p;/";

impl 变异 for 冰雪双拼操作 {
    fn 变异(&mut self, 映射: &mut 元素映射) -> Vec<元素> {
        let 随机数: f64 = random();
        if 随机数 < 0.4 {
            self.随机操作字根树(映射, 策略::移动)
        } else if 随机数 < 0.6 {
            self.随机操作字根树(映射, 策略::产生)
        } else if 随机数 < 0.7 {
            self.随机操作字根树(映射, 策略::湮灭)
        } else if 随机数 < 0.75 {
            self.随机移动声母(映射)
        } else if 随机数 < 0.8 {
            self.随机交换声母(映射)
        } else {
            self.随机交换韵部(映射)
        }
    }
}

impl 冰雪双拼操作 {
    pub fn 新建(数据: &数据) -> Self {
        let 双编码映射 = 构建双编码映射(数据);

        Self {
            元素分类: 冰雪双拼元素分类::新建(数据),
            键转数字: 数据.键转数字.clone(),
            字根树控制器: 字根树控制器::新建(数据),
            双编码映射,
        }
    }

    pub fn 随机移动字根(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let 元素 = *self.元素分类.字根列表.choose(&mut rng).unwrap();
        let 目标按键: Vec<char> = 小集合.chars().collect();
        let 按键 = 目标按键.choose(&mut rng).unwrap();
        映射[元素] = self.键转数字[按键];
        vec![元素]
    }

    pub fn 随机操作双编码(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let 不是双编码: Vec<_> = self
            .双编码映射
            .keys()
            .filter(|x| 映射[**x] != 双编码占位符)
            .cloned()
            .collect();
        let 是双编码: Vec<_> = self
            .双编码映射
            .keys()
            .filter(|x| 映射[**x] == 双编码占位符)
            .cloned()
            .collect();
        // 一分为四
        let 增加 = if 是双编码.is_empty() {
            true
        } else if 不是双编码.is_empty() {
            false
        } else {
            random::<f64>() < 0.2
        };
        if 增加 {
            let 元素 = *不是双编码.choose(&mut rng).unwrap();
            映射[元素] = 双编码占位符;
            vec![元素]
        } else {
            let 元素 = *是双编码.choose(&mut rng).unwrap();
            let 父字根 = self.字根树控制器.父映射[&元素];
            映射[元素] = 映射[父字根];
            vec![元素]
        }
    }

    pub fn 随机操作字根树(&self, 映射: &mut 元素映射, 策略: 策略) -> Vec<元素> {
        let mut rng = thread_rng();
        let _映射 = 映射.clone();
        let 可行字根列表: Vec<_> = if let 策略::产生 = 策略 {
            self.元素分类
                .字根列表
                .iter()
                .filter(|x| {
                    !self.字根树控制器.查询字根是否被选取(&_映射, x)
                        && 映射[**x] != 双编码占位符
                })
                .collect()
        } else {
            self.元素分类
                .字根列表
                .iter()
                .filter(|x| {
                    self.字根树控制器.查询字根是否被选取(&_映射, x)
                        && 映射[**x] != 双编码占位符
                })
                .collect()
        };
        if 可行字根列表.is_empty() {
            return vec![];
        }
        let 字根 = **可行字根列表.choose(&mut rng).unwrap();
        let 字根当前键 = 映射[字根];
        let 父字根 = self.字根树控制器.父映射[&字根];
        let 父字根当前键 = 映射[父字根];
        // 单笔画不能被湮灭
        if 策略 == 策略::湮灭 && 父字根 == 0 {
            return vec![];
        }
        let 所有目标按键集合: Vec<_> = 小集合.chars().map(|x| self.键转数字[&x]).collect();
        let 目标按键 = match 策略 {
            策略::产生 => {
                let 可行目标按键集合: Vec<_> = 所有目标按键集合
                    .into_iter()
                    .filter(|x| *x != 父字根当前键)
                    .collect();
                *可行目标按键集合.choose(&mut rng).unwrap()
            }
            策略::湮灭 => 父字根当前键,
            策略::移动 => {
                let 可行目标按键集合: Vec<_> = 所有目标按键集合
                    .into_iter()
                    .filter(|x| *x != 字根当前键 && *x != 父字根当前键)
                    .collect();
                *可行目标按键集合.choose(&mut rng).unwrap()
            }
        };
        let 所有被代表字根 = self.字根树控制器.字根树.获取所有被代表字根(&字根, 映射);
        for 被代表字根 in &所有被代表字根 {
            映射[*被代表字根] = 目标按键;
        }
        所有被代表字根
    }

    pub fn 随机移动声母(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let 元素 = *self.元素分类.声母列表.choose(&mut rng).unwrap();
        let 目标按键: Vec<char> = 大集合.chars().collect();
        let 按键 = 目标按键.choose(&mut rng).unwrap();
        映射[元素] = self.键转数字[按键];
        vec![元素]
    }

    pub fn 随机交换声母(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let 声母一 = *self.元素分类.声母列表.choose(&mut rng).unwrap();
        let 声母二 = *self.元素分类.声母列表.choose(&mut rng).unwrap();
        let 键一 = 映射[声母一];
        let 键二 = 映射[声母二];
        映射[声母一] = 键二;
        映射[声母二] = 键一;
        vec![声母一, 声母二]
    }

    pub fn 随机移动韵母(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let 韵母 = *self.元素分类.韵母列表.choose(&mut rng).unwrap();
        let 目标键位: Vec<char> = 全集合.chars().collect();
        let 键 = 目标键位.choose(&mut rng).unwrap();
        映射[韵母] = self.键转数字[键];
        vec![韵母]
    }

    pub fn 随机交换韵母(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let 韵母一 = *self.元素分类.韵母列表.choose(&mut rng).unwrap();
        let 韵母二 = *self.元素分类.韵母列表.choose(&mut rng).unwrap();
        let (键一, 键二) = (映射[韵母一], 映射[韵母二]);
        映射[韵母一] = 键二;
        映射[韵母二] = 键一;
        vec![韵母一, 韵母二]
    }

    pub fn 随机交换韵部(&mut self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let 韵部一 = self.元素分类.韵部列表.choose(&mut rng).unwrap();
        let 韵部二 = self.元素分类.韵部列表.choose(&mut rng).unwrap();
        for (韵母一, 韵母二) in 韵部一.iter().zip(韵部二.iter()) {
            let (键一, 键二) = (映射[*韵母一], 映射[*韵母二]);
            映射[*韵母一] = 键二;
            映射[*韵母二] = 键一;
        }
        韵部一.iter().chain(韵部二.iter()).copied().collect()
    }

    pub fn 相似(韵部一: &[键; 4], 韵部二: &[键; 4]) -> bool {
        let 相减: [i64; 4] = [
            韵部一[0] as i64 - 韵部二[0] as i64,
            韵部一[1] as i64 - 韵部二[1] as i64,
            韵部一[2] as i64 - 韵部二[2] as i64,
            韵部一[3] as i64 - 韵部二[3] as i64,
        ];
        if [-20, -10, 0, 10, 20]
            .iter()
            .any(|x| 相减.iter().all(|y| *y == *x))
        {
            return true;
        }
        let 相加: [键; 4] = [
            韵部一[0] + 韵部二[0],
            韵部一[1] + 韵部二[1],
            韵部一[2] + 韵部二[2],
            韵部一[3] + 韵部二[3],
        ];
        [11, 21, 31, 41, 51]
            .iter()
            .any(|x| 相加.iter().all(|y| *y == *x))
    }

    pub fn 随机交换声调(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let mut 区块列表: Vec<Vec<[usize; 4]>> = vec![];
        for 韵部 in self.元素分类.韵部列表.iter() {
            let mut 已分区 = false;
            for 区块 in 区块列表.iter_mut() {
                let 区块键位 = 区块.iter().map(|x| x.map(|y| 映射[y])).collect::<Vec<_>>();
                let 韵部键位 = 韵部.map(|x| 映射[x]);
                if Self::相似(&区块键位[0], &韵部键位)
                    && 区块键位.into_iter().all(|x| x != 韵部键位)
                {
                    区块.push(*韵部);
                    已分区 = true;
                    break;
                }
            }
            if !已分区 {
                区块列表.push(vec![*韵部]);
            }
        }
        assert!(区块列表.len() == 3);
        let 区块 = 区块列表.choose(&mut rng).unwrap();
        let (声调一, 声调二) = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]
            .into_iter()
            .choose(&mut rng)
            .unwrap();
        let mut 改变的韵母 = vec![];
        for 韵部 in 区块 {
            let 韵母一 = 韵部[声调一];
            let 韵母二 = 韵部[声调二];
            let (键一, 键二) = (映射[韵母一], 映射[韵母二]);
            映射[韵母一] = 键二;
            映射[韵母二] = 键一;
            改变的韵母.push(韵母一);
            改变的韵母.push(韵母二);
        }
        改变的韵母
    }
}
