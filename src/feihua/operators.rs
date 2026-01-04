use crate::feihua::{
    冰雪飞花上下文, 冰雪飞花决策, 冰雪飞花决策空间, 冰雪飞花安排
};
use chai::{operators::变异, 棱镜};
use rand::{
    random, rng,
    seq::{IndexedRandom, IteratorRandom},
};
use std::iter::zip;

pub struct 冰雪飞花操作 {
    pub 棱镜: 棱镜,
    pub 决策空间: 冰雪飞花决策空间,
}

impl 变异 for 冰雪飞花操作 {
    type 决策 = 冰雪飞花决策;

    fn 变异(&mut self, 决策: &mut 冰雪飞花决策) {
        let _r = random::<f64>();
        self.移动字根(决策);
        // if r < 2.0 {
        //     self.移动字根(决策);
        // } else {
        //     self.交换字根(决策);
        // } else if r < 0.9 {
        //     self.增加字根(决策);
        // } else {
        //     self.删除字根(决策);
        // }
    }
}

impl 冰雪飞花操作 {
    pub fn 新建(上下文: &冰雪飞花上下文) -> Self {
        冰雪飞花操作 {
            棱镜: 上下文.棱镜.clone(),
            决策空间: 上下文.决策空间.clone(),
        }
    }

    pub fn 移动字根(&self, 决策: &mut 冰雪飞花决策) {
        let mut r = rng();
        let mut 可行移动 = vec![];
        for ((元素, 当前安排), 安排列表) in
            zip(决策.元素.iter().enumerate(), self.决策空间.元素空间.iter())
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
    }

    pub fn 交换字根(&self, 决策: &mut 冰雪飞花决策) {
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
        }
    }

    pub fn 增加字根(&self, 决策: &mut 冰雪飞花决策) {
        let mut r = rng();
        let mut 可行增加 = vec![];
        for ((元素, 当前安排), 安排列表) in
            zip(决策.元素.iter().enumerate(), self.决策空间.元素空间.iter())
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
        }
    }

    pub fn 删除字根(&self, 决策: &mut 冰雪飞花决策) {
        let mut r = rng();
        let mut 可行删除 = vec![];
        for ((元素, 当前安排), 安排列表) in
            zip(决策.元素.iter().enumerate(), self.决策空间.元素空间.iter())
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
        }
    }
}
