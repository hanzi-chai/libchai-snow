use crate::qingyun::{
    context::冰雪清韵上下文, 不好的大集合键, 主根小码, 元素安排, 冰雪清韵决策, 冰雪清韵决策变化,
    冰雪清韵决策空间, 大集合, 笔画,
};
use chai::{operators::变异, 元素, 棱镜};
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
    下游字根: FxHashMap<元素, Vec<元素>>,
    笔画列表: Vec<元素>,
}

impl 变异 for 冰雪清韵操作 {
    type 解类型 = 冰雪清韵决策;
    fn 变异(&mut self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let 随机数: f64 = random();
        if 随机数 < 0.05 {
            self.改变补码键(决策)
        } else if 随机数 < 0.2 {
            self.交换韵母(决策)
        } else {
            let mut 变化 = if 随机数 < 0.3 {
                self.交换主副根(决策)
            } else if 随机数 < 0.35 {
                self.移动笔画(决策)
            } else if 随机数 < 0.4 {
                self.交换主根(决策)
            } else if 随机数 < 0.6 {
                self.产生副根(决策)
            } else if 随机数 < 0.8 {
                self.湮灭副根(决策)
            } else {
                self.移动副根(决策)
            };
            self.传播(&mut 变化, 决策);
            变化
        }
    }
}

impl 冰雪清韵操作 {
    pub fn 新建(上下文: &冰雪清韵上下文) -> Self {
        let 棱镜 = 上下文.棱镜.clone();
        let 决策空间 = 上下文.决策空间.clone();
        let 下游字根 = 上下文.下游字根.clone();
        for 字根 in &上下文.决策空间.字根 {
            let 安排 = &上下文.初始决策.元素[*字根];
            let mut valid = false;
            for 条件安排 in &决策空间.元素[*字根] {
                if &条件安排.安排 == 安排 && 上下文.初始决策.允许(条件安排) {
                    valid = true;
                    break;
                }
            }
            if !valid {
                let 字根 = &上下文.棱镜.数字转元素[&字根];
                println!("初始决策中的字根 {字根:?} 的安排 {安排:?} 在决策空间中没有合法的条件");
            }
        }
        let 笔画列表 = 笔画.iter().map(|s| 棱镜.元素转数字[*s]).collect();
        return 冰雪清韵操作 {
            _棱镜: 棱镜,
            决策空间,
            下游字根,
            笔画列表,
        };
    }

    fn 改变补码键(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        决策.补码键 = 大集合
            .into_iter()
            .filter(|x| !不好的大集合键.contains(x))
            .choose(&mut rng)
            .unwrap();
        冰雪清韵决策变化::无变化()
    }

