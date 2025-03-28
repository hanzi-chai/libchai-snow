//! 冰雪双拼的优化问题。
//!

use crate::tree::字根树控制器;
use chai::data::{元素, 元素映射, 数据};
use chai::operators::变异;
use rand::seq::SliceRandom;
use rand::{random, thread_rng};
use rustc_hash::FxHashMap;
use std::collections::HashMap;

pub struct 冰雪双拼操作 {
    声母列表: Vec<元素>,
    韵母列表: Vec<元素>,
    韵部列表: Vec<[元素; 4]>, // 《中华通韵》中的韵部
    字根列表: Vec<元素>,
    字根树控制器: 字根树控制器,
    键转数字: FxHashMap<char, u64>,
}

#[derive(PartialEq)]
pub enum 策略 {
    产生,
    湮灭,
    移动,
}

impl 变异 for 冰雪双拼操作 {
    fn 变异(&mut self, 映射: &mut 元素映射) -> Vec<元素> {
        let 随机数: f64 = random();
        if 随机数 < 0.4 {
            self.随机操作字根树(映射, 策略::移动)
        } else if 随机数 < 0.6 {
            self.随机操作字根树(映射, 策略::产生)
        } else if 随机数 < 0.7 {
            self.随机操作字根树(映射, 策略::湮灭)
        } else if 随机数 < 0.8 {
            self.随机移动声母(映射)
        } else {
            self.随机交换韵母(映射)
        }
    }
}

impl 冰雪双拼操作 {
    pub fn 新建(数据: &数据) -> Self {
        let mut 声母列表 = vec![];
        let mut 韵母列表 = vec![];
        let mut 韵部映射 = HashMap::new();
        let mut 字根列表 = vec![];
        for 元素 in (数据.进制 as usize)..数据.初始映射.len() {
            let 元素名 = &数据.数字转元素[&元素];
            if 元素名.starts_with("声介") || 元素名.starts_with("冰声") {
                声母列表.push(元素);
            } else if 元素名.starts_with("韵调") || 元素名.starts_with("冰韵") {
                韵母列表.push(元素);
                let 字符列表: Vec<char> = 元素名.chars().collect();
                let 声调 = 字符列表[字符列表.len() - 1].to_digit(10).unwrap() - 1;
                let 无声调韵母: String = 字符列表[..(字符列表.len() - 1)].iter().collect();
                韵部映射.entry(无声调韵母).or_insert([元素::default(); 4])[声调 as usize] = 元素;
            } else {
                字根列表.push(元素);
            }
        }
        let 韵部列表: Vec<[元素; 4]> = 韵部映射.into_iter().map(|(_, v)| v).collect();

        Self {
            声母列表,
            韵母列表,
            韵部列表,
            字根列表,
            键转数字: 数据.键转数字.clone(),
            字根树控制器: 字根树控制器::新建(数据),
        }
    }

    pub fn 随机移动字根(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let 元素 = *self.字根列表.choose(&mut rng).unwrap();
        let 目标按键: Vec<char> = "zawevmio;/".chars().collect();
        let 按键 = 目标按键.choose(&mut rng).unwrap();
        映射[元素] = self.键转数字[按键];
        vec![元素]
    }

    pub fn 随机操作字根树(&self, 映射: &mut 元素映射, 策略: 策略) -> Vec<元素> {
        let mut rng = thread_rng();
        let _映射 = 映射.clone();
        let 可行字根列表: Vec<_> = if let 策略::产生 = 策略 {
            self.字根列表
                .iter()
                .filter(|x| !self.字根树控制器.查询字根是否被选取(&_映射, x))
                .collect()
        } else {
            self.字根列表
                .iter()
                .filter(|x| self.字根树控制器.查询字根是否被选取(&_映射, x))
                .collect()
        };
        if 可行字根列表.is_empty() {
            return vec![];
        }
        let 字根 = **可行字根列表.choose(&mut rng).unwrap();
        let 字根当前键 = 映射[字根];
        let 父字根 = self.字根树控制器.父映射[&字根];
        let 父字根当前键 = 映射[父字根];
        if 策略 == 策略::湮灭 && 父字根 == 0 {
            return vec![];
        }
        let 所有目标按键集合: Vec<_> = "zawevmio;/".chars().map(|x| self.键转数字[&x]).collect();
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
        let 元素 = *self.声母列表.choose(&mut rng).unwrap();
        let 目标按键: Vec<char> = "qsxdcrftgbyhnujklp".chars().collect();
        let 按键 = 目标按键.choose(&mut rng).unwrap();
        映射[元素] = self.键转数字[按键];
        vec![元素]
    }

    pub fn 随机移动韵母(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let 韵母 = *self.韵母列表.choose(&mut rng).unwrap();
        let 目标键位: Vec<char> = "qazwsxedcrfvtgbyhnujmik,ol.p;/".chars().collect();
        let 键 = 目标键位.choose(&mut rng).unwrap();
        映射[韵母] = self.键转数字[键];
        vec![韵母]
    }

    pub fn 随机交换韵母(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let 韵母一 = *self.韵母列表.choose(&mut rng).unwrap();
        let 韵母二 = *self.韵母列表.choose(&mut rng).unwrap();
        let 键一 = 映射[韵母一];
        let 键二 = 映射[韵母二];
        映射[韵母一] = 键二;
        映射[韵母二] = 键一;
        vec![韵母一, 韵母二]
    }

    pub fn 随机交换韵部(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let 韵部一 = *self.韵部列表.choose(&mut rng).unwrap();
        let 韵部二 = *self.韵部列表.choose(&mut rng).unwrap();
        for (韵母一, 韵母二) in 韵部一.iter().zip(韵部二.iter()) {
            let (键一, 键二) = (映射[*韵母一], 映射[*韵母二]);
            映射[*韵母一] = 键二;
            映射[*韵母二] = 键一;
        }
        vec![
            韵部一[0], 韵部一[1], 韵部一[2], 韵部一[3], 韵部二[0], 韵部二[1], 韵部二[2], 韵部二[3],
        ]
    }
}
