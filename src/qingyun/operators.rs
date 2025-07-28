use crate::qingyun::{
    冰雪清韵上下文, 冰雪清韵决策, 冰雪清韵决策变化, 冰雪清韵决策空间, 大集合, 字根安排,
};
use chai::{operators::变异, 棱镜};
use rand::{random, seq::IteratorRandom, thread_rng};
use rustc_hash::FxHashSet;

pub struct 冰雪清韵操作 {
    _棱镜: 棱镜,
    决策空间: 冰雪清韵决策空间,
}

impl 变异 for 冰雪清韵操作 {
    type 解类型 = 冰雪清韵决策;
    fn 变异(&mut self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let 随机数: f64 = random();
        if 随机数 < 0.1 {
            self.移动声母(决策)
        } else if 随机数 < 0.2 {
            self.移动韵母(决策)
        } else if 随机数 < 0.4 {
            self.交换主副根(决策)
        } else if 随机数 < 0.45 {
            self.移动笔画(决策)
        } else if 随机数 < 0.5 {
            self.交换主根(决策)
        } else if 随机数 < 0.6 {
            self.产生副根(决策)
        } else if 随机数 < 0.7 {
            self.湮灭副根(决策)
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
        冰雪清韵决策变化::新建()
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
        冰雪清韵决策变化::新建()
    }

    fn 产生副根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let a: Vec<_> = 决策
            .字根
            .iter()
            .filter_map(|(字根, 安排)| {
                if *安排 != 字根安排::未选取 {
                    return None;
                }
                let 可行位置: Vec<_> = self.决策空间.字根[字根]
                    .iter()
                    .filter(|&x| match x {
                        字根安排::未选取 => false,
                        字根安排::归并 {
                            字根: 被归并到字根
                        } => 决策.字根[被归并到字根] != 字根安排::未选取,
                        _ => true,
                    })
                    .cloned()
                    .collect();
                if 可行位置.is_empty() {
                    return None;
                }
                Some((字根.clone(), 可行位置))
            })
            .collect();
        if let Some((字根, 可行位置)) = a.into_iter().choose(&mut rng) {
            决策.字根[&字根] = 可行位置.into_iter().choose(&mut rng).unwrap().clone();
            冰雪清韵决策变化 { 拆分改变: true }
        } else {
            冰雪清韵决策变化::新建()
        }
    }

    fn 湮灭副根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let mut 可湮灭字根: FxHashSet<_> = self
            .决策空间
            .字根
            .iter()
            .filter_map(|(字根, 安排列表)| {
                if 安排列表.contains(&字根安排::未选取) {
                    Some(字根)
                } else {
                    None
                }
            })
            .collect();
        for (字根, 安排) in 决策.字根.iter() {
            if let 字根安排::归并 {
                字根: 归并到字根
            } = 安排
            {
                if 可湮灭字根.contains(归并到字根) {
                    可湮灭字根.remove(归并到字根);
                }
            } else if let 字根安排::乱序 { .. } = 安排 {
                可湮灭字根.remove(字根);
            } else if let 字根安排::未选取 = 安排 {
                可湮灭字根.remove(字根);
            }
        }
        if 可湮灭字根.is_empty() {
            return 冰雪清韵决策变化::新建();
        }
        let 字根 = 可湮灭字根.into_iter().choose(&mut rng).unwrap();
        决策.字根[字根] = 字根安排::未选取;
        冰雪清韵决策变化 { 拆分改变: true }
    }

    fn 移动副根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let mut 移动空间 = Vec::new();
        for (字根, 安排) in 决策.字根.iter() {
            match 安排 {
                字根安排::未选取 => continue,
                字根安排::乱序 { .. } => continue,
                其他 => {
                    let 全部安排: Vec<_> = self.决策空间.字根[字根]
                        .iter()
                        .filter(|&x| match x {
                            字根安排::未选取 => false,
                            字根安排::归并 {
                                字根: 被归并到字根
                            } => 决策.字根[被归并到字根] != 字根安排::未选取,
                            a => a != 其他,
                        })
                        .cloned()
                        .collect();
                    if 全部安排.is_empty() {
                        continue;
                    }
                    移动空间.push((字根.clone(), 全部安排));
                }
            }
        }
        let (字根, 安排列表) = 移动空间.into_iter().choose(&mut rng).unwrap();
        决策.字根[&字根] = 安排列表.into_iter().choose(&mut rng).unwrap().clone();
        冰雪清韵决策变化::新建()
    }

    fn 交换主副根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let 字根 = self
            .决策空间
            .允许乱序
            .iter()
            .filter(|&x| matches!(决策.字根[x], 字根安排::归并 { .. } | 字根安排::读音 { .. }))
            .choose(&mut rng)
            .unwrap();
        let 键位 = 大集合.chars().choose(&mut rng).unwrap();
        let 当前该键位上主根 = 决策
            .字根
            .iter()
            .find(|(字根, 安排)| {
                !"123456".contains(*字根)
                    && matches!(安排, 字根安排::乱序 { 键位: k } if *k == 键位)
            })
            .unwrap()
            .0
            .clone();
        决策.字根[字根] = 字根安排::乱序 { 键位 };
        决策.字根[&当前该键位上主根] = self.决策空间.字根[&当前该键位上主根]
            .iter()
            .filter(|&x| match x {
                字根安排::未选取 | 字根安排::乱序 { .. } => false,
                字根安排::归并 {
                    字根: 归并到字根
                } => 决策.字根[归并到字根] != 字根安排::未选取,
                _ => true,
            })
            .choose(&mut rng)
            .unwrap()
            .clone();
        冰雪清韵决策变化::新建()
    }

    fn 移动笔画(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let 字根 = ["1", "2", "3", "4", "5"]
            .into_iter()
            .choose(&mut rng)
            .unwrap();
        let 键位 = 大集合.chars().choose(&mut rng).unwrap();
        决策.字根[字根] = 字根安排::乱序 { 键位 };
        冰雪清韵决策变化::新建()
    }

    fn 交换主根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let 主根列表: Vec<_> = 决策
            .字根
            .clone()
            .into_iter()
            .filter(|(k, y)| !"123456".contains(k) && matches!(y, 字根安排::乱序 { .. }))
            .collect();
        if 主根列表.len() < 2 {
            return 冰雪清韵决策变化::新建();
        }
        let (字根1, 安排1) = 主根列表.iter().choose(&mut rng).unwrap();
        let (字根2, 安排2) = 主根列表.iter().choose(&mut rng).unwrap();
        if 字根1 == 字根2 {
            return 冰雪清韵决策变化::新建();
        }
        决策.字根.insert(字根1.clone(), 安排2.clone());
        决策.字根.insert(字根2.clone(), 安排1.clone());
        冰雪清韵决策变化::新建()
    }
}
