use crate::feihua::{
    全集合, 冰雪飞花上下文, 冰雪飞花决策, 冰雪飞花安排, 小集合
};
use chai::{operators::变异, 棱镜};
use rand::{random, rng, seq::IndexedRandom};
use regex::Regex;

pub struct 冰雪飞花操作 {
    pub 棱镜: 棱镜,
    pub 可移动元素: Vec<usize>,
}

impl 变异 for 冰雪飞花操作 {
    type 决策 = 冰雪飞花决策;

    fn 变异(&mut self, 决策: &mut 冰雪飞花决策) {
        let mut r = rng();
        let 元素 = *self.可移动元素.choose(&mut r).unwrap();
        if random::<f64>() < 0.3 {
            // 小集合
            let 新位置 = 小集合.choose(&mut r).unwrap();
            决策.元素[元素] = 冰雪飞花安排::键位(self.棱镜.键转数字[新位置] as u8);
        } else {
            // 全集合
            let 新位置 = 全集合.choose(&mut r).unwrap();
            决策.元素[元素] = 冰雪飞花安排::键位(self.棱镜.键转数字[新位置] as u8);
        }
    }
}

impl 冰雪飞花操作 {
    pub fn 新建(上下文: &冰雪飞花上下文) -> Self {
        let mut 可移动元素 = vec![];
        let regex = Regex::new(r"[12345口八丷宀日人亻\ue43d十\ue068艹廾]").unwrap();
        for (元素, 安排) in 上下文.初始决策.元素.iter().enumerate() {
            if 元素 <= 上下文.棱镜.数字转键.len() as usize {
                continue;
            }
            let 元素名称 = &上下文.棱镜.数字转元素[&元素];
            if 元素名称.starts_with("声-") || regex.is_match(&元素名称.as_str()) {
                continue;
            }
            if let 冰雪飞花安排::归并(_) = 安排 {
                continue;
            }
            可移动元素.push(元素);
        }
        冰雪飞花操作 {
            棱镜: 上下文.棱镜.clone(),
            可移动元素,
        }
    }
}
