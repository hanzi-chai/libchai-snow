# AGENTS.md

**总是用中文回复。**

## 项目简介

`libchai-snow` 是一个 Rust 库及可执行文件集合，用于通过模拟退火算法优化中文输入法键位方案。专为「冰雪」系列输入方案定制。Cargo 包名和库名为 `snow`，使用 Rust 2024 edition。

核心框架依赖为 `libchai = { version = "0.4.0", path = "../libchai" }`，在 Rust 源码中以 `chai` 引用。构建前需要同级目录中的 `../libchai`，仅从 crates.io 下载依赖不能代替这个路径依赖。

代码库中的标识符（结构体名、方法名、字段名、变量名）均使用中文，这是有意为之，应当保持。

## 构建与运行

在仓库根目录执行命令。需要支持 Rust 2024 edition 的工具链；`.cargo/config.toml` 设置了 `target-cpu=native`，构建产物针对本机 CPU 优化。

```bash
# 构建所有目标（发布模式）
cargo build --release

# 运行测试
cargo test

# 拉取资产文件（key_distribution.txt、pair_equivalence.txt）
make fetch
```

目前 `tests/check_threads.rs` 仅检查可用并行线程数大于零，不能验证编码或优化结果。修改编码器、目标函数或操作算子后，应在具备对应方案数据时运行编码命令检查码表和指标；缺少数据时说明未验证的部分。

各方案的 Makefile 目标前缀：

| 方案 | 调试编码 | 发布编码 | 单线程优化 | 多线程优化 |
|------|----------|----------|------------|------------|
| 冰雪飞花 | `fed` | `fe` | `fo` | `fp` |
| 冰雪二拼 | `s2d` | `s2e` | `s2o` | `s2p` |
| 冰雪清韵 | `qyd` | `qye` | `qyo` | `qyp` |

`fp`、`s2p`、`qyp` 固定使用 10 线程。需要其他线程数时，参考 Makefile 直接运行 `cargo run --release --bin <方案> -- optimize ... -t <线程数>`。完整 CLI 参数由 `../libchai` 提供，可用 `cargo run --bin <方案> -- --help` 查看。

优化默认写入 `output-<时间戳>/`，多线程运行时各线程进度日志为 `<线程序号>/log.txt`，汇总为 `总结.txt`。优化可能耗时较长，不作为日常检查命令。

## 架构说明

### 方案设计文档

各方案的设计文档统一放在相应源码文件夹的 `DESIGN.md` 中，即 `src/<方案模块>/DESIGN.md`（例如 [冰雪飞花](src/feihua/DESIGN.md)）。修改某方案前先阅读其设计文档；新增方案或调整设计时，同步维护对应文档，说明设计目标、取码规则、关键约束以及方案规则与当前实现的边界。

### 模块结构

库（`src/lib.rs`）暴露四个方案模块与一个公共模块：

- **`src/common.rs`** — 定义 `转换` trait，将编码哈希为 `usize` 索引，用于重码统计数组；另提供从根目录 `pua_mapper.yaml` 读取映射的 `get_pua_mapper()`
- **`src/feihua/`** — 冰雪飞花方案（四码音形混合）
- **`src/snow2/`** — 冰雪二拼方案（双码音码）
- **`src/qingyun/`** — 冰雪清韵方案（最复杂，含独立的 `context.rs`）
- **`src/snow4/`** — 冰雪四拼手机键位布局操作算子，实现在 `snow4operators.rs`；没有对应的可执行文件、编码器和目标函数

飞花、二拼、清韵三个完整方案的主要职责划分如下：

- `mod.rs` — 定义方案的 `上下文`、`决策`、`决策空间` 及编码类型
- `encoder.rs` — 实现编码逻辑，将线性化决策填入编码数组
- `objective.rs` — 实现 `目标函数` trait，计算各项指标与综合分数
- `operators.rs` — 实现 `变异` trait，提供模拟退火的邻域移动

清韵的上下文及码表生成、分析逻辑单独放在 `src/qingyun/context.rs`，不在 `mod.rs`。

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

三者均解析 CLI 参数、构建上下文，再处理 `encode` 或 `optimize`。`encode` 计算初始决策的指标，是否导出码表取决于方案；`optimize` 按线程数启动优化并汇总分数。

### 输入文件

- `project-feihua/config.yaml`、`project-feihua/elements.yaml` — 飞花配置及元素序列
- `project-snow2/config.yaml`、`project-snow2/elements.txt` — 二拼配置及元素序列
- `project-qingyun/config.yaml`、`project-qingyun/elements.txt` — 清韵配置及元素序列
- `assets/key_distribution.txt`、`assets/pair_equivalence.txt` — 手指负荷与键对当量数据（通过 `make fetch` 获取）

清韵还会按代码路径读取根目录 `data/` 中的数据，包括 `rules.yaml`、`pinyin.txt`、`dynamic_analysis.yaml`、`ftzp.txt`、`t2tw.txt`、`拆分结果.txt`、`简词.txt` 和 `override.yaml`。`make fetch` 只获取上述两个资产文件，不会准备方案配置或这些数据。

方案配置、元素序列及辅助数据未随仓库提交。`.gitignore` 忽略所有 `*.txt`、`*.yaml`，以及 `output/`、`output-*`；不要因文件缺失而伪造输入，也不要把运行产物当作源码提交。

### 输出文件

- 飞花：`encode` 向 `project-feihua/` 写入 `code.txt` 和 `dazhu.txt`；优化时向各线程输出目录写入这两份码表及 `分析.md`，并导出配置。
- 清韵：`encode` 默认向 `output/` 写入 `冰雪清韵.txt`、`大竹码表.txt`、`形码盒子测评码表.txt`、`snow_qingyun.fixed.txt` 及 `分析.md`；优化时写入各线程输出目录。
- 二拼：当前入口的 `encode` 仅打印分数与指标，没有码表导出调用。
- 优化框架负责写入 `checkpoint-*.yaml`、`solution-*.yaml` / `solution-*.txt` 等文件，具体命名与触发条件以 `../libchai/src/interfaces/command_line.rs` 为准。

### 脚本（`scripts/`）

TypeScript 数据预处理工具：依赖声明在 `scripts/package.json`，可先执行 `npm ci --prefix scripts`，再从仓库根目录使用 `npx tsx scripts/<脚本名>.ts` 运行。`tsx` 未在该包中声明，需由执行环境提供或由 `npx` 获取。脚本中的相对路径以当前工作目录为基准，运行前检查输入和输出路径。

- `解析映射.ts`、`解析繁体字频.ts` — 解析键位映射与繁体字频
- `处理陆标繁体.ts` — 处理陆标繁体字数据
- `替换火焰图.ts` — 火焰图标注辅助工具

`scripts/package.json` 的 `test` 只是报错占位命令，不是有效的测试套件。
