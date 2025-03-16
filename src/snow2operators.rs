//! 冰雪双拼的优化问题。
//!

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
    键转数字: FxHashMap<char, u64>,
}

impl 变异 for 冰雪双拼操作 {
    fn 变异(&mut self, candidate: &mut 元素映射) -> Vec<元素> {
        let number: f64 = random();
        // 一共有三种情况：
        if number < 0.8 {
            self.随机移动字根(candidate)
        } else if number < 0.9 {
            self.随机移动声母(candidate)
        } else {
            // 3. 随机交换两个《中华通韵》中的韵部
            // self.randomly_swap_final(candidate)
            self.随机移动韵母(candidate)
        }
    }
}

impl 冰雪双拼操作 {
    pub fn 新建(数据: &数据) -> Self {
        let mut initials = vec![];
        let mut finals_map = HashMap::new();
        let mut radicals = vec![];
        let mut finals = vec![];
        for element in (数据.进制 as usize)..数据.初始映射.len() {
            let repr = &数据.数字转元素[&element];
            if repr.starts_with("声介") || repr.starts_with("冰声") {
                initials.push(element);
            } else if repr.starts_with("韵调") || repr.starts_with("冰韵") {
                finals.push(element);
                let chars: Vec<char> = repr.chars().collect();
                let tone = chars[chars.len() - 1].to_digit(10).unwrap() - 1;
                let toneless: String = chars[..(chars.len() - 1)].iter().collect();
                finals_map.entry(toneless).or_insert([元素::default(); 4])[tone as usize] = element;
            } else {
                radicals.push(element);
            }
        }
        let final_groups: Vec<[元素; 4]> = finals_map.into_iter().map(|(_, v)| v).collect();
        Self {
            声母列表: initials,
            韵母列表: finals,
            韵部列表: final_groups,
            字根列表: radicals,
            键转数字: 数据.键转数字.clone(),
        }
    }

    pub fn 随机移动字根(&self, candidate: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let element: 元素 = *self.字根列表.choose(&mut rng).unwrap();
        let destinations: Vec<char> = "zawevmio;/".chars().collect();
        let key = destinations.choose(&mut rng).unwrap();
        candidate[element] = self.键转数字[key];
        vec![element]
    }

    pub fn 随机移动声母(&self, candidate: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let element: 元素 = *self.声母列表.choose(&mut rng).unwrap();
        let destinations: Vec<char> = "qsxdcrftgbyhnujklp".chars().collect();
        let key = destinations.choose(&mut rng).unwrap();
        candidate[element] = self.键转数字[key];
        vec![element]
    }

    pub fn 随机移动韵母(&self, candidate: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let element: 元素 = *self.韵母列表.choose(&mut rng).unwrap();
        let destinations: Vec<char> = "qazwsxedcrfvtgbyhnujmik,ol.p;/".chars().collect();
        let key = destinations.choose(&mut rng).unwrap();
        candidate[element] = self.键转数字[key];
        vec![element]
    }

    pub fn 随机交换韵部(&self, candidate: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let group1 = *self.韵部列表.choose(&mut rng).unwrap();
        let group2 = *self.韵部列表.choose(&mut rng).unwrap();
        for (element1, element2) in group1.iter().zip(group2.iter()) {
            let (key1, key2) = (candidate[*element1], candidate[*element2]);
            candidate[*element1] = key2;
            candidate[*element2] = key1;
        }
        vec![
            group1[0], group1[1], group1[2], group1[3], group2[0], group2[1], group2[2], group2[3],
        ]
    }
}
