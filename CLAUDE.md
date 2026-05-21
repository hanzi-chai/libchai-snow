# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目简介

`libchai-snow` 是一个 Rust 库及可执行文件集合，用于通过模拟退火算法优化中文输入法键位方案。专为「冰雪」系列输入方案定制，依赖位于 `../libchai` 的 `libchai` crate 作为核心框架。

代码库中的标识符（结构体名、方法名、字段名、变量名）均使用中文，这是有意为之，应当保持。

## 构建与运行

```bash
# 构建所有目标（发布模式）
cargo build --release

# 运行测试
cargo test

# 拉取资产文件（key_distribution.txt、pair_equivalence.txt）
make fetch
```

各方案的 Makefile 目标前缀：

| 方案 | 调试编码 | 发布编码 | 单线程优化 | 多线程优化 |
|------|----------|----------|------------|------------|
| 冰雪飞花 | `fed` | `fe` | `fo` | `fp` |
| 冰雪二拼 | `s2d` | `s2e` | `s2o` | `s2p` |
| 冰雪清韵 | `qyd` | `qye` | `qyo` | `qyp` |

多线程运行时，各线程进度日志写入 `output-<时间戳>/<线程序号>/log.txt`，汇总写入 `output-<时间戳>/总结.txt`。

## 架构说明

### 模块结构

库（`src/lib.rs`）暴露四个方案模块与一个公共模块：

- **`src/common.rs`** — 定义 `转换` trait，将编码哈希为 `usize` 索引，用于重码统计数组
- **`src/feihua/`** — 冰雪飞花方案（四码音形混合）
- **`src/snow2/`** — 冰雪二拼方案（双码音码）
- **`src/qingyun/`** — 冰雪清韵方案（最复杂，含独立的 `context.rs`）
- **`src/snow4/`** — 冰雪 v4 方案（仅有操作算子，开发中）

每个方案模块遵循相同的四文件结构：
- `mod.rs` — 定义方案的 `上下文`、`决策`、`决策空间` 及编码类型
- `encoder.rs` — 实现 `编码器` trait，将线性化决策填入编码数组
- `objective.rs` — 实现 `目标函数` trait，计算各项指标与综合分数
- `operators.rs` — 实现 `变异` trait，提供模拟退火的邻域移动

### 核心概念

- **`决策`**：将元素（字根、音码分量）映射到键位。线性化为 `Vec` 供编码器快速访问。
- **`上下文`**：持有解析后的配置、初始决策、决策空间（合法移动）、棱镜（双向元素↔数字与键↔数字映射）以及词列表。
- **`棱镜`**：来自 `libchai` 的核心查找结构，存储元素名与整数、键盘键与整数的双向映射。
- **`编码空间`**：以哈希码为索引的平坦数组，用于重码计数。
- **模拟退火主循环**完全位于 `libchai` 中；各可执行文件只需组装对应方案的上下文/目标函数/操作算子，调用 `优化方法.优化(...)` 即可。

### 可执行文件入口

- `src/bin/feihua.rs` — 冰雪飞花
- `src/bin/qingyun.rs` — 冰雪清韵
- `src/bin/snow2.rs` — 冰雪二拼

三者模式相同：解析 CLI 参数 → 构建上下文 → 匹配 `encode`/`optimize` 命令 → 多线程优化。

### 输入文件

- `project-feihua/config.yaml` — 方案配置（拆分规则、键位映射、优化参数）
- `project-feihua/elements.yaml` — 元素（汉字拆分）序列列表
- `assets/key_distribution.txt`、`assets/pair_equivalence.txt` — 手指负荷与键对当量数据（通过 `make fetch` 获取）

### 输出文件

- `code.txt` — 标准码表（词条 + 编码）
- `dazhu.txt` — 大竹格式码表
- `分析.md` — 前 N 字重码与差指法分析
- `checkpoint-*.yaml` — 优化过程快照
- `solution-*.yaml` / `solution-*.txt` — 各线程最终解

### 脚本（`scripts/`）

TypeScript 数据预处理工具（使用 `npx tsx` 运行）：
- `解析映射.ts`、`解析繁体字频.ts` — 解析键位映射与繁体字频
- `处理陆标繁体.ts` — 处理陆标繁体字数据
- `替换火焰图.ts` — 火焰图标注辅助工具
