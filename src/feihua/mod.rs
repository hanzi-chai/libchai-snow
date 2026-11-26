//! 飞花专用增量编码与并行退火。预处理证明结构不变，热循环只处理整数索引。
mod consistency;
use chai::config::{安排 as 配置安排, 广义码位, 求解器配置, 配置};
use consistency::冲突索引;
use rand::{Rng, SeedableRng, rngs::SmallRng};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
    time::Instant,
};

pub type 错误 = Box<dyn Error + Send + Sync>;
type 结果<T> = Result<T, 错误>;
const 键表: &[u8] = b"aoeiuv;/bpmfdtnlgkhjqxzcsryw,.";
const 无根: usize = u32::MAX as usize;
const 编码空间: usize = 1 << 20;
fn 报错(说明: impl Into<String>) -> 错误 {
    std::io::Error::new(std::io::ErrorKind::InvalidData, 说明.into()).into()
}
fn 检查(条件: bool, 说明: &str) -> 结果<()> {
    if 条件 { Ok(()) } else { Err(报错(说明)) }
}
fn 键号(键: &str) -> 结果<u8> {
    检查(键.len() == 1, "键必须是一个 ASCII 字符")?;
    键表
        .iter()
        .position(|&字节| 字节 == 键.as_bytes()[0])
        .map(|位| 位 as u8 + 1)
        .ok_or_else(|| 报错(format!("未知键：{键}")))
}
fn 是小集合(键: u8) -> bool {
    (1..=8).contains(&键)
}

