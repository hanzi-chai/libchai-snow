use crate::feihua::{
    冰雪飞花上下文, 冰雪飞花决策, 冰雪飞花决策空间, 冰雪飞花安排
};
use chai::{
    contexts::default::默认决策变化, operators::变异, optimizers::决策, 元素图, 棱镜
};
use rand::{
    random, rng,
    seq::{IndexedRandom, IteratorRandom},
};
use std::{collections::VecDeque, iter::zip};

pub struct 冰雪飞花操作 {
    pub 棱镜: 棱镜,
    pub 元素图: 元素图,
    pub 决策空间: 冰雪飞花决策空间,
}

impl 变异 for 冰雪飞花操作 {
    type 决策 = 冰雪飞花决策;

    fn 变异(&mut self, 决策: &mut 冰雪飞花决策) -> 默认决策变化 {
        let r = random::<f64>();
        let mut 变化 = if r < 2.0 {
            self.移动字根(决策)
        } else if r < 0.5 {
            self.交换字根(决策)
        } else if r < 0.9 {
            self.增加字根(决策)
        } else {
            self.删除字根(决策)
        };
        self.传播(&mut 变化, 决策);
        变化
    }
}

impl 冰雪飞花操作 {
    pub fn 新建(上下文: &冰雪飞花上下文) -> Self {
        冰雪飞花操作 {
            棱镜: 上下文.棱镜.clone(),
            元素图: 上下文.元素图.clone(),
            决策空间: 上下文.决策空间.clone(),
        }
    }

    fn 传播(
        &self, 变化: &mut <冰雪飞花决策 as 决策>::变化, 决策: &mut 冰雪飞花决策
    ) {
        // 初始化队列
        let mut 队列 = VecDeque::new();
        for 元素 in 变化
            .增加元素
            .iter()
            .chain(变化.减少元素.iter())
            .chain(变化.移动元素.iter())
        {
            for 下游元素 in self.元素图.get(元素).unwrap_or(&vec![]) {
                if !队列.contains(下游元素) {
                    队列.push_back(下游元素.clone());
                }
            }
        }
        // 传播直到队列为空
        let mut iters = 0;
        while !队列.is_empty() {
            iters += 1;
            if iters > 100 {
                panic!("传播超过 100 次仍未结束，可能出现死循环");
            }
            let 元素 = 队列.pop_front().unwrap();
            let 当前安排 = 决策.元素[元素];
            let mut 合法 = false;
            let mut 新安排列表 = vec![];
            for 条件安排 in &self.决策空间.元素[元素] {
                if 决策.允许(条件安排) {
                    if 条件安排.安排 == 当前安排 {
                        合法 = true;
                        break;
                    }
                    新安排列表.push(条件安排.安排.clone());
                }
            }
            if !合法 {
                if 新安排列表.is_empty() {
                    panic!("没有合法的安排，传播失败");
                } else {
                    let 新安排 = *新安排列表.choose(&mut rng()).unwrap();
                    if let 冰雪飞花安排::未选取 = 当前安排 {
                        变化.增加元素.push(元素);
                    } else if let 冰雪飞花安排::未选取 = 新安排 {
                        变化.减少元素.push(元素);
                    } else {
                        变化.移动元素.push(元素);
                    }
                    决策.元素[元素] = 新安排;
                }
            }
            for 下游元素 in self.元素图.get(&元素).unwrap_or(&vec![]) {
                if !队列.contains(下游元素) {
                    队列.push_back(*下游元素);
                }
            }
        }
    }

    pub fn 移动字根(&self, 决策: &mut 冰雪飞花决策) -> 默认决策变化 {
        let mut r = rng();
        let mut 可行移动 = vec![];
        for ((元素, 当前安排), 安排列表) in
            zip(决策.元素.iter().enumerate(), self.决策空间.元素.iter())
        {
            let 新安排 = 安排列表
                .iter()
                .filter(|x| x.安排 != *当前安排 && x.安排 != 冰雪飞花安排::未选取)
                .choose(&mut r);
            if let Some(新安排) = 新安排 {
                可行移动.push((元素, 新安排.安排));
            }
        }
        let (元素, 新安排) = 可行移动.choose(&mut r).unwrap();
        决策.元素[*元素] = *新安排;
        默认决策变化::新建(vec![], vec![], vec![*元素])
    }

    pub fn 交换字根(&self, 决策: &mut 冰雪飞花决策) -> 默认决策变化 {
        let mut r = rng();
        let mut 可行交换 = vec![];
        for (元素, 当前安排) in 决策.元素.iter().enumerate() {
            if let 冰雪飞花安排::键位(_) = 当前安排 {
                可行交换.push(元素);
            }
        }
        let 交换: Vec<_> = 可行交换.choose_multiple(&mut r, 2).cloned().collect();
        if 交换.len() == 2 {
            let (字根一, 字根二) = (交换[0], 交换[1]);
            决策.元素[字根一] = 决策.元素[字根二];
            决策.元素[字根二] = 决策.元素[字根一];
            默认决策变化::新建(vec![], vec![], vec![字根一, 字根二])
        } else {
            默认决策变化::不变()
        }
    }

    pub fn 增加字根(&self, 决策: &mut 冰雪飞花决策) -> 默认决策变化 {
        let mut r = rng();
        let mut 可行增加 = vec![];
        for ((元素, 当前安排), 安排列表) in
            zip(决策.元素.iter().enumerate(), self.决策空间.元素.iter())
        {
            if 当前安排 == &冰雪飞花安排::未选取 {
                let 新安排 = 安排列表
                    .iter()
                    .filter(|x| x.安排 != 冰雪飞花安排::未选取)
                    .choose(&mut r)
                    .unwrap();
                可行增加.push((元素, 新安排.安排));
            }
        }
        if let Some((元素, 新安排)) = 可行增加.choose(&mut r) {
            决策.元素[*元素] = *新安排;
            默认决策变化::新建(vec![*元素], vec![], vec![])
        } else {
            默认决策变化::不变()
        }
    }

    pub fn 删除字根(&self, 决策: &mut 冰雪飞花决策) -> 默认决策变化 {
        let mut r = rng();
        let mut 可行删除 = vec![];
        for ((元素, 当前安排), 安排列表) in
            zip(决策.元素.iter().enumerate(), self.决策空间.元素.iter())
        {
            if 当前安排 == &冰雪飞花安排::未选取 {
                continue;
            }
            if 安排列表.iter().any(|x| x.安排 == 冰雪飞花安排::未选取) {
                可行删除.push(元素);
            }
        }
        if let Some(元素) = 可行删除.choose(&mut r) {
            决策.元素[*元素] = 冰雪飞花安排::未选取;
            默认决策变化::新建(vec![], vec![*元素], vec![])
        } else {
            默认决策变化::不变()
        }
    }
}
