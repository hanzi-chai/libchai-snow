use chai::{encoders::编码器, 元素, 棱镜, 编码信息, 错误};
use rustc_hash::FxHashMap;
use std::{iter::zip, vec};

use crate::qingyun::{
    context::冰雪清韵上下文, 一码掩码, 三码掩码, 二码掩码, 冰雪清韵决策, 冰雪清韵决策变化,
    冰雪清韵编码信息, 动态拆分项, 固定拆分项, 大集合, 小集合, 数字字根, 最大码长, 特简码, 空格,
    编码, 进制, 音节信息, 频序, 频率,
};

pub struct 冰雪清韵编码器 {
    pub 固定拆分: Vec<固定拆分项>,
    pub 动态拆分: Vec<动态拆分项>,
    pub 块转数字: FxHashMap<String, usize>,
    pub 数字转块: FxHashMap<usize, String>,
    pub 特简码: Vec<(usize, 编码)>,
    pub 数字字根: Vec<usize>,
    pub 简体空间: Vec<u8>,
    pub 繁体空间: Vec<u8>,
    pub 通打空间: Vec<u8>,
    pub 棱镜: 棱镜,
    pub 当量信息: Vec<f64>,
    pub 全部出简: bool,
    pub 繁体顺序: Vec<usize>,
    pub 简体顺序: Vec<usize>,
    pub 非主动出简组合: Vec<u32>,
    pub 字根字序号: Vec<usize>,
    pub 拼音: Vec<音节信息>,
    pub 当前拆分索引: Vec<[usize; 4]>,
    pub 子问题列表: Vec<出简子问题数据>,
}

const 最大备选长度: usize = 12;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct 队列 {
    pub 数据: [(usize, 频率); 最大备选长度],
    pub 当前索引: usize,
    pub 长度: usize,
    pub 二简: 编码,
}

impl 队列 {
    fn 入队(&mut self, 序号: usize, 频率: 频率) {
        if self.长度 < 最大备选长度 {
            self.数据[self.长度] = (序号, 频率);
            self.长度 += 1;
        }
    }

    fn 出队(&mut self) -> (usize, 频率) {
        let 数据 = self.数据[self.当前索引];
        self.当前索引 += 1;
        数据
    }

    fn 频率(&self) -> 频率 {
        if self.二简 == 0 {
            self.数据[self.当前索引].1
        } else {
            self.数据[self.当前索引].1 + self.数据[self.当前索引 + 1].1
        }
    }

    fn 重置(&mut self) {
        self.数据 = [(0, 0.0); 最大备选长度];
        self.当前索引 = 0;
        self.长度 = 0;
    }
}

#[derive(Debug, Clone, Default)]
pub struct 出简子问题数据 {
    pub 三码全码队列: 队列,
    pub 四码全码队列: [队列; 21],
    pub 一简十重: Vec<编码>,
}

impl 出简子问题数据 {
    fn 最大队列(&mut self) -> &mut 队列 {
        let mut 最大索引 = 0;
        let mut 最大频率 = self.三码全码队列.频率();
        for (索引, 队列) in self.四码全码队列.iter().enumerate() {
            let 频率 = 队列.频率();
            if 频率 > 最大频率 {
                最大频率 = 频率;
                最大索引 = 索引 + 1;
            }
        }
        if 最大索引 == 0 {
            &mut self.三码全码队列
        } else {
            &mut self.四码全码队列[最大索引 - 1]
        }
    }
}

