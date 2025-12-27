//! 冰雪二拼的优化问题。

use chai::operators::变异;
use chai::棱镜;
use rand::seq::{IndexedRandom, IteratorRandom};
use rand::{random, rng};

use crate::snow2::{
    冰雪二拼上下文, 冰雪二拼决策, 冰雪二拼字根安排, 声调总数, 小, 小集合, 键
};

pub struct 冰雪二拼操作 {
    pub 棱镜: 棱镜,
    pub 小集合键: [键; 小],
}

#[derive(PartialEq)]
pub enum 策略 {
    产生,
    湮灭,
    移动,
}

impl 变异 for 冰雪二拼操作 {
    type 决策 = 冰雪二拼决策;

    fn 变异(&mut self, 决策: &mut 冰雪二拼决策) {
        let 随机数: f64 = random();
        if 随机数 < 0.1 {
            self.随机移动韵母(决策)
        } else if 随机数 < 0.2 {
            self.随机交换韵母(决策)
        } else if 随机数 < 0.25 {
            self.随机移动声调(决策)
        } else if 随机数 < 0.3 {
            self.随机交换声调(决策)
        } else {
            self.随机移动字根(决策)
        }
    }
}

impl 冰雪二拼操作 {
    pub fn 新建(上下文: &冰雪二拼上下文) -> Self {
        let 小集合键 = 小集合.map(|x| 上下文.棱镜.键转数字[&x] as 键);
        Self {
            棱镜: 上下文.棱镜.clone(),
            小集合键,
        }
    }

    // pub fn 随机移动声母(&self, 决策: &mut 冰雪二拼决策) -> Vec<元素> {
    //     let mut rng = rng();
    //     let mut 声母逆决策 = FxHashMap::default();
    //     for 声母 in self.元素分类.声母列表.iter() {
    //         声母逆决策.entry(决策[*声母]).or_insert(vec![]).push(*声母);
    //     }
    //     assert!(声母逆决策.len() == 大集合大小);
    //     let mut 可移动声母 = vec![];
    //     for 声母列表 in 声母逆决策.values() {
    //         if 声母列表.len() > 1 {
    //             for 声母 in 声母列表 {
    //                 let 声母名 = self.数字转元素[声母][7..].to_string();
    //                 if 固定声母.contains(&声母名.as_str()) {
    //                     可移动声母.push(*声母);
    //                 }
    //             }
    //         }
    //     }
    //     let 声母 = *可移动声母.choose(&mut rng).unwrap();
    //     let 声母键 = *声母逆决策.keys().choose(&mut rng).unwrap();
    //     决策[声母] = 声母键;
    //     vec![声母]
    // }

    // pub fn 随机交换声母(&self, 决策: &mut 冰雪二拼决策) -> Vec<元素> {
    //     let mut rng = rng();
    //     let 可变声母列表: Vec<_> = self
    //         .元素分类
    //         .声母列表
    //         .iter()
    //         .filter(|x| {
    //             let 声母名 = self.数字转元素[*x][7..].to_string();
    //             固定声母.contains(&声母名.as_str())
    //         })
    //         .cloned()
    //         .collect();
    //     let 声母一 = *可变声母列表.choose(&mut rng).unwrap();
    //     let 声母二 = *可变声母列表.choose(&mut rng).unwrap();
    //     let 键一 = 决策[声母一];
    //     let 键二 = 决策[声母二];
    //     决策[声母一] = 键二;
    //     决策[声母二] = 键一;
    //     vec![声母一, 声母二]
    // }

