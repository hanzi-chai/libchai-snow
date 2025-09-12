use crate::{
    qingyun::{
        context::冰雪清韵上下文, 冰雪清韵决策, 冰雪清韵决策变化, 冰雪清韵决策空间, 大集合,
        字根安排, 笔画,
    },
    time_block,
};
use chai::{operators::变异, 棱镜};
use rand::{
    random,
    seq::{IteratorRandom, SliceRandom},
    thread_rng,
};
use rustc_hash::FxHashMap;
use std::collections::VecDeque;

pub struct 冰雪清韵操作 {
    _棱镜: 棱镜,
    决策空间: 冰雪清韵决策空间,
    下游字根: FxHashMap<String, Vec<String>>,
}

impl 变异 for 冰雪清韵操作 {
    type 解类型 = 冰雪清韵决策;
    fn 变异(&mut self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        time_block!("变异", {
            let 备份 = 决策.clone();
            let 随机数: f64 = random();
            if 随机数 < 0.05 {
                self.改变补码键(决策)
            } else if 随机数 < 0.1 {
                self.移动声母(决策)
            } else if 随机数 < 0.2 {
                self.移动韵母(决策)
            } else {
                let mut 变化 = if 随机数 < 0.4 {
                    self.交换主副根(决策)
                // } else if 随机数 < 0.45 {
                //     self.移动笔画(决策)
                } else if 随机数 < 0.5 {
                    self.交换主根(决策)
                } else if 随机数 < 0.6 {
                    self.产生副根(决策)
                } else if 随机数 < 0.7 {
                    self.湮灭副根(决策)
                } else {
                    self.移动副根(决策)
                };
                if !变化.变化元素.is_empty() {
                    if let Err(_) = self.传播(&mut 变化, 决策) {
                        *决策 = 备份;
                        return 冰雪清韵决策变化::无变化();
                    }
                }
                变化
            }
        })
    }
}

impl 冰雪清韵操作 {
    pub fn 新建(上下文: &冰雪清韵上下文) -> Self {
        let 棱镜 = 上下文.棱镜.clone();
        let 决策空间 = 上下文.决策空间.clone();
        let 下游字根 = 上下文.下游字根.clone();
        for (字根, 安排) in &上下文.初始决策.字根 {
            let mut valid = false;
            for 条件安排 in &决策空间.字根[字根] {
                if &条件安排.安排 == 安排 && 上下文.初始决策.允许(条件安排) {
                    valid = true;
                    break;
                }
            }
            if !valid {
                panic!("初始决策中的字根 {字根} 的安排 {安排:?} 在决策空间中没有合法的条件");
            }
        }
        return 冰雪清韵操作 {
            _棱镜: 棱镜,
            决策空间,
            下游字根,
        };
    }

    fn 改变补码键(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        决策.补码键 = 大集合.into_iter().choose(&mut rng).unwrap();
        冰雪清韵决策变化::无变化()
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
        冰雪清韵决策变化::无变化()
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
        冰雪清韵决策变化::无变化()
    }

    fn 传播(
        &self, 变化: &mut 冰雪清韵决策变化, 决策: &mut 冰雪清韵决策
    ) -> Result<(), ()> {
        let mut 队列 = VecDeque::from(变化.变化元素.clone());
        while !队列.is_empty() {
            let 元素 = 队列.pop_front().unwrap();
            let mut 合法 = false;
            let mut 新安排列表 = vec![];
            for 条件安排 in &self.决策空间.字根[&元素] {
                if 决策.允许(条件安排) {
                    if 条件安排.安排 == 决策.字根[&元素] {
                        合法 = true;
                        break;
                    }
                    if !matches!(条件安排.安排, 字根安排::乱序 { .. }) {
                        新安排列表.push(条件安排.安排.clone());
                    }
                }
            }
            if !合法 {
                if 新安排列表.is_empty() || matches!(决策.字根[&元素], 字根安排::乱序 { .. })
                {
                    println!(
                        "{元素:?} 没有合法的安排，传播失败，全部空间为 {:?}",
                        self.决策空间.字根[&元素]
                    );
                    return Err(());
                } else {
                    let 安排 = 新安排列表.choose(&mut thread_rng()).unwrap();
                    if 决策.字根[&元素] == 字根安排::未选取 || 安排 == &字根安排::未选取
                    {
                        变化.拆分改变 = true;
                    }
                    变化.变化元素.push(元素.clone());
                    决策.字根[&元素] = 安排.clone();
                }
            }
            for 下游元素 in self.下游字根.get(&元素).unwrap_or(&vec![]) {
                if !队列.contains(下游元素) {
                    队列.push_back(下游元素.clone());
                }
            }
        }
        Ok(())
    }