impl 冰雪清韵编码器 {
    pub fn 新建(上下文: &冰雪清韵上下文, 全部出简: bool) -> Result<Self, 错误> {
        let 当量信息 = 上下文
            .棱镜
            .预处理当量信息(&上下文.原始当量信息, 进制.pow(最大码长 as u32) as usize);
        let 编码空间大小 = 进制.pow(最大码长 as u32) as usize;
        let 简体空间 = vec![Default::default(); 编码空间大小];
        let 繁体空间 = vec![Default::default(); 编码空间大小];
        let 简繁通打空间 = vec![Default::default(); 编码空间大小];
        let 数字字根列表: Vec<usize> = 上下文
            .固定拆分
            .iter()
            .enumerate()
            .filter_map(|(序号, 词)| {
                if 数字字根.contains(&词.词) {
                    Some(序号)
                } else {
                    None
                }
            })
            .collect();
        let 特简码列表 = 上下文
            .固定拆分
            .iter()
            .enumerate()
            .filter_map(|(序号, 词)| {
                if let Some((_, 简码)) = 特简码.iter().find(|&&(c, _)| c == 词.词) {
                    Some((序号, 上下文.棱镜.键转数字[简码] as 编码))
                } else {
                    None
                }
            })
            .collect();
        let 非主动出简组合: Vec<_> =
            vec!["p,", "p.", "p/", "y,", "y.", "y/", "ce", "nu", "mu", "xe"]
                .iter()
                .map(|s| {
                    let c1 = s.chars().next().unwrap();
                    let c2 = s.chars().nth(1).unwrap();
                    上下文.棱镜.键转数字[&c1] as u32 + 上下文.棱镜.键转数字[&c2] as u32 * 进制
                })
                .collect();
        let 子问题列表 = vec![出简子问题数据::default(); 大集合.len()];
        Ok(Self {
            动态拆分: 上下文.动态拆分.clone(),
            固定拆分: 上下文.固定拆分.clone(),
            块转数字: 上下文.块转数字.clone(),
            数字转块: 上下文.数字转块.clone(),
            简体空间,
            繁体空间,
            通打空间: 简繁通打空间,
            棱镜: 上下文.棱镜.clone(),
            当量信息,
            特简码: 特简码列表,
            数字字根: 数字字根列表,
            全部出简,
            简体顺序: 上下文.简体顺序.clone(),
            繁体顺序: 上下文.繁体顺序.clone(),
            非主动出简组合,
            字根字序号: Vec::with_capacity((进制 * 进制) as usize),
            拼音: 上下文.拼音.clone(),
            当前拆分索引: vec![[0; 4]; 上下文.动态拆分.len()],
            子问题列表,
        })
    }

    pub fn construct_series(
        &mut self,
        映射: &Vec<编码>,
        拆分序列: &mut [[元素; 4]],
        解: &冰雪清韵决策,
    ) {
        self.select_series_for_block(映射, 解);
        self.update_series(拆分序列);
    }

    pub fn select_series_for_block(&mut self, 映射: &Vec<编码>, _解: &冰雪清韵决策) {
        for (指针, 拆分方式列表) in zip(&mut self.当前拆分索引, &self.动态拆分) {
            // let mut found = false;
            for 拆分方式 in 拆分方式列表.iter() {
                if 拆分方式.iter().all(|&x| x == 0 || 映射[x] != 0) {
                    *指针 = *拆分方式;
                    // found = true;
                    break;
                }
            }
            // if !found {
            //     解._打印(&self.棱镜);
            //     panic!(
            //         "无法为动态拆分项 {:?} 生成编码",
            //         拆分方式列表
            //             .iter()
            //             .map(|x| {
            //                 x.map(|y| {
            //                     if y == 0 {
            //                         "空".to_string()
            //                     } else {
            //                         self.棱镜.数字转元素[&y].clone()
            //                     }
            //                 })
            //             })
            //             .collect::<Vec<_>>()
            //     );
            // }
        }
    }

    pub fn update_series(&self, 拆分序列: &mut [[元素; 4]]) {
        for (序列, 固定拆分项) in zip(拆分序列, &self.固定拆分) {
            *序列 = [0; 4];
            let mut index = 0;
            for 块序号 in 固定拆分项.字块 {
                if 块序号 == usize::MAX {
                    break;
                }
                for 元素 in self.当前拆分索引[块序号] {
                    if 元素 == 0 {
                        break;
                    }
                    序列[index] = 元素;
                    if index <= 2 {
                        index += 1;
                    }
                }
            }
        }
    }

    pub fn reset_space(&mut self) {
        self.简体空间.iter_mut().for_each(|x| {
            *x = 0;
        });
        self.繁体空间.iter_mut().for_each(|x| {
            *x = 0;
        });
        self.通打空间.iter_mut().for_each(|x| {
            *x = 0;
        });
        self.字根字序号.clear();
        for (子问题, 名称) in zip(self.子问题列表.iter_mut(), 大集合) {
            let 一码 = self.棱镜.键转数字[&名称] as u32;
            let 二简列表: Vec<_> = 大集合
                .map(|x| 一码 + self.棱镜.键转数字[&x] as u32 * 进制 + 空格 * 进制 * 进制)
                .into_iter()
                .collect();
            子问题.三码全码队列.重置();
            for (i, 队列) in 子问题.四码全码队列.iter_mut().enumerate() {
                队列.重置();
                队列.二简 = 二简列表[i];
            }
            子问题.一简十重.clear();
            for x in 小集合 {
                let 编码 = 一码 + self.棱镜.键转数字[&x] as u32 * 进制;
                if self.非主动出简组合.contains(&编码) {
                    continue;
                }
                子问题.一简十重.push(编码);
            }
            子问题.一简十重.sort_by(|&x, &y| {
                self.当量信息[x as usize]
                    .partial_cmp(&self.当量信息[y as usize])
                    .unwrap()
            });
        }
    }

