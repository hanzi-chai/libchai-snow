use chai::{encoders::编码器, 元素, 棱镜, 编码信息, 错误};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use serde_yaml::from_str;
use std::{cmp::Reverse, collections::BinaryHeap, fs::read_to_string, iter::zip, vec};

use crate::qingyun::{
    context::冰雪清韵上下文, 冰雪清韵决策, 冰雪清韵决策变化, 冰雪清韵编码信息, 动态拆分项, 双键,
    固定拆分项, 大集合, 小集合, 无空格, 映射, 特简码, 空格, 编码, 转换, 进制, 键, 音节信息, 频序,
    频率,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 简码覆盖 {
    pub 一简: FxHashMap<char, String>,
    pub 字根: Vec<char>,
    pub 简词快符: FxHashMap<String, String>,
}

pub struct 冰雪清韵编码器 {
    pub 固定拆分: Vec<固定拆分项>,
    pub 动态拆分: Vec<动态拆分项>,
    pub 块转数字: FxHashMap<String, usize>,
    pub 数字转块: FxHashMap<usize, String>,
    pub 特简码: Vec<(usize, 编码)>,
    pub 简体空间: Vec<bool>,
    pub 繁体空间: Vec<bool>,
    pub 通打空间: Vec<u8>,
    pub 棱镜: 棱镜,
    pub 当量信息: Vec<f32>,
    pub 全部出简: bool,
    pub 繁体顺序: Vec<usize>,
    pub 简体顺序: Vec<usize>,
    pub 非主动出简组合: Vec<编码>,
    pub 固定占用组合: Vec<编码>,
    pub 拼音: Vec<音节信息>,
    pub 当前拆分索引: Vec<[usize; 4]>,
    pub 子问题列表: Vec<出简子问题数据>,
    pub 编码结果: Vec<冰雪清韵编码信息>,
    pub 拆分序列: Vec<[元素; 4]>,
    pub 拆分关联映射: FxHashMap<元素, Vec<usize>>,
    pub 音码空间: Vec<f32>,
    pub 字根字序号: Vec<usize>,
    pub 简码覆盖: 简码覆盖,
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
        if self.二简 == 编码::default() {
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
        let 编码空间大小 = 编码::编码空间大小();
        let 当量信息 = 上下文.预处理当量信息();
        let 简体空间 = vec![Default::default(); 编码空间大小];
        let 繁体空间 = vec![Default::default(); 编码空间大小];
        let 简繁通打空间 = vec![Default::default(); 编码空间大小];
        let 特简码列表 = 上下文
            .固定拆分
            .iter()
            .enumerate()
            .filter_map(|(序号, 词)| {
                if let Some((_, 简码)) = 特简码.iter().find(|&&(c, _)| c == 词.词) {
                    let k = 上下文.棱镜.键转数字[简码] as 键;
                    Some((序号, [0, 0, 0, k]))
                } else {
                    None
                }
            })
            .collect();
        let mut 非主动出简组合 = vec![];
        let mut 固定占用组合 = vec![];
        for 组合 in vec!["p,", "p.", "p/", "y,", "y.", "y/", "ce", "nu", "mu", "xe"] {
            let c1 = 组合.chars().next().unwrap();
            let c2 = 组合.chars().nth(1).unwrap();
            let k1 = 上下文.棱镜.键转数字[&c1] as 键;
            let k2 = 上下文.棱镜.键转数字[&c2] as 键;
            非主动出简组合.push([0, 0, k1, k2]);
        }
        let 简码覆盖: 简码覆盖 = from_str(&read_to_string("data/override.yaml")?).unwrap();
        for 组合 in 简码覆盖.简词快符.values().chain(简码覆盖.一简.values()) {
            let c1 = 组合.chars().next().unwrap();
            let c2 = 组合.chars().nth(1).unwrap();
            let k1 = 上下文.棱镜.键转数字[&c1] as 键;
            let k2 = 上下文.棱镜.键转数字[&c2] as 键;
            固定占用组合.push([0, 0, k1, k2]);
        }

        let 子问题列表 = vec![出简子问题数据::default(); 大集合.len()];
        let 拆分序列 = vec![<[元素; 4]>::default(); 上下文.固定拆分.len()];
        let 编码结果: Vec<_> = 上下文
            .固定拆分
            .iter()
            .map(|x| 冰雪清韵编码信息 {
                简体: x.gb2312,
                繁体: x.国字常用 || x.陆标,
                简体频率: x.简体频率,
                简体指数频率: ((x.简体频序.min(6000) as 频率) / -1500.0).exp(),
                简体频序: x.简体频序,
                繁体频率: x.繁体频率,
                繁体指数频率: ((x.繁体频序.min(6000) as 频率) / -1000.0).exp(),
                繁体频序: x.繁体频序,
                通打频率: x.通打频率,
                全码: Default::default(),
                计重全码: Default::default(),
                计重索引: Default::default(),
                简体简码: Default::default(),
                特简: 特简码.iter().any(|(c, _)| c == &x.词),
                字根字: false,
            })
            .collect();
        Ok(Self {
            动态拆分: 上下文.动态拆分.clone(),
            固定拆分: 上下文.固定拆分.clone(),
            块转数字: 上下文.块转数字.clone(),
            数字转块: 上下文.数字转块.clone(),
            简体空间,
            繁体空间,
            编码结果,
            拆分序列,
            通打空间: 简繁通打空间,
            棱镜: 上下文.棱镜.clone(),
            当量信息,
            特简码: 特简码列表,
            全部出简,
            简体顺序: 上下文.简体顺序.clone(),
            繁体顺序: 上下文.繁体顺序.clone(),
            非主动出简组合,
            固定占用组合,
            拼音: 上下文.拼音.clone(),
            当前拆分索引: vec![[0; 4]; 上下文.动态拆分.len()],
            拆分关联映射: Self::构建拆分关联映射(&上下文.固定拆分, &上下文.动态拆分),
            子问题列表,
            音码空间: vec![0.0; (进制 as usize).pow(2)],
            字根字序号: Vec::with_capacity((进制 as usize).pow(2)),
            简码覆盖,
        })
    }

    fn 构建拆分关联映射(
        固定拆分: &Vec<固定拆分项>,
        动态拆分: &Vec<动态拆分项>,
    ) -> FxHashMap<元素, Vec<usize>> {
        let mut 拆分关联映射: FxHashMap<元素, Vec<usize>> = Default::default();
        for (序号, 固定拆分项) in 固定拆分.iter().enumerate() {
            let mut 元素集合 = FxHashSet::default();
            for &块 in 固定拆分项.字块.iter() {
                if 块 == usize::MAX {
                    continue;
                }
                let 拆分方式列表 = &动态拆分[块];
                for 拆分方式 in 拆分方式列表.iter() {
                    for &元素 in 拆分方式.iter() {
                        if 元素 != 0 {
                            元素集合.insert(元素);
                        }
                    }
                }
            }
            for 元素 in 元素集合 {
                拆分关联映射.entry(元素).or_insert_with(Vec::new).push(序号);
            }
        }
        拆分关联映射
    }

    fn 合并关联(&self, 变化: &冰雪清韵决策变化, include_move: bool) -> Vec<usize> {
        // 收集切片（只读），并统计总长度用于预分配
        let mut lists: Vec<&[usize]> = vec![];
        let mut total_len = 0usize;
        let mut it: Vec<_> = 变化
            .增加字根
            .iter()
            .chain(变化.减少字根.iter())
            .cloned()
            .collect();
        if include_move {
            it.extend(变化.移动字根.iter().cloned())
        };
        for key in it {
            if let Some(v) = self.拆分关联映射.get(&key) {
                if !v.is_empty() {
                    total_len += v.len();
                    lists.push(v.as_slice());
                }
            }
        }
        if lists.is_empty() {
            return Vec::new();
        }

        // (value, list_idx, elem_idx) 的最小堆（用 Reverse 包装）
        let mut heap: BinaryHeap<(Reverse<usize>, usize, usize)> =
            BinaryHeap::with_capacity(lists.len());
        for (i, lst) in lists.iter().enumerate() {
            // lst 至少有 1 个元素
            heap.push((Reverse(lst[0]), i, 0));
        }

        // 结果向量。上界不超过 total_len（去重后会更小）
        let mut out = Vec::with_capacity(total_len);
        // 主循环
        while let Some((Reverse(val), li, ei)) = heap.pop() {
            // 写出前去重：只当与上一个输出不同才 push
            if out.last().map_or(true, |&last| last < val) {
                out.push(val);
            }
            // 推进该列表
            let next_idx = ei + 1;
            if next_idx < lists[li].len() {
                heap.push((Reverse(lists[li][next_idx]), li, next_idx));
            }

            // 额外优化：把堆顶与 out.last() 相同的跨列表重复项一并弹出并推进
            while let Some((Reverse(topv), _, _)) = heap.peek() {
                if *topv == *out.last().unwrap() {
                    let (_, li2, ei2) = heap.pop().unwrap();
                    let ni = ei2 + 1;
                    if ni < lists[li2].len() {
                        heap.push((Reverse(lists[li2][ni]), li2, ni));
                    }
                } else {
                    break;
                }
            }
        }

        out
    }

    pub fn construct_series(
        &mut self,
        映射: &映射,
        解: &冰雪清韵决策,
        变化: Option<&冰雪清韵决策变化>,
    ) {
        self.select_series_for_block(映射, 解);
        self.update_series(变化);
    }

    pub fn select_series_for_block(&mut self, 映射: &映射, _解: &冰雪清韵决策) {
        for (指针, 拆分方式列表) in zip(&mut self.当前拆分索引, &self.动态拆分) {
            // let mut found = false;
            for 拆分方式 in 拆分方式列表.iter() {
                if 拆分方式
                    .iter()
                    .all(|&x| x == 0 || 映射[x] != 双键::default())
                {
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

    #[inline(always)]
    fn 合并字块拆分(
        拆分索引: &Vec<[元素; 4]>, 拆分项: &固定拆分项
    ) -> [元素; 4] {
        let mut 结果 = [0; 4];
        let mut index = 0;
        for &块序号 in 拆分项.字块.iter() {
            if 块序号 == usize::MAX {
                break;
            }
            for 元素 in 拆分索引[块序号] {
                if 元素 == 0 {
                    break;
                }
                结果[index] = 元素;
                if index <= 2 {
                    index += 1;
                }
            }
        }
        结果
    }

    pub fn update_series(&mut self, 变化: Option<&冰雪清韵决策变化>) {
        if let Some(变化) = 变化 {
            let 相关拆分 = self.合并关联(变化, false);
            for &序号 in &相关拆分 {
                self.拆分序列[序号] =
                    Self::合并字块拆分(&self.当前拆分索引, &self.固定拆分[序号]);
            }
        } else {
            for (序列, 固定拆分项) in zip(&mut self.拆分序列, &self.固定拆分) {
                *序列 = Self::合并字块拆分(&self.当前拆分索引, 固定拆分项);
            }
        }
    }

    pub fn reset_space(&mut self) {
        self.字根字序号.clear();
        self.简体空间.iter_mut().for_each(|x| {
            *x = false;
        });
        self.繁体空间.iter_mut().for_each(|x| {
            *x = false;
        });
        self.通打空间.iter_mut().for_each(|x| {
            *x = 0;
        });
        for (子问题, 名称) in zip(self.子问题列表.iter_mut(), 大集合) {
            let 一码 = self.棱镜.键转数字[&名称] as 键;
            let 二简列表: Vec<_> = 大集合
                .map(|x| [0, 一码, self.棱镜.键转数字[&x] as 键, 空格])
                .into_iter()
                .collect();
            子问题.三码全码队列.重置();
            for (i, 队列) in 子问题.四码全码队列.iter_mut().enumerate() {
                队列.重置();
                队列.二简 = 二简列表[i];
                if 无空格 {
                    队列.二简 = Default::default();
                }
            }
            子问题.一简十重.clear();
            for x in 小集合 {
                if 无空格 && x == '_' {
                    continue;
                }
                let 编码 = [0, 0, 一码, self.棱镜.键转数字[&x] as 键];
                if self.非主动出简组合.contains(&编码) || self.固定占用组合.contains(&编码) {
                    continue;
                }
                子问题.一简十重.push(编码);
            }
            子问题.一简十重.sort_by(|&x, &y| {
                self.当量信息[x.to_usize()]
                    .partial_cmp(&self.当量信息[y.to_usize()])
                    .unwrap()
            });
        }
    }

    #[inline(always)]
    fn 全码规则(元素序列: &[元素; 4], 映射: &映射) -> 编码 {
        let mut 编码 = [0; 4];
        unsafe {
            if 元素序列[1] != 0 {
                if 元素序列[2] != 0 {
                    编码[0] = 映射.get_unchecked(元素序列[0]).0;
                    编码[1] = 映射.get_unchecked(元素序列[1]).0;
                    if 元素序列[3] != 0 {
                        编码[2] = 映射.get_unchecked(元素序列[2]).0;
                        编码[3] = 映射.get_unchecked(元素序列[3]).0;
                    } else {
                        (编码[2], 编码[3]) = *映射.get_unchecked(元素序列[2]);
                    }
                } else {
                    编码[1] = 映射.get_unchecked(元素序列[0]).0;
                    (编码[2], 编码[3]) = *映射.get_unchecked(元素序列[1]);
                }
            } else {
                (编码[2], 编码[3]) = *映射.get_unchecked(元素序列[0]);
            }
        }
        编码
    }

    fn make_full(
        &mut self, 映射: &映射, 决策: &冰雪清韵决策, 变化: Option<&冰雪清韵决策变化>
    ) {
        self.generate(映射, 变化);
        self.handle_roots(决策);
    }

    fn generate(&mut self, 映射: &映射, _变化: Option<&冰雪清韵决策变化>) {
        for (i, (编码信息, 序列)) in zip(self.编码结果.iter_mut(), &self.拆分序列).enumerate()
        {
            编码信息.全码 = Self::全码规则(序列, 映射);
            编码信息.计重全码 = 编码信息.全码;
            编码信息.计重索引 = 编码信息.全码.to_usize();
            编码信息.字根字 = 序列[1] == 0;
            if 编码信息.字根字 {
                self.字根字序号.push(i);
            }
        }
    }

    fn handle_roots(&mut self, 决策: &冰雪清韵决策) {
        self.字根字序号.sort_by_key(|&x| {
            if self.编码结果[x].简体 {
                self.编码结果[x].简体频序
            } else {
                频序::MAX
            }
        });
        let mut 字根字空间 = vec![Default::default(); 进制 as usize * 进制 as usize];
        for 编码 in &self.固定占用组合 {
            字根字空间[编码.to_usize()] = 1;
        }
        let 补码键 = self.棱镜.键转数字[&决策.补码键] as 键;
        for 序号 in &self.字根字序号 {
            if self.简码覆盖.字根.contains(&self.固定拆分[*序号].词) {
                let 编码信息 = &mut self.编码结果[*序号];
                字根字空间[编码信息.计重索引] = 1;
                编码信息.简体简码 = 编码信息.计重全码;
            }
        }
        for 序号 in &self.字根字序号 {
            if self.简码覆盖.字根.contains(&self.固定拆分[*序号].词) {
                continue;
            }
            let 编码信息 = &mut self.编码结果[*序号];
            if self.固定拆分[*序号].通规 && 编码信息.简体 {
                let 占据二码 = (编码信息.简体频序 < 1000
                    || self.非主动出简组合.contains(&编码信息.全码))
                    && 字根字空间[编码信息.计重索引] == 0;
                if 占据二码 {
                    字根字空间[编码信息.计重索引] = 1;
                } else {
                    编码信息.计重全码[1] = 补码键;
                    println!("字根字 {} 无法使用原码，改为补码", self.固定拆分[*序号].词);
                }
            } else {
                编码信息.计重全码[0] = 补码键;
                编码信息.计重全码[1] = 补码键;
            }
            if 编码信息.简体 {
                编码信息.简体简码 = 编码信息.计重全码;
            }
            编码信息.计重索引 = 编码信息.计重全码.to_usize();
        }
    }

    fn make_short(&mut self, _决策: &冰雪清韵决策) {
        for (序号, 编码) in self.特简码.iter().copied() {
            self.编码结果[序号].简体简码 = 编码;
        }
        const 声码位移: usize = 1;
        for (字, 简码字符串) in &self.简码覆盖.一简 {
            if let Some(序号) = self.固定拆分.iter().position(|x| &x.词 == 字) {
                let 简码: Vec<_> = 简码字符串
                    .chars()
                    .map(|c| self.棱镜.键转数字[&c] as 键)
                    .collect();
                let 编码 = [0, 0, 简码[0], 简码[1]];
                self.编码结果[序号].简体简码 = 编码;
                self.编码结果[序号].特简 = true;
                self.子问题列表[简码[0] as usize - 声码位移]
                    .一简十重
                    .retain(|&x| x != 编码);
            }
        }
        for 序号 in &self.简体顺序 {
            let 编码信息 = &mut self.编码结果[*序号];
            // 跳过已经处理的优先简码
            if 编码信息.特简 || 编码信息.字根字 {
                if 编码信息.简体简码[2] != 0 && 编码信息.简体简码[1] == 0 {
                    let 第一码 = 编码信息.简体简码[2] as usize - 声码位移;
                    self.子问题列表[第一码]
                        .一简十重
                        .retain(|&x| x != 编码信息.简体简码);
                }
                continue;
            } else if 编码信息.简体频序 >= 3000 {
                编码信息.简体简码 = 编码信息.全码;
                continue;
            } else if 编码信息.全码[0] == 0 {
                // 二根字
                let 第一码 = 编码信息.全码[1] as usize - 声码位移;
                self.子问题列表[第一码]
                    .三码全码队列
                    .入队(*序号, 编码信息.简体频率);
            } else {
                // 三根以上字
                let 第一码 = 编码信息.全码[0] as usize - 声码位移;
                let 第二码 = 编码信息.全码[1] as usize - 声码位移;
                self.子问题列表[第一码].四码全码队列[第二码].入队(*序号, 编码信息.简体频率);
            }
            编码信息.简体简码 = 编码信息.全码;
        }
        for 子问题 in self.子问题列表.iter_mut() {
            while !子问题.一简十重.is_empty() {
                let 一级简码 = 子问题.一简十重.remove(0);
                let 队列 = 子问题.最大队列();
                let (序号, _) = 队列.出队();
                self.编码结果[序号].简体简码 = 一级简码;
            }
            if 无空格 {
                continue;
            }
            // 输出二级简码
            for 队列 in 子问题.四码全码队列.iter() {
                if 队列.二简 != 编码::default() {
                    let (序号, _) = 队列.数据[队列.当前索引];
                    if 序号 != 0 {
                        self.编码结果[序号].简体简码 = 队列.二简;
                    }
                }
            }
        }
    }

    fn 输出三级简码(&mut self) {
        for 序号 in &self.简体顺序 {
            let 编码信息 = &mut self.编码结果[*序号];
            if 编码信息.简体简码 != 编码信息.全码
                || 编码信息.全码[0] == 0
                || 编码信息.全码[3] >= 空格
                || 编码信息.简体频序 >= 1500
            {
                continue;
            }
            let mut 三级简码 = 编码信息.全码;
            三级简码[3] = 空格;
            if !self.简体空间[三级简码.to_usize()] {
                编码信息.简体简码 = 三级简码;
                self.简体空间[三级简码.to_usize()] = true;
            }
        }
    }

    pub fn make_pinyin(&mut self, 映射: &映射) {
        self.音码空间.iter_mut().for_each(|x| {
            *x = 0.0;
        });
        for 音节信息 {
            声母, 韵母, 频率
        } in &self.拼音
        {
            let 声码 = 映射[*声母].0;
            let 韵码 = 映射[*韵母].0;
            let 音码 = [0, 0, 声码, 韵码];
            self.音码空间[音码.to_usize()] += *频率;
        }
    }
}

impl 编码器 for 冰雪清韵编码器 {
    type 解类型 = 冰雪清韵决策;
    fn 编码(
        &mut self,
        决策: &冰雪清韵决策,
        决策变化: &Option<冰雪清韵决策变化>,
        _输出: &mut [编码信息],
    ) {
        let 映射 = 决策.线性化(&self.棱镜);
        if let Some(变化) = 决策变化 {
            if 变化.增加字根.len() > 0 || 变化.减少字根.len() > 0 {
                self.construct_series(&映射, 决策, Some(变化));
            }
        } else {
            self.construct_series(&映射, 决策, None);
        };
        self.reset_space();
        self.make_full(&映射, 决策, 决策变化.as_ref());
        self.make_short(决策);
        self.make_pinyin(&映射);
        if self.全部出简 {
            self.输出三级简码();
        }
    }
}
