use std::collections::HashSet;

use crate::qingyun::{
    冰雪清韵上下文, 冰雪清韵决策, 冰雪清韵决策变化, 冰雪清韵决策空间, 大集合, 字根安排,
};
use chai::{operators::变异, 棱镜};
use rand::{random, seq::IteratorRandom, thread_rng};

pub struct 冰雪清韵操作 {
    _棱镜: 棱镜,
    决策空间: 冰雪清韵决策空间,
}

impl 变异 for 冰雪清韵操作 {
    type 解类型 = 冰雪清韵决策;
    fn 变异(&mut self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let number: f64 = random();
        if number < 0.1 {
            self.移动声母(决策)
        } else if number < 0.3 {
            self.移动韵母(决策)
        } else if number < 0.4 {
            self.产生主根(决策)
        } else if number < 0.5 {
            self.湮灭主根(决策)
        } else {
            self.移动副根(决策)
        }
    }
}

impl 冰雪清韵操作 {
    pub fn 新建(上下文: &冰雪清韵上下文) -> Self {
        let 棱镜 = 上下文.棱镜.clone();
        let 决策空间 = 上下文.决策空间.clone();
        return 冰雪清韵操作 {
            _棱镜: 棱镜,
            决策空间,
        };
    }

    fn 移动声母(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let (声母, 安排列表) = self
            .决策空间
            .声母
            .iter()
            .filter(|(_, y)| y.len() > 1)
            .choose(&mut rng)
            .unwrap();
        决策.声母[声母] = *安排列表.iter().choose(&mut rng).unwrap();
        冰雪清韵决策变化 {}
    }

    fn 移动韵母(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let (韵母, 安排列表) = self
            .决策空间
            .韵母
            .iter()
            .filter(|(_, y)| y.len() > 1)
            .choose(&mut rng)
            .unwrap();
        决策.韵母[韵母] = 安排列表.iter().choose(&mut rng).unwrap().clone();
        冰雪清韵决策变化 {}
    }

    // fn 产生副根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {}

    // fn 湮灭副根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {}

    fn 移动副根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let (字根, 安排列表) = self
            .决策空间
            .字根
            .iter()
            .filter(|(_, y)| y.len() > 1)
            .choose(&mut rng)
            .unwrap();
        决策.字根[字根] = 安排列表
            .iter()
            .filter(|x| **x != 字根安排::未选取)
            .choose(&mut rng)
            .unwrap()
            .clone();
        冰雪清韵决策变化 {}
    }

    fn 产生主根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let mut 可行主根位置: HashSet<_> = 大集合.chars().collect();
        for 安排 in 决策.字根.values() {
            if let 字根安排::乱序 { 键位 } = 安排 {
                可行主根位置.remove(&键位);
            }
        }
        if 可行主根位置.is_empty() {
            return 冰雪清韵决策变化 {};
        }
        let 字根 = self.决策空间.字根.keys().choose(&mut rng).unwrap();
        决策.字根[字根] = 字根安排::乱序 {
            键位: *可行主根位置.iter().choose(&mut rng).unwrap(),
        };
        冰雪清韵决策变化 {}
    }

    fn 湮灭主根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let 主根列表: Vec<_> = 决策
            .字根
            .clone()
            .into_iter()
            .filter(|(_, y)| matches!(y, 字根安排::乱序 { .. }))
            .collect();
        if 主根列表.is_empty() {
            return 冰雪清韵决策变化 {};
        }
        let (字根, _) = 主根列表.into_iter().choose(&mut rng).unwrap();
        let 安排列表 = &self.决策空间.字根[&字根];
        决策.字根[&字根] = 安排列表
            .iter()
            .filter(|x| **x != 字根安排::未选取)
            .choose(&mut rng)
            .unwrap()
            .clone();
        冰雪清韵决策变化 {}
    }

    // fn 交换主根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
    //     let mut rng = thread_rng();
    //     let 主根 = self.必要字根.iter().choose(&mut rng).unwrap();
    //     let 键位 = 大集合.chars().choose(&mut rng).unwrap();
    //     决策[主根] = self.棱镜.键转数字[&键位];
    //     vec![*主根]
    // }
}
