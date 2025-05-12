//! 冰雪二拼的优化问题。
//!

use crate::dual::构建双编码映射;
use crate::snow2encoder::空格;
use crate::tree::字根树控制器;
use crate::冰雪二拼元素分类;
use chai::data::{元素, 元素映射, 数据};
use chai::operators::变异;
use rand::seq::{IteratorRandom, SliceRandom};
use rand::{random, thread_rng};
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::{HashMap, HashSet};

const 全集合: &str = "qazwsxedcrfvtgbyhnujmik,ol.p;/";
const 小集合: &str = "azxcrtyu;/";
const 大集合大小: usize = 18;
const 固定声母: [&str; 24] = [
    "b", "p", "m", "f", "d", "t", "n", "l", "g", "k", "h", "j", "q", "x", "zh", "ch", "sh", "r",
    "z", "c", "s", "零齐", "零开", "零合",
];
const 固定字根: [&str; 0] = [
    // "1", "2", "3", "4", "5", "二", "三", "四", "五", "六", "七", "八", "九", "十",
];

pub struct 冰雪二拼操作 {
    元素分类: 冰雪二拼元素分类,
    字根树控制器: 字根树控制器,
    键转数字: FxHashMap<char, u64>,
    数字转元素: FxHashMap<元素, String>,
    双编码映射: HashMap<元素, (元素, 元素)>,
    固定字根: HashSet<元素>,
}

#[derive(PartialEq)]
pub enum 策略 {
    产生,
    湮灭,
    移动,
}

impl 变异 for 冰雪二拼操作 {
    fn 变异(&mut self, 映射: &mut 元素映射) -> Vec<元素> {
        let 随机数: f64 = random();
        if 随机数 < 0.05 {
            self.随机移动声母(映射)
        } else if 随机数 < 0.1 {
            self.随机交换声母(映射)
        } else if 随机数 < 0.2 {
            self.随机移动韵部(映射)
        } else if 随机数 < 0.3 {
            self.随机交换韵部(映射)
        } else if 随机数 < 0.7 {
            self.随机操作字根树(映射, 策略::移动)
        } else if 随机数 < 0.9 {
            self.随机操作字根树(映射, 策略::产生)
        } else {
            self.随机操作字根树(映射, 策略::湮灭)
        }
    }
}

impl 冰雪二拼操作 {
    pub fn 新建(数据: &数据) -> Self {
        Self {
            元素分类: 冰雪二拼元素分类::新建(数据),
            键转数字: 数据.键转数字.clone(),
            数字转元素: 数据.数字转元素.clone(),
            字根树控制器: 字根树控制器::新建(数据),
            双编码映射: 构建双编码映射(数据),
            固定字根: 固定字根.map(|x| 数据.元素转数字[x]).into(),
        }
    }

    pub fn 随机移动声母(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let mut 声母逆映射 = FxHashMap::default();
        for 声母 in self.元素分类.声母列表.iter() {
            声母逆映射.entry(映射[*声母]).or_insert(vec![]).push(*声母);
        }
        assert!(声母逆映射.len() == 大集合大小);
        let mut 可移动声母 = vec![];
        for 声母列表 in 声母逆映射.values() {
            if 声母列表.len() > 1 {
                for 声母 in 声母列表 {
                    let 声母名 = self.数字转元素[声母][7..].to_string();
                    if 固定声母.contains(&声母名.as_str()) {
                        可移动声母.push(*声母);
                    }
                }
            }
        }
        let 声母 = *可移动声母.choose(&mut rng).unwrap();
        let 声母键 = *声母逆映射.keys().choose(&mut rng).unwrap();
        映射[声母] = 声母键;
        vec![声母]
    }

    pub fn 随机交换声母(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let 可变声母列表: Vec<_> = self
            .元素分类
            .声母列表
            .iter()
            .filter(|x| {
                let 声母名 = self.数字转元素[*x][7..].to_string();
                固定声母.contains(&声母名.as_str())
            })
            .cloned()
            .collect();
        let 声母一 = *可变声母列表.choose(&mut rng).unwrap();
        let 声母二 = *可变声母列表.choose(&mut rng).unwrap();
        let 键一 = 映射[声母一];
        let 键二 = 映射[声母二];
        映射[声母一] = 键二;
        映射[声母二] = 键一;
        vec![声母一, 声母二]
    }

