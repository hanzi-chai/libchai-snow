use crate::{
    common::转换,
    feihua::{
        冰雪飞花上下文, 冰雪飞花决策, 冰雪飞花汉字信息, 动态拆分项, 小, 线性化决策, 编码
    },
};
use chai::{encoders::编码器, 元素, 棱镜, 编码信息};
use std::iter::zip;

pub struct 冰雪飞花编码信息 {
    pub 全码: 编码,
    pub 简码: 编码,
    pub 频率: u64,
    pub 指数频率: f64,
    pub 选重: bool,
    pub 候选位置: u8,
}

pub struct 冰雪飞花编码器 {
    pub 汉字信息: Vec<冰雪飞花汉字信息>,
    pub 动态拆分: Vec<动态拆分项>,
    pub 拆分序列: Vec<[元素; 4]>,
    pub 棱镜: 棱镜,
    pub 编码结果: Vec<冰雪飞花编码信息>,
    pub 编码空间: Vec<u8>,
}

impl 冰雪飞花编码器 {
    pub fn 新建(上下文: &冰雪飞花上下文) -> Self {
        let 编码结果 = 上下文
            .信息列表
            .iter()
            .enumerate()
            .map(|(索引, 词)| 冰雪飞花编码信息 {
                全码: 编码::default(),
                简码: 编码::default(),
                频率: 词.频率,
                指数频率: ((索引.min(6000) as f64) / -2000.0).exp(),
                选重: false,
                候选位置: 0,
            })
            .collect();
        Self {
            汉字信息: 上下文.信息列表.clone(),
            动态拆分: 上下文.动态拆分.clone(),
            拆分序列: 上下文
                .信息列表
                .iter()
                .map(|x| [x.声母, x.部首, 0, 0])
                .collect(),
            棱镜: 上下文.棱镜.clone(),
            编码空间: vec![0; 编码::编码空间大小()],
            编码结果,
        }
    }
}

impl 冰雪飞花编码器 {
    pub fn 刷新拆分序列(&mut self, 决策: &线性化决策, _变化: &Option<()>) {
        let mut 当前拆分索引 = vec![Default::default(); self.动态拆分.len()];
        for (指针, 拆分方式列表) in zip(&mut 当前拆分索引, &self.动态拆分) {
            let mut found = false;
            for 拆分方式 in 拆分方式列表.iter() {
                // 找到一个所有字根都在小集合内的拆分方式
                if 拆分方式.iter().all(|&x| {
                    if x == 0 {
                        return true;
                    }
                    let c = 决策[x] as usize;
                    c <= 小 && c != 0
                }) {
                    *指针 = *拆分方式;
                    found = true;
                    break;
                }
            }
            if !found {
                panic!(
                    "无法为拆分项 {:?} 找到合适的拆分方式",
                    拆分方式列表
                        .iter()
                        .map(|x| x.map(|y| if y == 0 {
                            "".into()
                        } else {
                            self.棱镜.数字转元素[&y].clone()
                        }))
                        .collect::<Vec<_>>()
                );
            }
        }
        for (输出, 信息) in zip(&mut self.拆分序列, &self.汉字信息) {
            let 拆分 = if 信息.字块[1] == usize::MAX {
                当前拆分索引[信息.字块[0]]
            } else {
                let 拆分一 = 当前拆分索引[信息.字块[0]];
                let 拆分二 = 当前拆分索引[信息.字块[1]];
                [拆分一[0], 拆分二[0], 拆分二[1]]
            };
            if 信息.部首 == 0 {
                (输出[1], 输出[2], 输出[3]) = (拆分[0], 拆分[1], 拆分[2]);
            } else {
                (输出[2], 输出[3]) = (拆分[0], 拆分[1]);
            }
        }
    }

    pub fn 重置空间(&mut self) {
        self.编码空间.iter_mut().for_each(|x| {
            *x = 0;
        });
    }

    #[inline(always)]
    fn 全码规则(元素序列: &[元素; 4], 映射: &线性化决策) -> 编码 {
        编码([
            映射[元素序列[0]],
            映射[元素序列[1]],
            映射[元素序列[2]],
            映射[元素序列[3]],
        ])
    }

    pub fn 生成全码(&mut self, 决策: &线性化决策) {
        for (输出, 序列) in zip(&mut self.编码结果, &self.拆分序列) {
            输出.全码 = Self::全码规则(序列, 决策);
            let hash = 输出.全码.hash();
            if hash >= self.编码空间.len() {
                panic!(
                    "编码：{:?} 超出编码空间范围，当前序列：{:?}",
                    输出.全码, 序列
                );
            }
            输出.选重 = self.编码空间[hash] > 0;
            输出.候选位置 = self.编码空间[hash];
            self.编码空间[hash] += 1;
        }
    }

    // pub fn 生成简码(&mut self) {
    //     for 输出 in &mut self.编码结果 {
    //         输出.简码 = 输出.全码;
    //     }
    // }
}

impl 编码器 for 冰雪飞花编码器 {
    type 决策 = 冰雪飞花决策;

    fn 编码(&mut self, 决策: &冰雪飞花决策, 变化: &Option<()>, _: &mut [编码信息]) {
        self.重置空间();
        let 决策 = 决策.线性化(&self.棱镜);
        self.刷新拆分序列(&决策, 变化);
        self.生成全码(&决策);
        // self.生成简码();
    }
}