    #[inline(always)]
    fn 全码规则(元素序列: &[元素; 4], 映射: &Vec<编码>) -> 编码 {
        let mut code = 映射[元素序列[0]];
        if 元素序列[1] != 0 {
            code &= 一码掩码;
            code += 映射[元素序列[1]] * 进制;
            if 元素序列[2] != 0 {
                code &= 二码掩码;
                code += 映射[元素序列[2]] * 进制 * 进制;
                if 元素序列[3] != 0 {
                    code &= 三码掩码;
                    code += (映射[元素序列[3]] % 进制) * 进制 * 进制 * 进制;
                }
            }
        }
        code
    }

    fn make_full(
        &mut self,
        编码结果: &mut [冰雪清韵编码信息],
        映射: &Vec<编码>,
        拆分序列: &[[元素; 4]],
        决策: &冰雪清韵决策,
    ) {
        self.generate(拆分序列, 映射, 编码结果);
        self.handle_roots(编码结果, 决策);
        self.annotate_duplication(编码结果);
    }

    fn generate(
        &mut self,
        拆分序列: &[[元素; 4]],
        映射: &Vec<编码>,
        编码结果: &mut [冰雪清韵编码信息],
    ) {
        // 生成全码
        for (序列, (序号, 编码信息)) in zip(拆分序列, 编码结果.iter_mut().enumerate())
        {
            编码信息.全码 = Self::全码规则(序列, 映射);
            // 将字根字序号记录下来用于确定最终编码
            if 编码信息.全码 < 进制 * 进制 {
                self.字根字序号.push(序号);
            }
        }
    }

    fn handle_roots(
        &mut self, 编码结果: &mut [冰雪清韵编码信息], 决策: &冰雪清韵决策
    ) {
        self.字根字序号.sort_by_key(|&x| {
            if 编码结果[x].简体 {
                编码结果[x].简体频序
            } else {
                频序::MAX
            }
        });
        let mut 字根字空间 = vec![Default::default(); (进制 * 进制) as usize];
        let 补码键 = self.棱镜.键转数字[&决策.补码键] as 编码;
        for 序号 in &self.字根字序号 {
            let 编码信息 = &mut 编码结果[*序号];
            if self.固定拆分[*序号].通规 != 0 {
                let 占据二码 = if self.数字字根.contains(序号) {
                    true
                } else {
                    (编码信息.简体频序 < 1000 || self.非主动出简组合.contains(&编码信息.全码))
                        && 字根字空间[编码信息.全码 as usize] == 0
                };
                if 占据二码 {
                    字根字空间[编码信息.全码 as usize] = 1;
                } else {
                    编码信息.全码 = 补码键 + 编码信息.全码 * 进制;
                }
            } else {
                编码信息.全码 = 补码键 + 补码键 * 进制 + 编码信息.全码 * 进制 * 进制;
            }
            if 编码信息.简体 {
                编码信息.简体简码 = 编码信息.全码;
                编码信息.完成出简 = true;
            }
        }
    }

    fn annotate_duplication(&mut self, 编码结果: &mut [冰雪清韵编码信息]) {
        // 简体选重标记
        for 索引 in &self.简体顺序 {
            let 编码信息 = &mut 编码结果[*索引];
            编码信息.简体选重 = self.简体空间[编码信息.全码 as usize];
            self.简体空间[编码信息.全码 as usize] =
                self.简体空间[编码信息.全码 as usize].saturating_add(1);
        }
        // 繁体选重标记
        for 索引 in &self.繁体顺序 {
            let 编码信息 = &mut 编码结果[*索引];
            编码信息.繁体选重 = self.繁体空间[编码信息.全码 as usize];
            self.繁体空间[编码信息.全码 as usize] =
                self.繁体空间[编码信息.全码 as usize].saturating_add(1);
        }
        // 简繁选重标记
        for 编码信息 in 编码结果.iter_mut() {
            编码信息.通打选重 = self.通打空间[编码信息.全码 as usize];
            self.通打空间[编码信息.全码 as usize] =
                self.通打空间[编码信息.全码 as usize].saturating_add(1);
        }
    }