    pub fn 随机整键移动声母(&mut self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        // sdfghjkl 键必须为声母键
        let 固定声母键: FxHashSet<_> = "sdfghjkl".chars().map(|x| self.键转数字[&x]).collect();
        let 一码顶键: FxHashSet<_> = ",.".chars().map(|x| self.键转数字[&x]).collect();
        let 声母键: FxHashSet<_> = self.元素分类.声母列表.iter().map(|x| 映射[*x]).collect();
        // 避免小集合出现大跨排，因此如果声母键关于中排的反射不在声母键中，则不允许移出
        let 反射 = |x: &u64| {
            let 列 = (x - 1) % 10 + 1;
            let 行 = (x - 1) / 10;
            10 * (2 - 行) + 列
        };
        let 移出键 = 声母键
            .iter()
            .cloned()
            .filter(|x| !固定声母键.contains(x))
            .filter(|x| 声母键.contains(&反射(x)) || 一码顶键.contains(&反射(x)))
            .choose(&mut rng)
            .unwrap();
        // ;,./ 不能是声母键
        let 移入键 = "qazwsxedcrfvtgbyhnujmikolp"
            .chars()
            .map(|x| self.键转数字[&x])
            .filter(|x| !声母键.contains(x))
            .choose(&mut rng)
            .unwrap();
        let mut 更改的声母 = vec![];
        for 声母 in self.元素分类.声母列表.iter() {
            if 映射[*声母] == 移出键 {
                映射[*声母] = 移入键;
                更改的声母.push(*声母);
            }
        }
        更改的声母
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

    pub fn 随机移动韵部(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        let 韵部 = *self.元素分类.韵部列表.choose(&mut rng).unwrap();
        let 当前列 = (映射[韵部[0]] - 1) % 10 + 1;
        let 目标列 = (1..=10).filter(|x| *x != 当前列).choose(&mut rng).unwrap();
        for 韵母 in 韵部 {
            let 当前键 = 映射[韵母];
            let 目标键 = 当前键 + (目标列 - 当前列);
            assert!(
                目标键 > 0 && 目标键 < 31,
                "当前键: {}, 目标键: {}, 当前列: {}, 目标列: {}",
                当前键,
                目标键,
                当前列,
                目标列
            );
            映射[韵母] = 目标键;
        }
        韵部.iter().copied().collect()
    }

    pub fn 随机交换韵部(&self, 映射: &mut 元素映射) -> Vec<元素> {
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

    pub fn 随机移动声调(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        // 固定轻声
        let 声调 = *[0, 1, 2, 3].choose(&mut rng).unwrap();
        let 当前行 = (映射[self.元素分类.韵部列表[0][声调]] - 1) / 10;
        let 目标行 = vec![0, 1, 2]
            .into_iter()
            .filter(|x| *x != 当前行)
            .choose(&mut rng)
            .unwrap();
        for 韵部 in self.元素分类.韵部列表.iter() {
            let 韵母 = 韵部[声调];
            let 当前键 = 映射[韵母];
            let 目标键 = 当前键 + (目标行 - 当前行) * 10;
            assert!(目标键 > 0 && 目标键 < 31);
            映射[韵母] = 目标键;
        }
        self.元素分类.韵部列表.iter().map(|x| x[声调]).collect()
    }

    pub fn 随机交换声调(&self, 映射: &mut 元素映射) -> Vec<元素> {
        let mut rng = thread_rng();
        // 固定轻声
        let 声调: Vec<usize> = [0, 1, 2, 3].choose_multiple(&mut rng, 2).cloned().collect();
        let mut 移动的韵母 = vec![];
        let (声调一, 声调二) = (声调[0], 声调[1]);
        for 韵部 in self.元素分类.韵部列表.iter() {
            let (韵母一, 韵母二) = (韵部[声调一], 韵部[声调二]);
            let (键一, 键二) = (映射[韵母一], 映射[韵母二]);
            映射[韵母一] = 键二;
            映射[韵母二] = 键一;
            移动的韵母.push(韵母一);
            移动的韵母.push(韵母二);
        }
        移动的韵母
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
            .filter(|x| 映射[**x] != 空格)
            .cloned()
            .collect();
        let 是双编码: Vec<_> = self
            .双编码映射
            .keys()
            .filter(|x| 映射[**x] == 空格)
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
            映射[元素] = 空格;
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
                .filter(|x| !self.字根树控制器.查询字根是否被选取(&_映射, x) && 映射[**x] != 空格)
                .collect()
        } else {
            self.元素分类
                .字根列表
                .iter()
                .filter(|x| self.字根树控制器.查询字根是否被选取(&_映射, x) && 映射[**x] != 空格)
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
            if !self.固定字根.contains(被代表字根) {
                映射[*被代表字根] = 目标按键;
            }
        }
        所有被代表字根
    }
}
