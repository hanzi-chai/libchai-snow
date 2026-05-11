use crate::feihua::{
    冰雪飞花上下文, 冰雪飞花决策, 冰雪飞花决策空间, 冰雪飞花安排
};
use chai::{
    contexts::default::默认决策变化, operators::变异, optimizers::决策, 元素图, 棱镜
};
use rand::{rng, seq::IndexedRandom};
use std::{collections::VecDeque, iter::zip};

pub struct 冰雪飞花操作 {
    pub 棱镜: 棱镜,
    pub 元素图: 元素图,
    pub 决策空间: 冰雪飞花决策空间,
}

impl 变异 for 冰雪飞花操作 {
    type 决策 = 冰雪飞花决策;

    fn 变异(&mut self, 决策: &mut 冰雪飞花决策) -> 默认决策变化 {
        let mut 变化 = self.均匀变异(决策);
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
                    队列.push_back(*下游元素);
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

    pub fn 均匀变异(&self, 决策: &mut 冰雪飞花决策) -> 默认决策变化 {
        let mut r = rng();
        let mut 所有可行变化 = vec![];
        for ((元素, 当前安排), 安排列表) in
            zip(决策.元素.iter().enumerate(), self.决策空间.元素.iter())
        {
            for 条件安排 in 安排列表.iter() {
                if 条件安排.安排 != *当前安排 && 决策.允许(条件安排) {
                    所有可行变化.push((元素, 条件安排.安排));
                }
            }
        }
        let &(元素, 新安排) = 所有可行变化.choose(&mut r).unwrap();
        let 旧安排 = 决策.元素[元素];
        决策.元素[元素] = 新安排;
        match (旧安排, 新安排) {
            (冰雪飞花安排::未选取, _) => {
                默认决策变化::新建(vec![元素], vec![], vec![])
            }
            (_, 冰雪飞花安排::未选取) => {
                默认决策变化::新建(vec![], vec![元素], vec![])
            }
            _ => 默认决策变化::新建(vec![], vec![], vec![元素]),
        }
    }
}