    fn 产生副根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let mut 备选列表 = vec![];
        for (字根, 安排) in &决策.字根 {
            if *安排 != 字根安排::未选取 {
                continue;
            }
            let mut 可行安排 = vec![];
            for 条件安排 in &self.决策空间.字根[字根] {
                if matches!(条件安排.安排, 字根安排::未选取 | 字根安排::乱序 { .. })
                {
                    continue;
                }
                if 决策.允许(条件安排) {
                    可行安排.push(条件安排.安排.clone());
                }
            }
            if !可行安排.is_empty() {
                备选列表.push((字根.clone(), 可行安排));
            }
        }
        if let Some((字根, 可行位置)) = 备选列表.into_iter().choose(&mut rng) {
            决策.字根[&字根] = 可行位置.into_iter().choose(&mut rng).unwrap().clone();
            冰雪清韵决策变化::新建(vec![字根], true)
        } else {
            冰雪清韵决策变化::无变化()
        }
    }

    fn 湮灭副根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let mut 备选列表 = vec![];
        for (字根, 安排) in &决策.字根 {
            if matches!(安排, 字根安排::未选取 | 字根安排::乱序 { .. }) {
                continue;
            }
            for 条件安排 in &self.决策空间.字根[字根] {
                if 条件安排.安排 == 字根安排::未选取 && 决策.允许(条件安排) {
                    备选列表.push(字根.clone());
                }
            }
        }
        if 备选列表.is_empty() {
            return 冰雪清韵决策变化::无变化();
        }
        let 字根 = 备选列表.iter().choose(&mut rng).unwrap();
        决策.字根[字根] = 字根安排::未选取;
        冰雪清韵决策变化::新建(vec![字根.clone()], true)
    }

    fn 移动副根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let mut 备选列表 = vec![];
        for (字根, 安排) in &决策.字根 {
            if 笔画.contains(&字根.as_str()) {
                continue;
            }
            if matches!(安排, 字根安排::未选取 | 字根安排::乱序 { .. }) {
                continue;
            }
            let mut 可行安排 = vec![];
            for 条件安排 in &self.决策空间.字根[字根] {
                if matches!(条件安排.安排, 字根安排::未选取 | 字根安排::乱序 { .. })
                    || &条件安排.安排 == 安排
                {
                    continue;
                }
                if 决策.允许(条件安排) {
                    可行安排.push(条件安排.安排.clone());
                }
            }
            if !可行安排.is_empty() {
                备选列表.push((字根.clone(), 可行安排));
            }
        }
        let (字根, 安排列表) = 备选列表.into_iter().choose(&mut rng).unwrap();
        决策.字根[&字根] = 安排列表.into_iter().choose(&mut rng).unwrap().clone();
        冰雪清韵决策变化::新建(vec![字根], false)
    }

    fn 交换主副根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let mut 备选列表 = vec![];
        for (字根, 安排) in &决策.字根 {
            if 笔画.contains(&字根.as_str()) {
                continue;
            }
            if matches!(安排, 字根安排::未选取 | 字根安排::乱序 { .. }) {
                continue;
            }
            let 乱序安排 = &self.决策空间.字根[字根]
                .iter()
                .filter(|&x| matches!(x.安排, 字根安排::乱序 { .. }))
                .choose(&mut rng);
            if let Some(乱序安排) = 乱序安排 {
                if 决策.允许(&乱序安排) {
                    备选列表.push((字根.clone(), 乱序安排.安排.clone()));
                }
            }
        }
        let (字根, 安排) = 备选列表.into_iter().choose(&mut rng).unwrap();
        let 字根安排::乱序 { 键位, .. } = 安排 else {
            panic!("字根 {字根:?} 的安排不是乱序");
        };
        let 当前该键位上主根 = 决策
            .字根
            .iter()
            .find(|(字根, 安排)| {
                !笔画.contains(&字根.as_str())
                    && matches!(安排, 字根安排::乱序 { 键位: k, .. } if *k == 键位)
            })
            .unwrap()
            .0
            .clone();
        决策.字根[&字根] = 安排;
        let mut 可行安排 = vec![];
        for 条件安排 in &self.决策空间.字根[&当前该键位上主根] {
            if !matches!(条件安排.安排, 字根安排::未选取 | 字根安排::乱序 { .. })
                && 决策.允许(条件安排)
            {
                可行安排.push(条件安排.安排.clone());
            }
        }
        if 可行安排.is_empty() {
            panic!(
                "在 {:?} 中没有可行的安排，无法交换主副根",
                self.决策空间.字根[&当前该键位上主根]
            );
        }
        决策.字根[&当前该键位上主根] = 可行安排.choose(&mut rng).unwrap().clone();
        冰雪清韵决策变化::新建(vec![字根, 当前该键位上主根], false)
    }

    fn 移动笔画(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let 字根 = *笔画.choose(&mut rng).unwrap();
        let 安排 = self.决策空间.字根[字根]
            .iter()
            .filter(|&x| matches!(x.安排, 字根安排::乱序 { .. }))
            .choose(&mut rng)
            .unwrap()
            .clone();
        决策.字根[字根] = 安排.安排;
        冰雪清韵决策变化::新建(vec![字根.to_string()], false)
    }

    fn 交换主根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let 主根列表: Vec<_> = 决策
            .字根
            .iter()
            .filter_map(|(k, y)| {
                if 笔画.contains(&k.as_str()) {
                    return None;
                }
                if let 字根安排::乱序 { 键位, 韵母 } = y {
                    return Some((k.clone(), *键位, 韵母.clone()));
                }
                None
            })
            .collect();
        if 主根列表.len() < 2 {
            return 冰雪清韵决策变化::无变化();
        }
        let (字根1, 键位1, 韵母1) = 主根列表.iter().choose(&mut rng).unwrap();
        let (字根2, 键位2, 韵母2) = 主根列表.iter().choose(&mut rng).unwrap();
        if 字根1 == 字根2 {
            return 冰雪清韵决策变化::无变化();
        }
        决策.字根.insert(
            字根1.clone(),
            字根安排::乱序 {
                键位: 键位2.clone(),
                韵母: 韵母1.clone(),
            },
        );
        决策.字根.insert(
            字根2.clone(),
            字根安排::乱序 {
                键位: 键位1.clone(),
                韵母: 韵母2.clone(),
            },
        );
        冰雪清韵决策变化::新建(vec![字根1.clone(), 字根2.clone()], false)
    }
}