    fn make_short(
        &mut self, 编码结果: &mut [冰雪清韵编码信息], _决策: &冰雪清韵决策
    ) {
        for (序号, 编码) in self.特简码.iter().copied() {
            编码结果[序号].简体简码 = 编码;
            编码结果[序号].完成出简 = true;
        }
        const 声码位移: usize = 1;
        for 序号 in &self.简体顺序 {
            let 编码信息 = &mut 编码结果[*序号];
            if 编码信息.简体频序 >= 3000 {
                编码信息.简体简码 = 编码信息.全码;
                continue;
            }
            // 跳过已经处理的优先简码
            if 编码信息.完成出简 {
                编码信息.完成出简 = false;
                if 编码信息.简体简码 > 进制 && 编码信息.简体简码 < 进制 * 进制
                {
                    let 第一码 = (编码信息.简体简码 % 进制) as usize - 声码位移;
                    self.子问题列表[第一码]
                        .一简十重
                        .retain(|&x| x != 编码信息.简体简码);
                }
                continue;
            } else if 编码信息.全码 < 进制 * 进制 * 进制 {
                // 二根字
                let 第一码 = (编码信息.全码 % 进制) as usize - 声码位移;
                self.子问题列表[第一码]
                    .三码全码队列
                    .入队(*序号, 编码信息.简体频率);
            } else {
                // 三根以上字
                let 第一码 = (编码信息.全码 % 进制) as usize - 声码位移;
                let 第二码 = ((编码信息.全码 / 进制) % 进制) as usize - 声码位移;
                self.子问题列表[第一码].四码全码队列[第二码].入队(*序号, 编码信息.简体频率);
            }
            编码信息.简体简码 = 编码信息.全码;
        }
        for 子问题 in self.子问题列表.iter_mut() {
            while !子问题.一简十重.is_empty() {
                let 一级简码 = 子问题.一简十重.remove(0);
                let 队列 = 子问题.最大队列();
                let (序号, _) = 队列.出队();
                编码结果[序号].简体简码 = 一级简码;
            }
            // 输出二级简码
            for 队列 in 子问题.四码全码队列.iter() {
                if 队列.二简 != 0 {
                    let (序号, _) = 队列.数据[队列.当前索引];
                    if 序号 != 0 {
                        编码结果[序号].简体简码 = 队列.二简;
                    }
                }
            }
        }
    }

    fn 输出三级简码(&mut self, 编码结果: &mut [冰雪清韵编码信息]) {
        let 三码 = 进制 * 进制 * 进制;
        for 序号 in &self.简体顺序 {
            let 编码信息 = &mut 编码结果[*序号];
            if 编码信息.简体简码 != 编码信息.全码
                || 编码信息.全码 <= 三码
                || 编码信息.全码 >= 三码 * 空格
            {
                continue;
            }
            let 三级简码 = 编码信息.全码 % 三码 + 三码 * 空格;
            if self.简体空间[三级简码 as usize] == 0 {
                编码信息.简体简码 = 三级简码;
                self.简体空间[三级简码 as usize] = 1;
            }
        }
    }

    pub fn make_pinyin(&mut self, 映射: &Vec<编码>, 音码空间: &mut Vec<频率>) {
        音码空间.iter_mut().for_each(|x| {
            *x = 0.0;
        });
        for 音节信息 {
            声母, 韵母, 频率
        } in &self.拼音
        {
            let 声码 = 映射[*声母];
            let 韵码 = 映射[*韵母];
            let 音码 = 声码 + 韵码 * 进制;
            音码空间[音码 as usize] += *频率;
        }
    }

    pub fn dynamic_encode(
        &mut self,
        决策: &冰雪清韵决策,
        映射: &Vec<编码>,
        拆分序列: &[[元素; 4]],
        输出: &mut [冰雪清韵编码信息],
        音码输出: &mut Vec<频率>,
    ) {
        self.reset_space();
        self.make_full(输出, &映射, 拆分序列, 决策);
        self.make_short(输出, 决策);
        self.make_pinyin(&映射, 音码输出);
        if self.全部出简 {
            self.输出三级简码(输出);
        }
    }
}

impl 编码器 for 冰雪清韵编码器 {
    type 解类型 = 冰雪清韵决策;
    fn 编码(
        &mut self,
        _决策: &冰雪清韵决策,
        _决策变化: &Option<冰雪清韵决策变化>,
        _输出: &mut [编码信息],
    ) {
        self.reset_space();
    }
}
