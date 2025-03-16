//! 冰雪四拼手机键位布局的优化问题。
//!

use chai::{
    data::{元素, 元素映射, 数据, 键},
    operators::变异,
};
use itertools::Itertools;
use rustc_hash::FxHashMap;

pub struct 冰雪四拼操作 {
    group1: Vec<Vec<[char; 3]>>,
    group2: Vec<Vec<char>>,
    group3: Vec<Vec<char>>,
    index: usize,
    元素转数字: FxHashMap<String, 元素>,
    键转数字: FxHashMap<char, 键>,
}

impl 变异 for 冰雪四拼操作 {
    fn 变异(&mut self, 映射: &mut 元素映射) -> Vec<元素> {
        let index1 = self.index % self.group1.len();
        let index2 = (self.index / self.group1.len()) % self.group2.len();
        let index3 = (self.index / self.group1.len() / self.group2.len()) % self.group3.len();
        let info1 = self.group1[index1].clone();
        let info2 = self.group2[index2].clone();
        let info3 = self.group3[index3].clone();
        // b, p, m, d, t, n 不变
        // g, k, h, j, q, x, z, c, s, w, y, v
        for (i, elements) in vec![
            ["g", "k", "h"],
            ["j", "q", "x"],
            ["z", "c", "s"],
            ["w", "y", "v"],
        ]
        .into_iter()
        .enumerate()
        {
            for (j, element) in elements.iter().enumerate() {
                let repr = self.元素转数字[&element.to_string()];
                映射[repr] = self.键转数字[&info1[i][j]];
            }
        }
        // r, l, f
        for (i, element) in vec!["r", "l", "f"].into_iter().enumerate() {
            let repr = self.元素转数字[element];
            映射[repr] = self.键转数字[&info2[i]];
        }
        // a, e, i, o, u
        for (i, element) in vec!["a", "e", "i", "o", "u"].into_iter().enumerate() {
            let repr = self.元素转数字[element];
            映射[repr] = self.键转数字[&info3[i]];
        }
        self.index += 1;
        vec![]
    }
}

fn make_permutation<T: Clone>(elements: &Vec<T>) -> Vec<Vec<T>> {
    let length = elements.len();
    elements
        .iter()
        .permutations(length)
        .map(|p| p.into_iter().cloned().collect())
        .collect()
}

impl 冰雪四拼操作 {
    pub fn new(数据: 数据) -> Self {
        let group1 = vec![
            ['G', 'K', 'H'],
            ['J', 'Q', 'X'],
            ['Z', 'C', 'S'],
            ['W', 'Y', 'V'],
        ];
        let group2 = vec!['R', 'L', 'F'];
        let group3 = vec!['A', 'E', 'I', 'O', 'U'];
        Self {
            group1: make_permutation(&group1),
            group2: make_permutation(&group2),
            group3: make_permutation(&group3),
            index: 0,
            元素转数字: 数据.元素转数字.clone(),
            键转数字: 数据.键转数字.clone(),
        }
    }
}