    // pub fn 随机整键移动声母(&mut self, 决策: &mut 冰雪二拼决策) -> Vec<元素> {
    //     let mut rng = rng();
    //     // sdfghjkl 键必须为声母键
    //     let 固定声母键: FxHashSet<_> = "sdfghjkl".chars().map(|x| self.键转数字[&x]).collect();
    //     let 一码顶键: FxHashSet<_> = ",.".chars().map(|x| self.键转数字[&x]).collect();
    //     let 声母键: FxHashSet<_> = self.元素分类.声母列表.iter().map(|x| 决策[*x]).collect();
    //     // 避免小集合出现大跨排，因此如果声母键关于中排的反射不在声母键中，则不允许移出
    //     let 反射 = |x: &u64| {
    //         let 列 = (x - 1) % 10 + 1;
    //         let 行 = (x - 1) / 10;
    //         10 * (2 - 行) + 列
    //     };
    //     let 移出键 = 声母键
    //         .iter()
    //         .cloned()
    //         .filter(|x| !固定声母键.contains(x))
    //         .filter(|x| 声母键.contains(&反射(x)) || 一码顶键.contains(&反射(x)))
    //         .choose(&mut rng)
    //         .unwrap();
    //     // ;,./ 不能是声母键
    //     let 移入键 = "qazwsxedcrfvtgbyhnujmikolp"
    //         .chars()
    //         .map(|x| self.键转数字[&x])
    //         .filter(|x| !声母键.contains(x))
    //         .choose(&mut rng)
    //         .unwrap();
    //     let mut 更改的声母 = vec![];
    //     for 声母 in self.元素分类.声母列表.iter() {
    //         if 决策[*声母] == 移出键 {
    //             决策[*声母] = 移入键;
    //             更改的声母.push(*声母);
    //         }
    //     }
    //     更改的声母
    // }

    pub fn 随机移动韵母(&self, 决策: &mut 冰雪二拼决策) {
        let mut rng = rng();
        let 韵母 = 决策.韵母.keys().choose(&mut rng).cloned().unwrap();
        决策.韵母.insert(韵母, (0..10).choose(&mut rng).unwrap());
    }

    pub fn 随机交换韵母(&self, 决策: &mut 冰雪二拼决策) {
        let mut rng = rng();
        let 韵母一 = 决策.韵母.keys().choose(&mut rng).cloned().unwrap();
        let 韵母二 = 决策.韵母.keys().choose(&mut rng).cloned().unwrap();
        let (键一, 键二) = (决策.韵母[&韵母一], 决策.韵母[&韵母二]);
        决策.韵母.insert(韵母一, 键二);
        决策.韵母.insert(韵母二, 键一);
    }

    pub fn 随机移动声调(&self, 决策: &mut 冰雪二拼决策) {
        let mut rng = rng();
        let 声调 = (0..声调总数).choose(&mut rng).unwrap();
        决策.声调[声调] = (0..3).choose(&mut rng).unwrap();
    }

    pub fn 随机交换声调(&self, 决策: &mut 冰雪二拼决策) {
        let mut rng = rng();
        let 声调 = (0..声调总数).choose_multiple(&mut rng, 2);
        let (声调一, 声调二) = (声调[0], 声调[1]);
        let (键一, 键二) = (决策.声调[声调一], 决策.声调[声调二]);
        决策.声调[声调一] = 键二;
        决策.声调[声调二] = 键一;
    }

    pub fn 随机移动字根(&self, 决策: &mut 冰雪二拼决策) {
        let mut rng = rng();
        let (字根, _) = 决策
            .字根
            .iter()
            .filter(|(_, v)| matches!(v, 冰雪二拼字根安排::主根(_) | 冰雪二拼字根安排::副根(_, _)))
            .choose(&mut rng)
            .unwrap();
        let 笔画元素 = ["1", "2", "3", "4", "5"].map(|x| self.棱镜.元素转数字[&x.to_string()]);
        if 笔画元素.contains(&字根) || random::<f64>() < 0.5 {
            let 编码 = *self.小集合键.choose(&mut rng).unwrap();
            决策.字根.insert(字根.clone(), 冰雪二拼字根安排::主根(编码));
        } else {
            let 编码一 = *self.小集合键.choose(&mut rng).unwrap();
            let 编码二 = *self.小集合键.choose(&mut rng).unwrap();
            决策
                .字根
                .insert(字根.clone(), 冰雪二拼字根安排::副根(编码一, 编码二));
        }
    }
}