#[derive(Deserialize)]
struct 输入 {
    同笔主根: Option<[String; 3]>,
    版本: u32,
    初始拆分: Vec<usize>,
    局部约束: Option<Vec<Vec<Vec<usize>>>>,
    配置: Value,
    字根: Vec<输入根>,
    部件: Vec<输入部件>,
    复合体: Vec<输入复合体>,
    字: Vec<输入字>,
    字根字: Vec<输入字根字>,
}
#[derive(Deserialize)]
struct 输入根 {
    名称: String,
    落键: 键域,
    #[serde(default)]
    核心: bool,
    初始: Option<配置安排>,
    空间: Vec<配置安排>,
}
#[derive(Deserialize)]
struct 输入候选 {
    笔画: Vec<u8>,
    字根: Vec<String>,
}
#[derive(Deserialize)]
struct 输入部件 {
    id: u32,
    候选: Vec<输入候选>,
    拆分: Vec<Vec<usize>>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum 输入正根 {
    完整 { 字根: String },
    首根 { 部件: u32 },
}
#[derive(Deserialize)]
struct 输入余根 {
    部件: u32,
    数量: usize,
}
#[derive(Deserialize)]
struct 输入复合体 {
    id: u32,
    正根: 输入正根,
    余根: Vec<输入余根>,
}
#[derive(Deserialize)]
struct 输入字 {
    字: String,
    字形: u32,
    声母: Vec<String>,
}
#[derive(Deserialize)]
struct 输入字根字 {
    字形: u32,
    重复: u8,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
enum 键域 {
    小集合,
    补集合,
    全集合,
}
impl 键域 {
    fn 包含(self, 键: u8) -> bool {
        键 != 0
            && match self {
                Self::小集合 => 是小集合(键),
                Self::补集合 => !是小集合(键),
                Self::全集合 => true,
            }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum 布局 {
    关闭,
    键(u8),
    声(u8, Option<usize>),
    归并(usize),
}
struct 根 {
    名称: String,
    落键: 键域,
    核心: bool,
    空间: Vec<布局>,
}
impl 根 {
    fn 是备选(&self) -> bool {
        self.空间.first() == Some(&布局::关闭)
    }
}

struct 归并索引 {
    下游: Vec<Vec<usize>>,
    联动: Vec<Vec<usize>>,
}
impl 归并索引 {
    fn 新建(根列表: &[根]) -> Self {
        let mut 下游 = vec![Vec::new(); 根列表.len()];
        for (r, 根) in 根列表.iter().enumerate() {
            for &安排 in &根.空间 {
                if let 布局::归并(目标) = 安排 {
                    下游[目标].push(r);
                }
            }
        }
        let mut 已访问 = vec![false; 根列表.len()];
        let mut 联动 = Vec::new();
        let mut 分量 = Vec::new();
        for 起点 in 0..根列表.len() {
            if 已访问[起点] {
                continue;
            }
            已访问[起点] = true;
            分量.clear();
            分量.push(起点);
            let mut i = 0;
            while i < 分量.len() {
                let r = 分量[i];
                let 目标 = 根列表[r].空间.iter().filter_map(|安排| match 安排 {
                    布局::归并(目标) => Some(*目标),
                    _ => None,
                });
                // 固定终点不传递约束，可以让引用同一笔画的根独立采样。
                for 邻 in 下游[r].iter().copied().chain(目标) {
                    let 固定终点 = |r: usize| {
                        matches!(根列表[r].空间.as_slice(), [布局::键(_) | 布局::声(..)])
                    };
                    if !已访问[邻] && !固定终点(r) && !固定终点(邻) {
                        已访问[邻] = true;
                        分量.push(邻);
                    }
                }
                i += 1;
            }
            let 可变: Vec<_> = 分量
                .iter()
                .copied()
                .filter(|&r| 根列表[r].空间.len() > 1)
                .collect();
            if 可变.len() > 1 {
                联动.push(可变);
            }
        }
        Self { 下游, 联动 }
    }
}
#[derive(Clone, Copy)]
enum 解析标记 {
    已知,
    待解析,
    解析中,
}
struct 键位解析器 {
    标记: Vec<解析标记>,
    路径: Vec<usize>,
}
impl 键位解析器 {
    fn 新建(根数: usize) -> Self {
        Self {
            标记: vec![解析标记::已知; 根数],
            路径: Vec::new(),
        }
    }

    fn 解析(
        &mut self, 根列表: &[根], 安排: &[usize], 待根: &[usize], 键位: &mut [u8],
        变化: &mut Vec<(usize, u8)>,
    ) -> Result<(), usize> {
        for &r in 待根 {
            self.标记[r] = 解析标记::待解析;
        }
        let 结果 = 待根.iter().try_for_each(|&起点| {
            self.路径.clear();
            let mut r = 起点;
            let 键 = loop {
                match self.标记[r] {
                    解析标记::已知 => break 键位[r],
                    解析标记::解析中 => return Err(r),
                    解析标记::待解析 => {}
                }
                self.标记[r] = 解析标记::解析中;
                self.路径.push(r);
                match 根列表[r].空间[安排[r]] {
                    布局::关闭 => break 0,
                    布局::键(键) | 布局::声(键, _) => break 键,
                    布局::归并(目标) => r = 目标,
                }
            };
            for &r in self.路径.iter().rev() {
                let 根 = &根列表[r];
                if 根.空间[安排[r]] != 布局::关闭 && !根.落键.包含(键) {
                    return Err(r);
                }
                if 键位[r] != 键 {
                    变化.push((r, 键位[r]));
                    键位[r] = 键;
                }
                self.标记[r] = 解析标记::已知;
            }
            Ok(())
        });
        for &r in 待根 {
            self.标记[r] = 解析标记::已知;
        }
        结果
    }
}

struct 候选 {
    根: Vec<usize>,
}
struct 部件数据 {
    id: u32,
    候选: std::ops::Range<usize>,
    拆分: Vec<Vec<usize>>,
}
#[derive(Clone, Copy)]
enum 来源 {
    空,
    根(usize),
    部件(usize, usize),
}
struct 字数据 {
    字: char,
    声: u8,
    同字声键: u32, // 零声母落到同字已有声键时，只保留固定声母的码。
    来源: [来源; 3],
    整字: Option<usize>,
}

/// 快照仅保存决策；缓存由输入重建。无根哨兵在 JSON 中也能精确表示。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct 决策 {
    安排: Vec<usize>,
    拆分: Vec<usize>,
    选根: Vec<usize>,
    重复: Vec<u8>,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct 指标 {
    pub 重码: usize,
    pub 小集合组数: usize,
    pub 补集合例外组数: usize,
    pub 分数: f64,
}
#[derive(Clone, Copy, Debug)]
pub struct 预算 {
    pub 小集合: usize,
    pub 补集合: usize,
    pub 惩罚: f64,
}
impl Default for 预算 {
    fn default() -> Self {
        Self {
            小集合: 50,
            补集合: 30,
            惩罚: 1.0,
        }
    }
}
impl 预算 {
    fn 评分(self, 重码: usize, 小: usize, 补: usize) -> 指标 {
        let 超小 = 小.saturating_sub(self.小集合) as f64;
        let 超补 = 补.saturating_sub(self.补集合) as f64;
        指标 {
            重码,
            小集合组数: 小,
            补集合例外组数: 补,
            分数: 重码 as f64 + self.惩罚 * (超小 * 超小 + 超补 * 超补),
        }
    }
}
#[derive(Clone, Debug)]
pub struct 退火参数 {
    pub 线程: usize,
    pub 步数: usize,
    pub 最高温: f64,
    pub 最低温: f64,
    pub 种子: u64,
    pub 报告间隔: usize,
    pub 校验间隔: usize,
    pub 预算: 预算,
}
#[derive(Serialize)]
pub struct 优化结果 {
    pub 线程: usize,
    pub 步数: usize,
    pub 有效步数: usize,
    pub 接受步数: usize,
    pub 添加字根: usize,
    pub 耗时秒: f64,
    pub 每步微秒: f64,
    pub 每有效步微秒: f64,
    pub 最优指标: 指标,
    pub 结束指标: 指标,
    #[serde(skip)]
    最优决策: 决策,
}
#[derive(Serialize, Deserialize)]
struct 快照 {
    版本: u32,
    输入校验: u64,
    步数: usize,
    决策: 决策,
}

pub struct 上下文 {
    零声母: Option<usize>,
    同笔主根: Option<[usize; 3]>,
    原配置: Value,
    pub 配置: 配置,
    输入校验: u64,
    冲突: 冲突索引,
    根: Vec<根>,
    声母锚点: Vec<String>,
    归并: 归并索引,
    候选: Vec<候选>,
    部件: Vec<部件数据>,
    字: Vec<字数据>,
    根部件: Vec<Vec<usize>>,
    根字: Vec<Vec<usize>>,
    部件字: Vec<Vec<usize>>,
    部件整字: Vec<Vec<usize>>,
    可换拆分: Vec<usize>,
    可重复: Vec<usize>,
    备选: Vec<usize>,
    初始: 决策,
}

impl 上下文 {
    pub fn 读取(路径: &Path) -> 结果<Self> {
        let 字节 = fs::read(路径)?;
        let 输入校验 = 字节.iter().fold(0xcbf29ce484222325u64, |哈希, &字节| {
            (哈希 ^ 字节 as u64).wrapping_mul(0x100000001b3)
        });
        let 输入: 输入 = serde_json::from_slice(&字节)?;
        检查(
            输入.版本 == 3,
            "局部一致性需要版本 3 输入，请重新运行 prepare.ts",
        )?;
        let 冲突 = 冲突索引::新建(
            输入.部件.iter().map(|件| 件.拆分.len()),
            输入
                .局部约束
                .as_deref()
                .ok_or_else(|| 报错("缺少局部约束"))?,
        )?;
        // 使用 libchai 配置结构读取；另保留原始值，防止旧结构丢弃 fusion/glyphs 等字段。
        let 配置: 配置 = serde_json::from_value(输入.配置.clone())?;
        let mut 字母表 = 配置.form.alphabet.as_bytes().to_vec();
        字母表.sort_unstable();
        let mut 预期字母表 = 键表.to_vec();
        预期字母表.sort_unstable();
        检查(字母表 == 预期字母表, "alphabet 必须恰好包含方案的 30 个键")?;
        let 根索引: FxHashMap<_, _> = 输入
            .字根
            .iter()
            .enumerate()
            .map(|(i, 根)| (根.名称.clone(), i))
            .collect();
        检查(
            根索引.len() == 输入.字根.len() && !根索引.is_empty(),
            "字根为空或重名",
        )?;
        let 取根 = |名称: &str| {
            根索引
                .get(名称)
                .copied()
                .ok_or_else(|| 报错(format!("未知字根：{名称}")))
        };
        let 同笔主根 = match 输入.同笔主根 {
            Some([二, 三, 四]) => Some([取根(&二)?, 取根(&三)?, 取根(&四)?]),
            None => None,
        };
        let mut 声母锚点: Vec<_> = 配置
            .form
            .mapping
            .keys()
            .filter(|名称| 名称.starts_with("声-"))
            .cloned()
            .collect();
        声母锚点.sort_unstable();
        // 旧输入已把零声母映射到固定键，沿用原决策长度；新输入追加独立的声母决策。
        let 零声空间 = 配置.form.mapping_space.as_ref().and_then(|表| 表.get("声-0"));
        let 零声母 = 零声空间.filter(|列| !列.is_empty()).map(|_| 输入.字根.len());
        let 展开 = |安排: &配置安排| -> 结果<Vec<布局>> {
            match 安排 {
                配置安排::Basic(键) => Ok(vec![布局::键(键号(键)?)]),
                配置安排::Grouped { element } if element.starts_with("声-") => {
                    if element == "声-0" && let Some(r) = 零声母 {
                        return Ok(vec![布局::归并(r)]);
                    }
                    检查(
                        配置
                            .form
                            .mapping_space
                            .as_ref()
                            .and_then(|表| 表.get(element))
                            .is_none_or(Vec::is_empty),
                        "声母锚点不能移动",
                    )?;
                    let Some(配置安排::Basic(键)) = 配置.form.mapping.get(element) else {
                        return Err(报错("声母锚点必须固定在单键"));
                    };
                    let 键 = 键号(键)?;
                    检查(!是小集合(键), "声母锚点不能位于 B")?;
                    let 锚点 = Some(声母锚点.binary_search(element).expect("已声明的声母锚点"));
                    Ok(vec![布局::声(键, 锚点)])
                }
                配置安排::Grouped { element } => Ok(vec![布局::归并(取根(element)?)]),
                配置安排::Unused(_) => Ok(vec![布局::关闭]),
                配置安排::Advanced(位) => {
                    检查(位.len() == 1, "只支持一码字根")?;
                    match &位[0] {
                        广义码位::Ascii(键) => Ok(vec![布局::键(键号(&键.to_string())?)]),
                        广义码位::Variable { variable } => {
                            let 键集 = 配置
                                .form
                                .mapping_variables
                                .as_ref()
                                .and_then(|表| 表.get(variable))
                                .ok_or_else(|| 报错("未知键位变量"))?;
                            键集
                                .keys
                                .iter()
                                .map(|键| Ok(布局::键(键号(&键.to_string())?)))
                                .collect()
                        }
                        _ => Err(报错("不支持的字根安排")),
                    }
                }
            }
        };
        let mut 根列表 = Vec::new();
        let mut 初始安排 = Vec::new();
        for 根 in &输入.字根 {
            let (落键, 核心, 备选) = (根.落键, 根.核心, 根.初始.is_none());
            检查(
                !(核心 || 备选) || 落键 == 键域::小集合,
                "核心根和动态候选必须位于小集合",
            )?;
            检查(!(核心 && 备选), "核心根必须始终启用")?;
            let mut 空间 = Vec::new();
            if 备选 {
                空间.push(布局::关闭);
            }
            for 安排 in &根.空间 {
                for 值 in 展开(安排)? {
                    if !空间.contains(&值) {
                        空间.push(值);
                    }
                }
            }
            let 初始 = 根
                .初始
                .as_ref()
                .map(&展开)
                .transpose()?
                .unwrap_or_else(|| vec![布局::关闭]);
            检查(!备选 || 空间.len() > 1, "动态候选必须有可启用安排")?;
            检查(
                初始.len() == 1 && (初始[0] == 布局::关闭) == 备选,
                "字根初始启用状态与键域声明不一致",
            )?;
            let 配置初始 = 配置
                .form
                .mapping
                .get(&根.名称)
                .map(&展开)
                .transpose()?
                .unwrap_or_else(|| vec![布局::关闭]);
            检查(初始 == 配置初始, "展开输入与配置初始安排不一致")?;
            let mut 配置空间 = 配置初始;
            if let Some(安排表) = 配置
                .form
                .mapping_space
                .as_ref()
                .and_then(|表| 表.get(&根.名称))
            {
                for 描述 in 安排表 {
                    检查(
                        描述.condition.as_ref().is_none_or(Vec::is_empty),
                        "不支持条件字根空间",
                    )?;
                    for 值 in 展开(&描述.value)? {
                        if !配置空间.contains(&值) {
                            配置空间.push(值);
                        }
                    }
                }
            }
            检查(
                空间.iter().all(|值| 配置空间.contains(值))
                    && 配置空间.iter().all(|值| 空间.contains(值)),
                "输入与配置决策空间不一致",
            )?;
            for &值 in &空间 {
                let 合法 = match 值 {
                    布局::关闭 => 备选,
                    布局::键(键) => 落键.包含(键),
                    布局::声(..) | 布局::归并(_) => true,
                };
                if !合法 {
                    return Err(报错(format!("字根安排超出允许键域：{}", 根.名称)));
                }
            }
            if matches!(根.名称.as_str(), "1" | "2" | "3" | "4" | "5") {
                检查(
                    核心 && matches!(空间.as_slice(), [布局::键(_)]),
                    "笔画必须固定在小集合并属于核心",
                )?;
            }
            初始安排.push(
                空间
                    .iter()
                    .position(|值| *值 == 初始[0])
                    .ok_or_else(|| 报错("初始安排不在空间中"))?,
            );
            根列表.push(根 {
                名称: 根.名称.clone(),
                落键,
                核心,
                空间,
            });
        }
        if let Some(列) = 零声空间.filter(|列| !列.is_empty()) {
            let mut 空间 = Vec::new();
            for 项 in 列 {
                let 配置安排::Basic(键) = &项.value else { return Err(报错("零声母需要单键候选")); };
                检查("bpmfdtnlgkhwjqxyzcsr".contains(键.as_str()) && 项.condition.as_ref().is_none_or(Vec::is_empty), "零声母键域无效")?;
                let 键 = 布局::键(键号(键)?);
                if !空间.contains(&键) { 空间.push(键); }
            }
            let 初始 = 配置.form.mapping.get("声-0").ok_or_else(|| 报错("缺少零声母键"))?;
            let 初始 = 展开(初始)?;
            初始安排.push(空间.iter().position(|键| 初始.as_slice() == [*键]).ok_or_else(|| 报错("零声母初始键不在空间内"))?);
            根列表.push(根 { 名称: "声-0".into(), 落键: 键域::补集合, 核心: false, 空间 });
        }
        let 归并 = 归并索引::新建(&根列表);
        let 部件索引: FxHashMap<_, _> = 输入
            .部件
            .iter()
            .enumerate()
            .map(|(i, 件)| (件.id, i))
            .collect();
        检查(部件索引.len() == 输入.部件.len(), "部件 ID 重复")?;
        let 取部件 = |id: u32| {
            部件索引
                .get(&id)
                .copied()
                .ok_or_else(|| 报错(format!("未知部件：{id:05x}")))
        };
        let mut 候选列表 = Vec::new();
        let mut 部件列表 = Vec::new();
        let mut 根部件 = vec![Vec::new(); 根列表.len()];
        for (i, 件) in 输入.部件.iter().enumerate() {
            let 起点 = 候选列表.len();
            let mut 掩码列表 = Vec::new();
            let mut 全掩码 = 0u32;
            for 项 in &件.候选 {
                检查(
                    !项.笔画.is_empty() && 项.字根.len() == 1,
                    "版本 3 候选必须有笔画且恰有一个实际字根",
                )?;
                检查(
                    项.笔画.windows(2).all(|对| 对[0] < 对[1]),
                    "候选笔序未排序或重复",
                )?;
                let mut 掩码 = 0;
                for &笔 in &项.笔画 {
                    检查((1..=30).contains(&笔), "笔序超出 1–30")?;
                    掩码 |= 1 << (笔 - 1);
                }
                全掩码 |= 掩码;
                掩码列表.push(掩码);
                let 根: Vec<_> = 项.字根.iter().map(|名| 取根(名)).collect::<结果<_>>()?;
                for &r in &根 {
                    根部件[r].push(i);
                }
                候选列表.push(候选 { 根 });
            }
            检查(
                全掩码 > 0 && 全掩码 & (全掩码 + 1) == 0,
                "部件笔画覆盖不连续",
            )?;
            检查(
                !件.拆分.is_empty() && 件.拆分.windows(2).all(|对| 对[0].len() <= 对[1].len()),
                "拆分为空或未按根数排序",
            )?;
            let mut 永久覆盖 = false;
            for 分法 in &件.拆分 {
                let mut 已用 = 0u32;
                let mut 首笔 = None;
                for &j in 分法 {
                    检查(j < 掩码列表.len(), "拆分候选索引越界")?;
                    let 码 = 掩码列表[j];
                    检查(
                        已用 & 码 == 0 && 首笔.is_none_or(|笔| 笔 < 码.trailing_zeros()),
                        "拆分重叠或首笔顺序错误",
                    )?;
                    已用 |= 码;
                    首笔 = Some(码.trailing_zeros());
                }
                检查(已用 == 全掩码, "拆分未完整覆盖部件")?;
                永久覆盖 |= 分法
                    .iter()
                    .all(|&j| 候选列表[起点 + j].根.iter().any(|&r| 根列表[r].核心));
            }
            检查(永久覆盖, "缺少核心字根组成的完整拆分")?;
            部件列表.push(部件数据 {
                id: 件.id,
                候选: 起点..候选列表.len(),
                拆分: 件
                    .拆分
                    .iter()
                    .map(|分法| 分法.iter().map(|j| 起点 + j).collect())
                    .collect(),
            });
        }
        for 列 in &mut 根部件 {
            列.sort_unstable();
            列.dedup();
        }
        let 复合表: FxHashMap<_, _> = 输入.复合体.iter().map(|形| (形.id, 形)).collect();
        检查(复合表.len() == 输入.复合体.len(), "复合体 ID 重复")?;
        let mut 字列表 = Vec::new();
        let mut 唯一字音 = FxHashSet::default();
        let mut 根字 = vec![Vec::new(); 根列表.len()];
        let mut 部件字 = vec![Vec::new(); 部件列表.len()];
        let mut 部件整字 = vec![Vec::new(); 部件列表.len()];
        for 字 in &输入.字 {
            let 字符: Vec<_> = 字.字.chars().collect();
            检查(
                字符.len() == 1 && !字.声母.is_empty(),
                "必须是单字且具有声母",
            )?;
            let mut 来源表 = [来源::空; 3];
            let 整字;
            if let Some(&件) = 部件索引.get(&字.字形) {
                整字 = Some(件);
            } else {
                整字 = None;
                let 形 = 复合表
                    .get(&字.字形)
                    .ok_or_else(|| 报错("字符未找到冻结分部"))?;
                来源表[0] = match &形.正根 {
                    输入正根::完整 { 字根 } => {
                        let r = 取根(字根)?;
                        检查(!根列表[r].是备选(), "完整正根必须初始启用")?;
                        来源::根(r)
                    }
                    输入正根::首根 { 部件 } => 来源::部件(取部件(*部件)?, 0),
                };
                let mut 位 = 1;
                for 请求 in &形.余根 {
                    检查(
                        (1..=2).contains(&请求.数量) && 位 + 请求.数量 <= 3,
                        "余根请求超出两码",
                    )?;
                    for 偏移 in 0..请求.数量 {
                        来源表[位] = 来源::部件(取部件(请求.部件)?, 偏移);
                        位 += 1;
                    }
                }
                检查(位 >= 2, "复合体缺少余根请求")?;
            }
            let mut 固定声键 = 0u32;
            for 声 in 字.声母.iter().filter(|声| 声.as_str() != "0") {
                固定声键 |= 1 << 键号(声)?;
            }
            for 声 in &字.声母 {
                let 声 = if 声 == "0" {
                    let r = 零声母.ok_or_else(|| 报错("零声母缺少决策空间"))?;
                    根字[r].push(字列表.len());
                    0
                } else { 键号(声)? };
                检查(
                    !是小集合(声) && 唯一字音.insert((字符[0], 声)),
                    "声母位于 B 或字声母组合重复",
                )?;
                let i = 字列表.len();
                if let Some(件) = 整字 {
                    部件字[件].push(i);
                    部件整字[件].push(i);
                }
                for &来源 in &来源表 {
                    match 来源 {
                        来源::根(r) => 根字[r].push(i),
                        来源::部件(件, _) => 部件字[件].push(i),
                        来源::空 => {}
                    }
                }
                字列表.push(字数据 {
                    字: 字符[0],
                    声,
                    同字声键: 固定声键,
                    来源: 来源表,
                    整字,
                });
            }
        }
        检查(!字列表.is_empty(), "输入没有可编码字")?;
        for 列 in &mut 部件字 {
            列.sort_unstable();
            列.dedup();
        }
        let mut 重复 = vec![1; 部件列表.len()];
        for 条 in &输入.字根字 {
            检查((1..=3).contains(&条.重复), "重复次数必须为 1–3")?;
            重复[取部件(条.字形)?] = 条.重复;
        }
        let 可重复 = 部件整字
            .iter()
            .enumerate()
            .filter_map(|(i, 字)| (!字.is_empty()).then_some(i))
            .collect();
        let 可换拆分 = 部件列表
            .iter()
            .enumerate()
            .filter_map(|(i, 件)| (件.拆分.len() > 1).then_some(i))
            .collect();
        let 备选 = 根列表
            .iter()
            .enumerate()
            .filter_map(|(i, 根)| (根.是备选()).then_some(i))
            .collect();
        let 初始 = 决策 {
            安排: 初始安排,
            拆分: 输入.初始拆分,
            选根: vec![无根; 候选列表.len()],
            重复,
        };
        let mut 上下文 = Self {
            零声母,
            同笔主根,
            原配置: 输入.配置,
            配置,
            输入校验,
            冲突,
            根: 根列表,
            声母锚点,
            归并,
            候选: 候选列表,
            部件: 部件列表,
            字: 字列表,
            根部件,
            根字,
            部件字,
            部件整字,
            可换拆分,
            可重复,
            备选,
            初始,
        };
        上下文.初始 = 状态::新建(&上下文, 上下文.初始.clone(), false)?.决策;
        Ok(上下文)
    }
    pub fn 默认参数(&self) -> (usize, f64, f64) {
        if let Some(求解器配置::SimulatedAnnealing(方法)) = self
            .配置
            .optimization
            .as_ref()
            .and_then(|项| 项.metaheuristic.as_ref())
            && let Some(参数) = 方法.parameters
        {
            return (参数.steps, 参数.t_max, 参数.t_min);
        }
        (1_000_000, 2.5, 0.015)
    }
    pub fn 初始决策(&self) -> 决策 {
        self.初始.clone()
    }
    pub fn 读取决策(&self, 路径: &Path) -> 结果<决策> {
        let 快照: 快照 = serde_json::from_slice(&fs::read(路径)?)?;
        检查(
            快照.版本 == 1 && 快照.输入校验 == self.输入校验,
            "快照与预处理输入不匹配，请使用原 input.json",
        )?;
        Ok(状态::新建(self, 快照.决策, true)?.决策)
    }
    pub fn 编码(&self, 决策: 决策, 目录: &Path, 预算: 预算) -> 结果<指标> {
        检查(
            预算.惩罚.is_finite() && 预算.惩罚 >= 0.0,
            "预算惩罚必须为有限非负数",
        )?;
        let 状态 = 状态::新建(self, 决策, true)?;
        self.导出(&状态, 目录, 预算, 0)?;
        Ok(状态.指标(预算))
    }
    fn 保存快照(
        &self, 决策: &决策, 目录: &Path, 文件: &str, 步数: usize
    ) -> 结果<()> {
        let 临时 = 目录.join(format!("{文件}.tmp"));
        let mut 写入 = BufWriter::new(File::create(&临时)?);
        serde_json::to_writer(
            &mut 写入,
            &快照 {
                版本: 1,
                输入校验: self.输入校验,
                步数,
                决策: 决策.clone(),
            },
        )?;
        写入.flush()?;
        fs::rename(临时, 目录.join(文件))?;
        Ok(())
    }
    fn 导出(&self, 状态: &状态, 目录: &Path, 预算: 预算, 步数: usize) -> 结果<()> {
        fs::create_dir_all(目录)?;
        self.保存快照(&状态.决策, 目录, "solution.json", 步数)?;
        let mut 配置 = self.原配置.clone();
        let 映射 = 配置["form"]["mapping"]
            .as_object_mut()
            .ok_or_else(|| 报错("配置缺少 mapping"))?;
        for (i, 根) in self.根.iter().enumerate() {
            let 值 = match 根.空间[状态.决策.安排[i]] {
                布局::关闭 => {
                    映射.remove(&根.名称);
                    continue;
                }
                布局::键(键) => Value::String((键表[键 as usize - 1] as char).to_string()),
                布局::归并(j) => serde_json::json!({ "element": self.根[j].名称 }),
                布局::声(_, Some(锚点)) => {
                    serde_json::json!({ "element": self.声母锚点[锚点] })
                }
                布局::声(键, None) => {
                    // 旧输入按键合并同键声母，沿用旧快照的锚点选择。
                    let 原 = self.配置.form.mapping.get(&根.名称).into_iter().chain(
                        self.配置
                            .form
                            .mapping_space
                            .as_ref()
                            .and_then(|表| 表.get(&根.名称))
                            .into_iter()
                            .flatten()
                            .map(|项| &项.value),
                    );
                    let 名 = 原.filter_map(|项| if let 配置安排::Grouped { element } = 项 { Some(element) } else { None }).find(|名| {
                        名.starts_with("声-") && matches!(self.配置.form.mapping.get(*名), Some(配置安排::Basic(字母)) if 键号(字母).ok() == Some(键))
                    }).ok_or_else(|| 报错("未找到声母锚点"))?;
                    serde_json::json!({ "element": 名 })
                }
            };
            映射.insert(根.名称.clone(), 值);
        }
        fs::write(目录.join("config.yaml"), serde_yaml::to_string(&配置)?)?;
        let 拆分: Vec<_> = self.部件.iter().enumerate().map(|(i, 件)| {
            let 根: Vec<_> = 件.拆分[状态.决策.拆分[i]].iter().map(|&候选| &self.根[状态.决策.选根[候选]].名称).collect();
            serde_json::json!({ "字形": format!("字形-{:05x}", 件.id), "字根": 根, "重复": 状态.决策.重复[i] })
        }).collect();
        fs::write(目录.join("splits.json"), serde_json::to_vec_pretty(&拆分)?)?;
        let mut 码表 = BufWriter::new(File::create(目录.join("code.txt"))?);
        for (i, 字) in self.字.iter().enumerate() {
            let mut 码 = 状态.码[i];
            if 码 == 0 { continue; }
            let mut 文本 = String::with_capacity(4);
            for _ in 0..4 {
                let 键 = (码 & 31) as usize;
                if 键 != 0 {
                    文本.push(键表[键 - 1] as char);
                }
                码 >>= 5;
            }
            writeln!(码表, "{}\t{}", 字.字, 文本)?;
        }
        码表.flush()?;
        fs::write(
            目录.join("分析.json"),
            serde_json::to_vec_pretty(&状态.指标(预算))?,
        )?;
        Ok(())
    }
    pub fn 并行退火(
        &self, 参数: &退火参数, 初始: &决策, 输出: Option<&Path>,
    ) -> 结果<Vec<优化结果>> {
        检查(
            参数.线程 > 0 && 参数.步数 > 0 && 参数.报告间隔 > 0,
            "线程数、步数和报告间隔必须大于零",
        )?;
        检查(
            参数.最高温.is_finite()
                && 参数.最低温.is_finite()
                && 参数.最高温 >= 参数.最低温
                && 参数.最低温 > 0.0,
            "温度必须有限且最高温 ≥ 最低温 > 0",
        )?;
        检查(
            参数.预算.惩罚.is_finite() && 参数.预算.惩罚 >= 0.0,
            "预算惩罚必须为有限非负数",
        )?;
        if let Some(目录) = 输出 {
            fs::create_dir_all(目录)?;
        }
        let mut 结果 = std::thread::scope(|范围| -> 结果<Vec<优化结果>> {
            let 线程: Vec<_> = (0..参数.线程)
                .map(|i| 范围.spawn(move || self.退火(参数, 初始.clone(), i, 输出)))
                .collect();
            线程
                .into_iter()
                .map(|线程| 线程.join().map_err(|_| 报错("退火线程异常"))?)
                .collect()
        })?;
        结果.sort_by(|甲, 乙| 甲.最优指标.分数.total_cmp(&乙.最优指标.分数));
        if let Some(目录) = 输出 {
            fs::write(目录.join("总结.json"), serde_json::to_vec_pretty(&结果)?)?;
            let 最优 = 状态::新建(self, 结果[0].最优决策.clone(), true)?;
            self.导出(&最优, &目录.join("best"), 参数.预算, 参数.步数)?;
        }
        Ok(结果)
    }
    fn 退火(
        &self, 参数: &退火参数, 决策: 决策, 线程序号: usize, 输出: Option<&Path>,
    ) -> 结果<优化结果> {
        let mut 状态 = 状态::新建(self, 决策, true)?;
        let mut 最优指标 = 状态.指标(参数.预算);
        let mut 最优决策 = 状态.决策.clone();
        let mut 当前分数 = 最优指标.分数;
        let mut 随机 = 随机::新建(
            参数
                .种子
                .wrapping_add((线程序号 as u64).wrapping_mul(0x9e3779b97f4a7c15)),
        );
        let 可移动: Vec<_> = self
            .根
            .iter()
            .enumerate()
            .filter_map(|(i, 根)| (根.空间.len() > 1 && !根.是备选()).then_some(i))
            .collect();
        let mut 工作 = 工作区::新建(self);
        let 目录 = 输出.map(|目录| 目录.join(线程序号.to_string()));
        let mut 日志 = if let Some(目录) = &目录 {
            fs::create_dir_all(目录)?;
            Some(BufWriter::new(File::create(目录.join("log.txt"))?))
        } else {
            None
        };
        let mut 有效步数 = 0;
        let mut 接受步数 = 0;
        let mut 添加字根 = 0;
        let 系数 = (参数.最低温 / 参数.最高温).powf(1.0 / 参数.步数 as f64);
        let mut 温度 = 参数.最高温;
        let 开始 = Instant::now();
        for 步 in 1..=参数.步数 {
            工作.清空();
            self.提议(&mut 状态, &mut 工作, &mut 随机, &可移动);
            if !工作.安排.is_empty()
                || !工作.拆分.is_empty()
                || !工作.选根.is_empty()
                || !工作.重复.is_empty()
            {
                有效步数 += 1;
                let 原小 = 状态.小组;
                let 原补 = 状态.补组;
                let 启用数 = 工作
                    .安排
                    .iter()
                    .filter(|&&(r, 旧)| self.根[r].空间[旧] == 布局::关闭)
                    .count();
                let 合法 = self.更新(&mut 状态, &mut 工作, &mut 随机, 启用数 > 0);
                let 新指标 = 状态.指标(参数.预算);
                let 差 = 新指标.分数 - 当前分数;
                let 接受 = 合法 && (差 <= 0.0 || 随机.小数().ln() < -差 / 温度);
                if 接受 {
                    接受步数 += 1;
                    当前分数 = 新指标.分数;
                    添加字根 += 启用数;
                    if 当前分数 < 最优指标.分数 {
                        最优指标 = 新指标;
                        最优决策.clone_from(&状态.决策);
                    }
                } else {
                    工作.回退(&mut 状态);
                    状态.小组 = 原小;
                    状态.补组 = 原补;
                }
            }
            温度 *= 系数;
            if 参数.校验间隔 != 0 && 步 % 参数.校验间隔 == 0 {
                状态.核验(self)?;
            }
            if 步 % 参数.报告间隔 == 0 || 步 == 参数.步数 {
                if let Some(日志) = &mut 日志 {
                    writeln!(
                        日志,
                        "{步}\t{温度:.6}\t当前 {当前分数:.3}\t最优 {:.3}\t重码 {}\tB {}\t补例外 {}\t有效 {有效步数}\t接受 {接受步数}",
                        最优指标.分数, 最优指标.重码, 最优指标.小集合组数, 最优指标.补集合例外组数
                    )?;
                    日志.flush()?;
                }
                if let Some(目录) = &目录 {
                    self.保存快照(&状态.决策, 目录, "checkpoint.json", 步)?;
                    self.保存快照(&最优决策, 目录, "solution.json", 步)?;
                }
            }
        }
        let 耗时秒 = 开始.elapsed().as_secs_f64();
        状态.核验(self)?;
        if let Some(目录) = &目录 {
            self.导出(
                &状态::新建(self, 最优决策.clone(), true)?,
                目录,
                参数.预算,
                参数.步数,
            )?;
        }
        Ok(优化结果 {
            线程: 线程序号,
            步数: 参数.步数,
            有效步数,
            接受步数,
            添加字根,
            耗时秒,
            每步微秒: 耗时秒 * 1e6 / 参数.步数 as f64,
            每有效步微秒: if 有效步数 == 0 {
                0.0
            } else {
                耗时秒 * 1e6 / 有效步数 as f64
            },
            最优指标,
            结束指标: 状态.指标(参数.预算),
            最优决策,
        })
    }

    fn 提议(
        &self, 状态: &mut 状态, 工作: &mut 工作区, 随机: &mut 随机, 可移动: &[usize]
    ) {
        match 随机.范围(100) {
            0..=43 if !可移动.is_empty() => {
                let r = 可移动[随机.范围(可移动.len())];
                工作.改安排(状态, r, 随机.范围(self.根[r].空间.len()));
            }
            操作 @ 44..=53 if 可移动.len() > 1 => {
                let (a, b) = if 操作 == 44 && let Some(根) = self.同笔主根 {
                    let i = 随机.范围(3);
                    (根[i], 根[(i + 1 + 随机.范围(2)) % 3])
                } else {
                    (可移动[随机.范围(可移动.len())], 可移动[随机.范围(可移动.len())])
                };
                if a != b {
                    let 甲 = self.根[a].空间[状态.决策.安排[a]];
                    let 乙 = self.根[b].空间[状态.决策.安排[b]];
                    if matches!(甲, 布局::键(_))
                        && matches!(乙, 布局::键(_))
                        && 甲 != 乙
                        && let (Some(新甲), Some(新乙)) = (
                            self.根[a].空间.iter().position(|值| *值 == 乙),
                            self.根[b].空间.iter().position(|值| *值 == 甲),
                        )
                    {
                        工作
                            .安排
                            .extend([(a, 状态.决策.安排[a]), (b, 状态.决策.安排[b])]);
                        状态.决策.安排[a] = 新甲;
                        状态.决策.安排[b] = 新乙;
                    }
                }
            }
            操作 @ 54..=75 if !self.可换拆分.is_empty() => {
                let 件 = self.可换拆分[随机.范围(self.可换拆分.len())];
                let 新 = self.最短拆分(状态, 件, 随机, false).expect("永久 B 覆盖");
                if 新 != 状态.决策.拆分[件] {
                    工作.拆分.push((件, 状态.决策.拆分[件]));
                    状态.决策.拆分[件] = 新;
                    工作.标部件(件, false);
                    if 操作 >= 70 {
                        let 号 = self.冲突.起点[件] + 新;
                        for &(块, 掩码) in &self.冲突.掩码[号] {
                            let mut 冲突位 = 状态.已选[块] & 掩码;
                            while 冲突位 != 0 {
                                let 邻件 =
                                    self.冲突.所属[块 * 64 + 冲突位.trailing_zeros() as usize];
                                冲突位 &= 冲突位 - 1;
                                let 旧 = 状态.决策.拆分[邻件];
                                let 长 = self.部件[邻件].拆分[旧].len();
                                let mut 数 = 0;
                                let mut 选择 = 旧;
                                for (i, 分) in self.部件[邻件]
                                    .拆分
                                    .iter()
                                    .enumerate()
                                    .take_while(|(_, 分)| 分.len() <= 长)
                                {
                                    if 分.len() == 长
                                        && 分.iter().all(|&候选| 状态.决策.选根[候选] != 无根)
                                        && !self.冲突.互斥(号, self.冲突.起点[邻件] + i)
                                    {
                                        数 += 1;
                                        if 随机.范围(数) == 0 {
                                            选择 = i
                                        }
                                    }
                                }
                                if 选择 != 旧 {
                                    工作.拆分.push((邻件, 旧));
                                    状态.决策.拆分[邻件] = 选择;
                                    工作.标部件(邻件, false);
                                }
                            }
                        }
                    }
                }
            }
            76..=83 if !self.可重复.is_empty() => {
                let 件 = self.可重复[随机.范围(self.可重复.len())];
                if self.部件[件].拆分[状态.决策.拆分[件]].len() == 1 {
                    let 新 = 随机.范围(3) as u8 + 1;
                    if 新 != 状态.决策.重复[件] {
                        工作.重复.push((件, 状态.决策.重复[件]));
                        状态.决策.重复[件] = 新;
                    }
                }
            }
            92..=97 if !self.备选.is_empty() => {
                let r = self.备选[随机.范围(self.备选.len())];
                let 空间 = self.根[r].空间.len();
                let 新 = if 状态.决策.安排[r] != 0 && 随机.范围(4) == 0 {
                    0
                } else {
                    1 + 随机.范围(空间 - 1)
                };
                工作.改安排(状态, r, 新);
            }
            98..=99 if !self.归并.联动.is_empty() => {
                let 分量 = &self.归并.联动[随机.范围(self.归并.联动.len())];
                // 整体重选保留所有合法组合的可达性，部分重选提高大分量的接受率。
                let 整体 = 随机.范围(2) == 0;
                for &r in 分量 {
                    if 整体 || 随机.范围(2) == 0 {
                        工作.改安排(状态, r, 随机.范围(self.根[r].空间.len()));
                    }
                }
            }
            _ => {}
        }
    }
    fn 最短拆分(
        &self, 状态: &状态, 件: usize, 随机: &mut 随机, 保留: bool
    ) -> Option<usize> {
        let mut 选中 = None;
        let mut 根数 = usize::MAX;
        let mut 数量 = 0;
        for (i, 分法) in self.部件[件].拆分.iter().enumerate() {
            if 分法.len() > 根数 {
                break;
            }
            if !分法.iter().all(|&候选| 状态.决策.选根[候选] != 无根) {
                continue;
            }
            根数 = 分法.len();
            if 保留 && i == 状态.决策.拆分[件] {
                return Some(i);
            }
            数量 += 1;
            if 选中.is_none() || 随机.范围(数量) == 0 {
                选中 = Some(i);
            }
        }
        选中
    }
    fn 同笔键合法(&self, 键: &[u8]) -> bool {
        self.同笔主根.is_none_or(|根| {
            let mut 三键 = 根.map(|r| 键[r]);
            三键.sort_unstable();
            三键 == [6, 7, 8] // 键表中的 v、;、/
        })
    }
    fn 更新键位(&self, 状态: &mut 状态, 工作: &mut 工作区) -> bool {
        for i in 0..工作.安排.len() {
            工作.标根(工作.安排[i].0);
        }
        let mut i = 0;
        while i < 工作.待根.len() {
            let r = 工作.待根[i];
            for &下游 in &self.归并.下游[r] {
                if self.根[下游].空间[状态.决策.安排[下游]] == 布局::归并(r) {
                    工作.标根(下游);
                }
            }
            i += 1;
        }
        if 工作
            .解析
            .解析(
                &self.根,
                &状态.决策.安排,
                &工作.待根,
                &mut 状态.键,
                &mut 工作.键,
            )
            .is_err()
        {
            return false;
        }
        if !self.同笔键合法(&状态.键) {
            return false;
        }
        for &(r, 原位置) in &工作.安排 {
            if Some(r) == self.零声母 { continue; }
            let 空间 = &self.根[r].空间;
            if let 布局::键(键) = 空间[原位置] {
                if 是小集合(键) {
                    状态.小组 -= 1;
                } else {
                    状态.补组 -= 1;
                }
            }
            if let 布局::键(键) = 空间[状态.决策.安排[r]] {
                if 是小集合(键) {
                    状态.小组 += 1;
                } else {
                    状态.补组 += 1;
                }
            }
        }
        for i in 0..工作.键.len() {
            let (r, 旧) = 工作.键[i];
            let 键 = 状态.键[r];
            if 是小集合(旧) || 是小集合(键) {
                for &件 in &self.根部件[r] {
                    工作.标部件(件, 是小集合(旧) != 是小集合(键));
                }
            }
            for &字 in &self.根字[r] {
                工作.标字(字);
            }
        }
        true
    }
    fn 更新(
        &self, 状态: &mut 状态, 工作: &mut 工作区, 随机: &mut 随机, 添加: bool
    ) -> bool {
        if !工作.安排.is_empty() && !self.更新键位(状态, 工作) {
            return false;
        }
        for i in 0..工作.待部件.len() {
            let 件 = 工作.待部件[i];
            if 工作.重选标记[件] == 工作.轮次 {
                for 候选 in self.部件[件].候选.clone() {
                    let 旧 = 状态.决策.选根[候选];
                    if 旧 != 无根 && 是小集合(状态.键[旧]) {
                        continue;
                    }
                    let 新 = self.候选[候选]
                        .根
                        .iter()
                        .copied()
                        .find(|&r| 是小集合(状态.键[r]))
                        .unwrap_or(无根);
                    if 新 != 旧 {
                        工作.选根.push((候选, 旧));
                        状态.决策.选根[候选] = 新;
                    }
                }
                let 新 = self.最短拆分(状态, 件, 随机, !添加).expect("永久 B 覆盖");
                if 新 != 状态.决策.拆分[件] {
                    工作.拆分.push((件, 状态.决策.拆分[件]));
                    状态.决策.拆分[件] = 新;
                }
            }
            let 新码 = 状态.部件编码(self, 件);
            if 新码 != 状态.部件码[件] {
                工作.部件码.push((件, 状态.部件码[件]));
                状态.部件码[件] = 新码;
                for &字 in &self.部件字[件] {
                    工作.标字(字);
                }
            }
        }
        for &(件, 旧) in &工作.拆分 {
            let 起点 = self.冲突.起点[件];
            for 号 in [起点 + 旧, 起点 + 状态.决策.拆分[件]] {
                状态.已选[号 / 64] ^= 1 << (号 % 64);
                工作.翻转.push(号);
            }
        }
        if 工作.拆分.iter().any(|&(件, _)| {
            !self
                .冲突
                .相容(self.冲突.起点[件] + 状态.决策.拆分[件], &状态.已选)
        }) {
            return false;
        }
        for i in 0..工作.重复.len() {
            let (件, _) = 工作.重复[i];
            if 状态.部件码[件][1] == 0 {
                for &字 in &self.部件整字[件] {
                    工作.标字(字);
                }
            }
        }
        for &字 in &工作.待字 {
            let 新码 = 状态.编码(self, 字);
            if 新码 != 状态.码[字] {
                工作.编码.push((字, 状态.码[字]));
                状态.换码(字, 新码);
            }
        }
        true
    }
}

struct 状态 {
    决策: 决策,
    键: Vec<u8>,
    已选: Vec<u64>,
    部件码: Vec<[u8; 3]>,
    码: Vec<u32>,
    计数: Vec<u32>,
    重码: usize,
    小组: usize,
    补组: usize,
}
impl 状态 {
    fn 新建(模型: &上下文, 决策: 决策, 严格: bool) -> 结果<Self> {
        检查(
            决策.安排.len() == 模型.根.len()
                && 决策.拆分.len() == 模型.部件.len()
                && 决策.重复.len() == 模型.部件.len()
                && 决策.选根.len() == 模型.候选.len(),
            "决策长度不匹配",
        )?;
        let mut 状态 = Self {
            决策,
            已选: 模型.冲突.空集(),
            键: vec![0; 模型.根.len()],
            部件码: vec![[0; 3]; 模型.部件.len()],
            码: Vec::new(),
            计数: vec![0; 编码空间],
            重码: 0,
            小组: 0,
            补组: 0,
        };
        for (i, (根, &安排)) in 模型.根.iter().zip(&状态.决策.安排).enumerate() {
            let 安排 = 根.空间.get(安排).ok_or_else(|| 报错("安排索引越界"))?;
            if Some(i) != 模型.零声母 && let 布局::键(键) = *安排 {
                if 是小集合(键) {
                    状态.小组 += 1;
                } else {
                    状态.补组 += 1;
                }
            }
        }
        let 所有根: Vec<_> = (0..模型.根.len()).collect();
        键位解析器::新建(模型.根.len())
            .解析(
                &模型.根,
                &状态.决策.安排,
                &所有根,
                &mut 状态.键,
                &mut Vec::new(),
            )
            .map_err(|r| {
                报错(format!(
                    "字根 {} 的归并成环、目标未启用或超出允许键域",
                    模型.根[r].名称
                ))
            })?;
        检查(模型.同笔键合法(&状态.键), "同笔三组必须一对一占用 v;/")?;
        for (i, 候选) in 模型.候选.iter().enumerate() {
            let 可用 = 候选.根.iter().copied().find(|&r| 是小集合(状态.键[r]));
            if !严格 {
                状态.决策.选根[i] = 可用.unwrap_or(无根);
            }
            let r = 状态.决策.选根[i];
            检查(
                if r == 无根 {
                    可用.is_none()
                } else {
                    候选.根.contains(&r) && 是小集合(状态.键[r])
                },
                "候选选根不是当前有效的 B 字根",
            )?;
        }
        for (件, 数据) in 模型.部件.iter().enumerate() {
            let 最短 = 数据
                .拆分
                .iter()
                .position(|分法| 分法.iter().all(|&i| 状态.决策.选根[i] != 无根))
                .ok_or_else(|| 报错("没有可行拆分"))?;
            let 分法 = 数据
                .拆分
                .get(状态.决策.拆分[件])
                .ok_or_else(|| 报错("拆分索引越界"))?;
            检查(
                分法.len() == 数据.拆分[最短].len()
                    && 分法.iter().all(|&i| 状态.决策.选根[i] != 无根),
                "所选拆分不可行或不是最少根拆分",
            )?;
            检查((1..=3).contains(&状态.决策.重复[件]), "重复次数必须为 1–3")?;
        }
        for (件, &分) in 状态.决策.拆分.iter().enumerate() {
            let 号 = 模型.冲突.起点[件] + 分;
            if !模型.冲突.相容(号, &状态.已选) {
                let 邻 = 模型.冲突.掩码[号]
                    .iter()
                    .find_map(|&(块, 位)| {
                        let 重叠 = 位 & 状态.已选[块];
                        (重叠 != 0)
                            .then(|| 模型.冲突.所属[块 * 64 + 重叠.trailing_zeros() as usize])
                    })
                    .expect("存在冲突");
                return Err(报错(format!(
                    "部件 {:05x} 与 {:05x} 的连续局部拆分冲突",
                    模型.部件[邻].id, 模型.部件[件].id
                )));
            }
            状态.已选[号 / 64] |= 1 << (号 % 64);
            状态.部件码[件] = 状态.部件编码(模型, 件);
        }
        for i in 0..模型.字.len() {
            let 码 = 状态.编码(模型, i);
            状态.码.push(码);
            if 码 == 0 { continue; }
            if 状态.计数[码 as usize] > 0 {
                状态.重码 += 1;
            }
            状态.计数[码 as usize] += 1;
        }
        Ok(状态)
    }
    fn 指标(&self, 预算: 预算) -> 指标 {
        预算.评分(self.重码, self.小组, self.补组)
    }
    fn 部件编码(&self, 模型: &上下文, 件: usize) -> [u8; 3] {
        let mut 码 = [0; 3];
        for (i, &候选) in 模型.部件[件].拆分[self.决策.拆分[件]]
            .iter()
            .take(3)
            .enumerate()
        {
            码[i] = self.键[self.决策.选根[候选]];
        }
        码
    }
    fn 编码(&self, 模型: &上下文, i: usize) -> u32 {
        let 字 = &模型.字[i];
        let 码 = if let Some(件) = 字.整字 {
            let mut 码 = self.部件码[件];
            if 码[1] == 0 {
                for 位 in 1..self.决策.重复[件] as usize {
                    码[位] = 码[0];
                }
            }
            码
        } else {
            字.来源.map(|来源| match 来源 {
                来源::空 => 0,
                来源::根(r) => self.键[r],
                来源::部件(件, 位) => self.部件码[件][位],
            })
        };
        let 声 = if 字.声 == 0 {
            let 键 = self.键[模型.零声母.expect("零声母决策")];
            if 字.同字声键 & (1 << 键) != 0 { return 0; }
            键
        } else { 字.声 };
        声 as u32 | ((码[0] as u32) << 5) | ((码[1] as u32) << 10) | ((码[2] as u32) << 15)
    }
    fn 换码(&mut self, i: usize, 新码: u32) {
        let 旧码 = self.码[i] as usize;
        if 旧码 != 0 {
            if self.计数[旧码] > 1 { self.重码 -= 1; }
            self.计数[旧码] -= 1;
        }
        self.码[i] = 新码;
        if 新码 == 0 { return; }
        if self.计数[新码 as usize] > 0 {
            self.重码 += 1;
        }
        self.计数[新码 as usize] += 1;
    }
    fn 核验(&self, 模型: &上下文) -> 结果<()> {
        let 全量 = Self::新建(模型, self.决策.clone(), true)?;
        检查(
            self.已选 == 全量.已选
                && self.键 == 全量.键
                && self.部件码 == 全量.部件码
                && self.码 == 全量.码
                && self.计数 == 全量.计数
                && self.重码 == 全量.重码
                && self.小组 == 全量.小组
                && self.补组 == 全量.补组,
            "增量状态与全量重算不一致",
        )
    }
}

struct 工作区 {
    翻转: Vec<usize>,
    安排: Vec<(usize, usize)>,
    拆分: Vec<(usize, usize)>,
    选根: Vec<(usize, usize)>,
    重复: Vec<(usize, u8)>,
    键: Vec<(usize, u8)>,
    解析: 键位解析器,
    部件码: Vec<(usize, [u8; 3])>,
    编码: Vec<(usize, u32)>,
    根标记: Vec<u32>,
    部件标记: Vec<u32>,
    重选标记: Vec<u32>,
    字标记: Vec<u32>,
    轮次: u32,
    待根: Vec<usize>,
    待部件: Vec<usize>,
    待字: Vec<usize>,
}
impl 工作区 {
    fn 新建(模型: &上下文) -> Self {
        Self {
            翻转: Vec::new(),
            安排: Vec::new(),
            拆分: Vec::new(),
            选根: Vec::new(),
            重复: Vec::new(),
            键: Vec::new(),
            解析: 键位解析器::新建(模型.根.len()),
            部件码: Vec::new(),
            编码: Vec::new(),
            根标记: vec![0; 模型.根.len()],
            部件标记: vec![0; 模型.部件.len()],
            重选标记: vec![0; 模型.部件.len()],
            字标记: vec![0; 模型.字.len()],
            轮次: 0,
            待根: Vec::new(),
            待部件: Vec::new(),
            待字: Vec::new(),
        }
    }
    fn 清空(&mut self) {
        self.翻转.clear();
        self.安排.clear();
        self.拆分.clear();
        self.选根.clear();
        self.重复.clear();
        self.键.clear();
        self.部件码.clear();
        self.编码.clear();
        self.待根.clear();
        self.待部件.clear();
        self.待字.clear();
        self.轮次 = self.轮次.wrapping_add(1);
        if self.轮次 == 0 {
            self.根标记.fill(0);
            self.部件标记.fill(0);
            self.重选标记.fill(0);
            self.字标记.fill(0);
            self.轮次 = 1;
        }
    }
    fn 改安排(&mut self, 状态: &mut 状态, 根: usize, 新: usize) {
        let 旧 = 状态.决策.安排[根];
        if 新 != 旧 {
            self.安排.push((根, 旧));
            状态.决策.安排[根] = 新;
        }
    }
    fn 标根(&mut self, 根: usize) {
        if self.根标记[根] != self.轮次 {
            self.根标记[根] = self.轮次;
            self.待根.push(根);
        }
    }
    fn 标部件(&mut self, 件: usize, 重选: bool) {
        if self.部件标记[件] != self.轮次 {
            self.部件标记[件] = self.轮次;
            self.待部件.push(件);
        }
        if 重选 {
            self.重选标记[件] = self.轮次;
        }
    }
    fn 标字(&mut self, 字: usize) {
        if self.字标记[字] != self.轮次 {
            self.字标记[字] = self.轮次;
            self.待字.push(字);
        }
    }
    fn 回退(&self, 状态: &mut 状态) {
        for &号 in &self.翻转 {
            状态.已选[号 / 64] ^= 1 << (号 % 64);
        }
        for &(i, 码) in self.编码.iter().rev() {
            状态.换码(i, 码);
        }
        for &(i, 值) in self.部件码.iter().rev() {
            状态.部件码[i] = 值;
        }
        for &(i, 值) in self.键.iter().rev() {
            状态.键[i] = 值;
        }
        for &(i, 值) in self.选根.iter().rev() {
            状态.决策.选根[i] = 值;
        }
        for &(i, 值) in self.拆分.iter().rev() {
            状态.决策.拆分[i] = 值;
        }
        for &(i, 值) in self.重复.iter().rev() {
            状态.决策.重复[i] = 值;
        }
        for &(i, 值) in self.安排.iter().rev() {
            状态.决策.安排[i] = 值;
        }
    }
}

/// 每线程独立 SmallRng，使用库的种子展开避免相邻线程共享重叠随机序列。
struct 随机 {
    状态: SmallRng,
}
impl 随机 {
    fn 新建(种子: u64) -> Self {
        Self {
            状态: SmallRng::seed_from_u64(种子),
        }
    }
    fn 范围(&mut self, 上限: usize) -> usize {
        self.状态.random_range(0..上限)
    }
    fn 小数(&mut self) -> f64 {
        self.状态.random()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn 小模型() -> 上下文 {
        let a = 键号("a").unwrap();
        let o = 键号("o").unwrap();
        let b = 键号("b").unwrap();
        let 根 = vec![
            根 {
                名称: "1".into(),
                落键: 键域::小集合,
                核心: true,
                空间: vec![布局::键(a)],
            },
            根 {
                名称: "2".into(),
                落键: 键域::小集合,
                核心: true,
                空间: vec![布局::键(o)],
            },
            根 {
                名称: "字形-00002".into(),
                落键: 键域::全集合,
                核心: false,
                空间: vec![
                    布局::键(a),
                    布局::键(3),
                    布局::键(b),
                    布局::归并(4),
                    布局::声(b, None),
                ],
            },
            根 {
                名称: "字形-f0000".into(),
                落键: 键域::小集合,
                核心: false,
                空间: vec![布局::关闭, 布局::键(5)],
            },
            根 {
                名称: "字形-00004".into(),
                落键: 键域::小集合,
                核心: true,
                空间: std::iter::once(布局::归并(0))
                    .chain((1..=8).map(布局::键))
                    .collect(),
            },
            根 {
                名称: "字形-00005".into(),
                落键: 键域::全集合,
                核心: false,
                空间: vec![布局::归并(2)],
            },
        ];
        let 归并 = 归并索引::新建(&根);
        let mut 模型 = 上下文 {
            零声母: None,
            同笔主根: None,
            原配置: Value::Null,
            配置: 配置::default(),
            输入校验: 0,
            冲突: 冲突索引::新建([2, 1].into_iter(), &[]).unwrap(),
            根,
            声母锚点: vec![],
            归并,
            候选: vec![
                候选 { 根: vec![2, 3] },
                候选 { 根: vec![0] },
                候选 { 根: vec![1] },
                候选 { 根: vec![0] },
            ],
            部件: vec![
                部件数据 {
                    id: 0,
                    候选: 0..3,
                    拆分: vec![vec![0], vec![1, 2]],
                },
                部件数据 {
                    id: 1,
                    候选: 3..4,
                    拆分: vec![vec![3]],
                },
            ],
            字: vec![
                字数据 {
                    字: '甲',
                    声: b,
                    同字声键: 0,
                    来源: [来源::空; 3],
                    整字: Some(0),
                },
                字数据 {
                    字: '乙',
                    声: b,
                    同字声键: 0,
                    来源: [来源::根(5), 来源::部件(1, 0), 来源::空],
                    整字: None,
                },
                字数据 {
                    字: '丙',
                    声: b,
                    同字声键: 0,
                    来源: [来源::空; 3],
                    整字: Some(1),
                },
            ],
            根部件: vec![vec![0, 1], vec![0], vec![0], vec![0], vec![], vec![]],
            根字: vec![vec![], vec![], vec![], vec![], vec![], vec![1]],
            部件字: vec![vec![0], vec![1, 2]],
            部件整字: vec![vec![0], vec![2]],
            可换拆分: vec![0],
            可重复: vec![0, 1],
            备选: vec![3],
            初始: 决策 {
                安排: vec![0; 6],
                拆分: vec![0; 2],
                选根: vec![无根; 4],
                重复: vec![1; 2],
            },
        };
        模型.初始 = 状态::新建(&模型, 模型.初始.clone(), false).unwrap().决策;
        模型
    }
    #[test]
    fn 同键归并计入组数且同时改键沿归并链传播() {
        let 模型 = 小模型();
        let mut 状态 = 状态::新建(&模型, 模型.初始.clone(), true).unwrap();
        let mut 工作 = 工作区::新建(&模型);
        let mut 随机 = 随机::新建(7);
        let 原小组 = 状态.小组;

        工作.清空();
        工作.安排.push((2, 0));
        状态.决策.安排[2] = 3;
        assert!(模型.更新(&mut 状态, &mut 工作, &mut 随机, false));
        assert!(工作.键.is_empty() && 工作.待字.is_empty());
        assert_eq!(状态.小组, 原小组 - 1);
        状态.核验(&模型).unwrap();
        工作.回退(&mut 状态);
        状态.小组 = 原小组;

        工作.清空();
        // 子根先写入试探，父根仍须先解析新键。
        工作.安排.extend([(2, 0), (4, 0)]);
        状态.决策.安排[2] = 3;
        状态.决策.安排[4] = 键号("o").unwrap() as usize;
        assert!(模型.更新(&mut 状态, &mut 工作, &mut 随机, false));
        assert_eq!(状态.小组, 原小组);
        assert_eq!(状态.键[5], 键号("o").unwrap());
        状态.核验(&模型).unwrap();
    }
    #[test]
    fn 跨区修复和回退保持最少根及计数() {
        let 模型 = 小模型();
        let mut 状态 = 状态::新建(&模型, 模型.初始.clone(), true).unwrap();
        assert_eq!(状态.重码, 1);
        let 原决策 = 状态.决策.clone();
        let 原组 = (状态.小组, 状态.补组);
        let mut 工作 = 工作区::新建(&模型);
        工作.清空();
        工作.安排.push((2, 0));
        状态.决策.安排[2] = 2;
        assert!(模型.更新(&mut 状态, &mut 工作, &mut 随机::新建(7), false));
        assert_eq!(状态.决策.拆分[0], 1);
        assert_eq!(状态.决策.选根[0], 无根);
        assert_eq!(状态.重码, 0);
        状态.核验(&模型).unwrap();
        工作.回退(&mut 状态);
        (状态.小组, 状态.补组) = 原组;
        assert_eq!(状态.决策, 原决策);
        状态.核验(&模型).unwrap();
    }
    #[test]
    fn 字根字重复不改变他字的部件取码() {
        let 模型 = 小模型();
        let mut 状态 = 状态::新建(&模型, 模型.初始.clone(), true).unwrap();
        let 乙 = 状态.码[1];
        let 丙 = 状态.码[2];
        let mut 工作 = 工作区::新建(&模型);
        工作.清空();
        工作.重复.push((1, 1));
        状态.决策.重复[1] = 3;
        模型.更新(&mut 状态, &mut 工作, &mut 随机::新建(8), false);
        assert_eq!(工作.待字, vec![2]);
        assert_eq!(状态.码[1], 乙);
        assert_ne!(状态.码[2], 丙);
        状态.核验(&模型).unwrap();
    }
    #[test]
    fn 混合变异逐步对照全量并核查备选可撤回() {
        let 模型 = 小模型();
        let mut 状态 = 状态::新建(&模型, 模型.初始.clone(), true).unwrap();
        let mut 工作 = 工作区::新建(&模型);
        let mut 随机 = 随机::新建(19);
        let mut 曾启用 = false;
        let mut 曾关闭 = false;
        for _ in 0..1000 {
            let 原 = 状态.决策.clone();
            let 原组 = (状态.小组, 状态.补组);
            工作.清空();
            模型.提议(&mut 状态, &mut 工作, &mut 随机, &[2, 4]);
            let 合法 = 模型.更新(&mut 状态, &mut 工作, &mut 随机, false);
            if !合法 || 随机.范围(2) == 0 {
                工作.回退(&mut 状态);
                (状态.小组, 状态.补组) = 原组;
                assert_eq!(状态.决策, 原);
            } else {
                曾启用 |= 原.安排[3] == 0 && 状态.决策.安排[3] != 0;
                曾关闭 |= 原.安排[3] != 0 && 状态.决策.安排[3] == 0;
            }
            状态.核验(&模型).unwrap();
        }
        assert!(曾启用 && 曾关闭);
    }
    #[test]
    fn 核心与复合体只能联动交换目标的键域() {
        let mut 模型 = 小模型();
        let 小键 = 键号("a").unwrap();
        let 补键 = 键号("b").unwrap();
        模型.根[2].空间 = vec![布局::键(小键), 布局::键(补键)];
        模型.根[4].落键 = 键域::全集合;
        模型.根[4].核心 = false;
        模型.根[4].空间 = vec![布局::键(补键), 布局::键(小键)];
        模型.根[5].落键 = 键域::补集合;
        模型.根[5].空间 = vec![布局::归并(4), 布局::归并(2)];
        模型.根.push(根 {
            名称: "核心根".into(),
            落键: 键域::小集合,
            核心: true,
            空间: vec![布局::归并(2), 布局::归并(4)],
        });
        模型.根部件.push(vec![]);
        模型.根字.push(vec![]);
        模型.初始.安排.push(0);
        模型.归并 = 归并索引::新建(&模型.根);
        let mut 状态 = 状态::新建(&模型, 模型.初始.clone(), true).unwrap();
        let mut 工作 = 工作区::新建(&模型);
        let mut 随机 = 随机::新建(24);
        for r in [2, 4, 5, 6] {
            工作.清空();
            工作.改安排(&mut 状态, r, 1);
            assert!(状态::新建(&模型, 状态.决策.clone(), false).is_err());
            assert!(!模型.更新(&mut 状态, &mut 工作, &mut 随机, false));
            工作.回退(&mut 状态);
            状态.核验(&模型).unwrap();
        }
        let 分量 = &模型.归并.联动[0];
        assert_eq!(分量.len(), 4);
        assert!([2, 4, 5, 6].iter().all(|r| 分量.contains(r)));
        let mut 提议联动 = false;
        for _ in 0..8192 {
            工作.清空();
            模型.提议(&mut 状态, &mut 工作, &mut 随机, &[]);
            if 分量.iter().all(|&r| 状态.决策.安排[r] == 1) {
                提议联动 = true;
                break;
            }
            工作.回退(&mut 状态);
        }
        assert!(提议联动);
        assert!(模型.更新(&mut 状态, &mut 工作, &mut 随机, false));
        assert_eq!(状态.键[5], 补键);
        assert_eq!(状态.键[6], 小键);
        状态.核验(&模型).unwrap();
    }
    #[test]
    fn 候选双向归并允许但选中环及关闭目标会回滚() {
        let mut 模型 = 小模型();
        let 双向 = 模型.根[4].空间.len();
        模型.根[4].空间.extend([布局::归并(2), 布局::归并(3)]);
        模型.归并 = 归并索引::新建(&模型.根);
        let mut 状态 = 状态::新建(&模型, 模型.初始.clone(), true).unwrap();
        let mut 工作 = 工作区::新建(&模型);
        let mut 随机 = 随机::新建(28);

        工作.清空();
        工作.改安排(&mut 状态, 2, 3);
        工作.改安排(&mut 状态, 4, 双向);
        assert!(!模型.更新(&mut 状态, &mut 工作, &mut 随机, false));
        工作.回退(&mut 状态);
        状态.核验(&模型).unwrap();

        工作.清空();
        工作.改安排(&mut 状态, 4, 双向 + 1);
        assert!(!模型.更新(&mut 状态, &mut 工作, &mut 随机, false));
        工作.回退(&mut 状态);
        工作.清空();
        工作.改安排(&mut 状态, 4, 双向 + 1);
        工作.改安排(&mut 状态, 3, 1);
        assert!(模型.更新(&mut 状态, &mut 工作, &mut 随机, true));
        状态.核验(&模型).unwrap();

        工作.清空();
        工作.改安排(&mut 状态, 3, 0);
        assert!(!模型.更新(&mut 状态, &mut 工作, &mut 随机, false));
        工作.回退(&mut 状态);
        状态.核验(&模型).unwrap();
    }
    #[test]
    fn 预算仅惩罚超额且同键独立字根分别计组() {
        let 模型 = 小模型();
        let 状态 = 状态::新建(&模型, 模型.初始.clone(), true).unwrap();
        assert_eq!(状态.小组, 3);
        assert_eq!(状态.指标(预算::default()).分数, 1.0);
        assert_eq!(
            状态
                .指标(预算 {
                    小集合: 1,
                    补集合: 0,
                    惩罚: 2.0
                })
                .分数,
            9.0
        );
    }
}