    fn _改变第一主根小码(
        &self, 决策: &mut 冰雪清韵决策
    ) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        决策.第一主根 = 主根小码.into_iter().choose(&mut rng).unwrap();
        冰雪清韵决策变化::无变化()
    }

    fn _改变第二主根小码(
        &self, 决策: &mut 冰雪清韵决策
    ) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        决策.第二主根 = ['o', 'u', ';'].into_iter().choose(&mut rng).unwrap();
        冰雪清韵决策变化::无变化()
    }

    fn _移动声母(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let (声母, 安排列表) = self
            .决策空间
            .声母
            .iter()
            .map(|&x| (x, &self.决策空间.元素[x]))
            .filter(|(_, y)| y.len() > 1)
            .choose(&mut rng)
            .unwrap();
        决策.元素[声母] = 安排列表.iter().choose(&mut rng).unwrap().安排;
        冰雪清韵决策变化::无变化()
    }

    fn _移动韵母(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let mut 备选列表 = vec![];
        for 韵母 in &self.决策空间.韵母 {
            // if 鼻音韵母列表.contains(韵母) {
            //     continue;
            // }
            let 安排 = &决策.元素[*韵母];
            let mut 可行安排 = vec![];
            for 条件安排 in &self.决策空间.元素[*韵母] {
                if 条件安排.安排 != *安排 {
                    可行安排.push(条件安排.安排.clone());
                }
            }
            if !可行安排.is_empty() {
                备选列表.push((韵母.clone(), 可行安排));
            }
        }
        let (韵母, 安排列表) = 备选列表.into_iter().choose(&mut rng).unwrap();
        决策.元素[韵母] = 安排列表.into_iter().choose(&mut rng).unwrap();
        冰雪清韵决策变化::无变化()
    }

    fn 交换韵母(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let 非鼻音韵母列表 =
            ["韵-ai", "韵-ei", "韵-ao", "韵-ou", "韵-ü"].map(|s| self._棱镜.元素转数字[s]);
        let 鼻音韵母列表 = ["韵-an", "韵-en", "韵-ang", "韵-eng"].map(|s| self._棱镜.元素转数字[s]);
        let 选择: Vec<_> = if random::<f64>() < 0.5 {
            鼻音韵母列表.choose_multiple(&mut rng, 2).collect()
        } else {
            非鼻音韵母列表.choose_multiple(&mut rng, 2).collect()
        };
        let (韵母1, 韵母2) = (*选择[0], *选择[1]);
        let 安排1 = 决策.元素[韵母1].clone();
        let 安排2 = 决策.元素[韵母2].clone();
        决策.元素[韵母1] = 安排2;
        决策.元素[韵母2] = 安排1;
        冰雪清韵决策变化::无变化()
    }

    fn 传播(&self, 变化: &mut 冰雪清韵决策变化, 决策: &mut 冰雪清韵决策) {
        let mut 队列 = VecDeque::new();
        队列.append(&mut 变化.增加字根.clone().into());
        队列.append(&mut 变化.减少字根.clone().into());
        队列.append(&mut 变化.移动字根.clone().into());
        while !队列.is_empty() {
            let 元素 = 队列.pop_front().unwrap();
            let mut 合法 = false;
            let mut 新安排列表 = vec![];
            for 条件安排 in &self.决策空间.元素[元素] {
                if 决策.允许(条件安排) {
                    if 条件安排.安排 == 决策.元素[元素] {
                        合法 = true;
                        break;
                    }
                    if !matches!(
                        条件安排.安排,
                        元素安排::键位第一 { .. } | 元素安排::键位第二 { .. }
                    ) {
                        新安排列表.push(条件安排.安排.clone());
                    }
                }
            }
            if !合法 {
                if 新安排列表.is_empty()
                    || matches!(
                        决策.元素[元素],
                        元素安排::键位第一 { .. } | 元素安排::键位第二 { .. }
                    )
                {
                    let 元素字符串 = &self._棱镜.数字转元素[&元素];
                    panic!(
                        "{元素字符串:?} 没有合法的安排，传播失败，全部空间为 {:?}",
                        self.决策空间.元素[元素]
                    );
                } else {
                    let 新安排 = 新安排列表.choose(&mut thread_rng()).unwrap();
                    if 决策.元素[元素] == 元素安排::未选取 {
                        变化.增加字根.push(元素);
                    } else if 新安排 == &元素安排::未选取 {
                        变化.减少字根.push(元素);
                    } else {
                        变化.移动字根.push(元素);
                    }
                    决策.元素[元素] = 新安排.clone();
                }
            }
            for 下游元素 in self.下游字根.get(&元素).unwrap_or(&vec![]) {
                if !队列.contains(下游元素) {
                    队列.push_back(下游元素.clone());
                }
            }
        }
    }

    fn 产生副根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let mut 备选列表 = vec![];
        for 字根 in &self.决策空间.字根 {
            let 安排 = 决策.元素[*字根];
            if 安排 != 元素安排::未选取 {
                continue;
            }
            let mut 可行安排 = vec![];
            for 条件安排 in &self.决策空间.元素[*字根] {
                if matches!(条件安排.安排, 元素安排::未选取 | 元素安排::键位第二 { .. })
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
            决策.元素[字根] = 可行位置.into_iter().choose(&mut rng).unwrap().clone();
            冰雪清韵决策变化::新建(vec![], vec![字根], vec![])
        } else {
            冰雪清韵决策变化::无变化()
        }
    }

    fn 湮灭副根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let mut 备选列表 = vec![];
        for 字根 in &self.决策空间.字根 {
            let 安排 = &决策.元素[*字根];
            if matches!(
                安排,
                元素安排::未选取 | 元素安排::键位第一 { .. } | 元素安排::键位第二 { .. }
            ) {
                continue;
            }
            for 条件安排 in &self.决策空间.元素[*字根] {
                if 条件安排.安排 == 元素安排::未选取 && 决策.允许(条件安排) {
                    备选列表.push(字根.clone());
                }
            }
        }
        if 备选列表.is_empty() {
            return 冰雪清韵决策变化::无变化();
        }
        let 字根 = *备选列表.iter().choose(&mut rng).unwrap();
        决策.元素[字根] = 元素安排::未选取;
        冰雪清韵决策变化::新建(vec![], vec![], vec![字根])
    }

    fn 移动副根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let mut 备选列表 = vec![];
        for 字根 in &self.决策空间.字根 {
            let 安排 = 决策.元素[*字根];
            if self.笔画列表.contains(&字根) {
                continue;
            }
            if matches!(
                安排,
                元素安排::未选取 | 元素安排::键位第一 { .. } | 元素安排::键位第二 { .. }
            ) {
                continue;
            }
            let mut 可行安排 = vec![];
            for 条件安排 in &self.决策空间.元素[*字根] {
                if matches!(
                    条件安排.安排,
                    元素安排::未选取 | 元素安排::键位第一 { .. } | 元素安排::键位第二 { .. }
                ) || 条件安排.安排 == 安排
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
        决策.元素[字根] = 安排列表.into_iter().choose(&mut rng).unwrap().clone();
        冰雪清韵决策变化::新建(vec![字根], vec![], vec![])
    }

    fn 交换主副根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let mut 备选列表 = vec![];
        for 字根 in &self.决策空间.字根 {
            let 安排 = 决策.元素[*字根];
            if self.笔画列表.contains(&字根) {
                continue;
            }
            if matches!(
                安排,
                元素安排::未选取 | 元素安排::键位第一 { .. } | 元素安排::键位第二 { .. }
            ) {
                continue;
            }
            let 乱序安排 = &self.决策空间.元素[*字根]
                .iter()
                .filter(|&x| {
                    matches!(
                        x.安排,
                        元素安排::键位第一 { .. } | 元素安排::键位第二 { .. }
                    )
                })
                .choose(&mut rng);
            if let Some(乱序安排) = 乱序安排 {
                if 决策.允许(&乱序安排) {
                    备选列表.push((字根.clone(), 乱序安排.安排.clone()));
                }
            }
        }
        let (字根, 安排) = 备选列表.into_iter().choose(&mut rng).unwrap();
        let 元素安排::键位第二(键位) = 安排 else {
            panic!("字根 {字根:?} 的安排不是乱序");
        };
        let 当前该键位上主根 = self
            .决策空间
            .字根
            .iter()
            .find(|&字根| {
                !self.笔画列表.contains(字根)
                    && matches!(决策.元素[*字根], 元素安排::键位第二(k) if k == 键位)
            })
            .unwrap()
            .clone();
        决策.元素[字根] = 安排;
        let mut 可行安排 = vec![];
        for 条件安排 in &self.决策空间.元素[当前该键位上主根] {
            if !matches!(条件安排.安排, 元素安排::未选取 | 元素安排::键位第二 { .. })
                && 决策.允许(条件安排)
            {
                可行安排.push(条件安排.安排.clone());
            }
        }
        if 可行安排.is_empty() {
            panic!(
                "在 {:?} 中没有 {:?} 可行的安排，无法交换主副根",
                self.决策空间.元素[当前该键位上主根], self._棱镜.数字转元素[&当前该键位上主根]
            );
        }
        决策.元素[当前该键位上主根] = 可行安排.choose(&mut rng).unwrap().clone();
        冰雪清韵决策变化::新建(vec![字根, 当前该键位上主根], vec![], vec![])
    }

    fn 移动笔画(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let 字根 = *self.笔画列表.choose(&mut rng).unwrap();
        let 安排 = self.决策空间.元素[字根]
            .iter()
            .choose(&mut rng)
            .unwrap()
            .clone();
        决策.元素[字根] = 安排.安排;
        冰雪清韵决策变化::新建(vec![字根], vec![], vec![])
    }

    fn 交换主根(&self, 决策: &mut 冰雪清韵决策) -> 冰雪清韵决策变化 {
        let mut rng = thread_rng();
        let 主根列表: Vec<_> = self
            .决策空间
            .字根
            .iter()
            .filter_map(|k| {
                if self.笔画列表.contains(&k) {
                    return None;
                }
                if let 元素安排::键位第二(键位) = &决策.元素[*k] {
                    return Some((k.clone(), *键位));
                }
                None
            })
            .collect();
        let (字根1, 键位1) = *主根列表.iter().choose(&mut rng).unwrap();
        let (字根2, 键位2) = *主根列表.iter().choose(&mut rng).unwrap();
        if 字根1 == 字根2 {
            return 冰雪清韵决策变化::无变化();
        }
        决策.元素[字根1] = 元素安排::键位第二(键位2);
        决策.元素[字根2] = 元素安排::键位第二(键位1);
        冰雪清韵决策变化::新建(vec![字根1, 字根2], vec![], vec![])
    }
}
