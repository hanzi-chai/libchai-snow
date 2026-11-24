//! 局部拓扑签名只在预处理使用；运行时比较全局拆分编号。
use super::{检查, 结果};

pub(super) struct 冲突索引 {
    pub 起点: Vec<usize>,
    pub 所属: Vec<usize>,
    pub 掩码: Vec<Vec<(usize, u64)>>,
}

impl 冲突索引 {
    pub fn 新建(数量: impl Iterator<Item = usize>, 类表: &[Vec<Vec<usize>>]) -> 结果<Self> {
        let mut 起点 = Vec::new();
        let mut 所属 = Vec::new();
        for (件, 数) in 数量.enumerate() {
            起点.push(所属.len());
            所属.extend(std::iter::repeat_n(件, 数));
        }
        let 总数 = 所属.len();
        let 字数 = 总数.div_ceil(64);
        let mut 矩阵 = vec![vec![0u64; 字数]; 总数];
        for 类 in 类表 {
            for (i, 组) in 类.iter().enumerate() {
                let mut 异组 = vec![0; 字数];
                for (_, 他组) in 类.iter().enumerate().filter(|(j, _)| *j != i) {
                    for &号 in 他组 {
                        检查(号 < 总数, "局部约束的拆分编号越界")?;
                        异组[号 / 64] |= 1 << (号 % 64);
                    }
                }
                for &号 in 组 {
                    检查(号 < 总数, "局部约束的拆分编号越界")?;
                    for (块, &位) in 矩阵[号].iter_mut().zip(&异组) {
                        *块 |= 位;
                    }
                }
            }
        }
        // 只比较不同部件。同一部件同一拆分中可以出现两种局部拆法。
        for (号, 行) in 矩阵.iter_mut().enumerate() {
            let 件 = 所属[号];
            for 同件 in 起点[件]..起点.get(件 + 1).copied().unwrap_or(总数) {
                行[同件 / 64] &= !(1 << (同件 % 64));
            }
        }
        let 掩码 = 矩阵
            .into_iter()
            .map(|行| {
                行.into_iter()
                    .enumerate()
                    .filter(|&(_, 位)| 位 != 0)
                    .collect()
            })
            .collect();
        Ok(Self {
            起点, 所属, 掩码
        })
    }

    pub fn 空集(&self) -> Vec<u64> {
        vec![0; self.所属.len().div_ceil(64)]
    }

    pub fn 互斥(&self, 甲: usize, 乙: usize) -> bool {
        let 行 = &self.掩码[甲];
        行.binary_search_by_key(&(乙 / 64), |&(块, _)| 块)
            .is_ok_and(|i| 行[i].1 & (1 << (乙 % 64)) != 0)
    }

    pub fn 相容(&self, 号: usize, 已选: &[u64]) -> bool {
        self.掩码[号].iter().all(|&(块, 位)| 已选[块] & 位 == 0)
    }
}
