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

飞花手写输入位于 `src/feihua/config.yaml`，预处理只向 `output/feihua/generated.yaml` 写入合成配置（js-yaml 序列化、不保留注释），后续编码、优化、报告和预算基准使用生成配置，不回写手写输入。静态／动态分析及专用 Rust 编码的准备和验证命令见 [DESIGN.md](src/feihua/DESIGN.md)。当前使用无参数的 `bun src/feihua/prepare.ts` 从手写配置全量重算，Rust 默认入口调用其 `准备飞花` 函数重算后读取 `input.json`，也可显式传 `--tree-data` 使用已生成的自包含输入，具体命令见设计文档；维护脚本统一使用 `.ts`。以下 `project-*/` 路径为原有 Makefile 工作流。

- `project-feihua/config.yaml`、`project-feihua/elements.yaml` — 飞花配置及元素序列
- `project-snow2/config.yaml`、`project-snow2/elements.txt` — 二拼配置及元素序列
- `project-qingyun/config.yaml`、`project-qingyun/elements.txt` — 清韵配置及元素序列
- `assets/key_distribution.txt`、`assets/pair_equivalence.txt` — 手指负荷与键对当量数据（通过 `make fetch` 获取）

清韵还会按代码路径读取根目录 `data/` 中的数据，包括 `rules.yaml`、`pinyin.txt`、`dynamic_analysis.yaml`、`ftzp.txt`、`t2tw.txt`、`拆分结果.txt`、`简词.txt` 和 `override.yaml`。`make fetch` 只获取上述两个资产文件，不会准备方案配置或这些数据。

除 `src/feihua/config.yaml` 外，上述方案配置、元素序列及辅助数据未随仓库提交。`.gitignore` 忽略所有 `*.txt`、`*.yaml`，以及 `output/`、`output-*`；不要因文件缺失而伪造输入，也不要把运行产物当作源码提交。

### 输出文件

- 飞花：`encode` 向 `output/feihua-encoding/` 写入 `code.txt`、`dazhu.txt` 和 `splits.yaml`；优化结果保存在 `output/feihua-optimization/<时间戳>/` 的各线程目录，包含码表、配置、拆分和分析。
- 清韵：`encode` 默认向 `output/` 写入 `冰雪清韵.txt`、`大竹码表.txt`、`形码盒子测评码表.txt`、`snow_qingyun.fixed.txt` 及 `分析.md`；优化时写入各线程输出目录。
- 二拼：当前入口的 `encode` 仅打印分数与指标，没有码表导出调用。
- 优化框架负责写入 `checkpoint-*.yaml`、`solution-*.yaml` / `solution-*.txt` 等文件，具体命名与触发条件以 `../libchai/src/interfaces/command_line.rs` 为准。

### 脚本（`scripts/`）

脚本入口和输入输出约定见 [飞花 DESIGN](src/feihua/DESIGN.md) 及各工具目录内的说明。维护脚本不编写测试文件，使用类型检查、实际运行和同一输入下的结果对比验证。文件路径统一使用英文，通用运行产物写入 `output/`，生成 JSON 的 `--out` 必须位于该目录内。飞花工具与人工预处理资料位于 `src/feihua/`，预处理最终文件统一写入 `output/feihua/`；完整入口为 `bun src/feihua/prepare.ts`；每次全量重算，不使用指纹或旧产物缓存，预处理只写 `output/feihua/index.html`、`generated.yaml`、`input.json`，中间结果只在进程内传递。

飞花先按初始方案冻结正余根位置，再核查单元、枚举字根和合法拆分，证明任意启用备选子集后重新运行算法也不改变初始完整正根，剔除危险候选后更新配置，最后校验五类空间与永久 B 覆盖。意符优先仅在 `src/feihua/prepare.ts` 中保留最终84字名单，不保存字源审核数据。优化不重新取正部，同一优化轨迹只能加根不能删根。B区只允许加载方案并融合后的部件字根，复合体在当前、所有空间及归并链上均不得进入B；运行时也须校验。

TypeScript 数据预处理工具：依赖声明在根 `package.json`，先在相邻 `../hanzi-chai` 根目录执行 `bun run build`，再在本仓库根目录执行 `npm ci`。`hanzi-chai` 通过 `file:../hanzi-chai` 引用相邻根包，脚本统一从 `hanzi-chai` 导入；读取原始字库时使用其公开的 `默认数据目录` 中的 UTF-8 JSON，不访问旧子包路径或 `.deflate` 文件。Bundler 类型解析通过 `customConditions: ["node"]` 选择 Node 入口，使用 ES2022 与 Node 全局类型；`output/`、`output-*` 是历史产物，不参与类型检查。随后从仓库根目录使用 `npx tsx scripts/<脚本名>.ts` 运行。`tsx` 未在该包中声明，需由执行环境提供或由 `npx` 获取。脚本中的相对路径以当前工作目录为基准，运行前检查输入和输出路径。



根 `package.json` 的 `test` 只是报错占位命令，不是有效的测试套件。

### TypeScript 与 Biome 规范

- 新增或修改 `.ts` 脚本以及配套 HTML 模板后，必须在仓库根目录运行 `npm run check`，保证全仓 Biome 检查零错误、零警告；该命令将警告视为失败。可先运行 `npm run check:fix` 应用安全修复，再人工处理剩余问题。不能通过关闭规则、扩大忽略范围或批量添加 `biome-ignore` 绕过检查。
- 格式与导入顺序以根目录 `biome.json` 为准，使用 Biome 自动格式化；类型导入使用 `import type` 或内联 `type`，及时移除未使用的导入、变量和参数，保留项目中文标识符。字符串插值使用模板字符串。
- 禁止显式 `any`，优先使用类型推断、具体结构、`hanzi-chai` 导出的类型及 `ReturnType`。外部 JSON 先作为 `unknown` 读取并收窄、校验；本流程生成的 JSON 按 `prepare.ts` 中的流程结构类型传递，结构变化时同步更新类型，不让无类型数据传播到回调和算法内部。
- 禁止使用非空断言 `!`。已经由类型保证存在的数组元素直接访问；`Map.get`、`find`、`at`、迭代器结果和可选字段先判断或用 `prepare.ts` 的 `确保存在` 检查。必需数据缺失应报错，不用可选链或默认空值掩盖错误；合法的 `false`、`0` 和空字符串不能误判为缺失。联合类型先按 `typeof`、`Array.isArray` 或 `in` 收窄再访问字段。
- `forEach` 等副作用回调使用语句块，不隐式返回赋值或其他回调调用；复杂遍历优先使用 `for...of`。赋值独立成语句，不嵌入表达式。
- HTML 模板也须通过 Biome：按钮明确写 `type="button"`，数据占位符使用合法 JSON 字符串（当前为 `"{{data}}"`），生成器整体替换带引号的占位符并继续转义 `<`，保证模板可解析且生成报告的数据结构不变。
- Biome 通过后还需执行 TypeScript 类型检查（使用环境中的 TypeScript 执行 `tsc --noEmit`），并按既有约定实际运行受影响入口、对比同输入结果；不要把 Biome 检查当作类型或行为验证，也不要无差别应用 `--unsafe` 修复。

飞花流程统一由无参数的 `bun src/feihua/prepare.ts` 驱动，一次性完成空间检查、冻结分部、读取单元分组、枚举与备选证明、永久 B 覆盖、Rust 输入及统一报告；优化结果重载由 Rust 入口调用 `prepare.ts` 的 `准备飞花` 全量重算后恢复决策。预处理逻辑全部收敛到 `prepare.ts`，删除原 common、glyphs、geometry、units、roots、space 脚本及 report.html，报告模板和浏览器脚本一并内联。每次从手写配置全量开始，本轮共用字库、固定结构及完整合法拆分，阶段间直接传递对象；永久 B 覆盖不足时列出具体部件并报错停止，禁止自动补根、改键或收窄空间。实际命令及文件说明统一见 DESIGN。范围为加载配置后 minimal 的全部原生／融合部件；直接使用 `units.json` 的既有分组枚举；预处理不自动识别开口框、平行或对称，也不按状态执行视觉复核。认同按默认退化规则双向匹配完整笔形，不使用口／囗、土／士、长短二横特殊逻辑；单笔按 classifier 和 1–5 比较。人工单元记录使用英文字段，id 为五位小写十六进制字符串（不带字形前缀），不含名称、图册、位置；读取由 prepare.ts 统一转换为内部格式，按字形 ID 复用，不保存或校验字形指纹及规则指纹；仅检查当前部件有记录、单元至少两笔、笔序有效且分组不重叠。源码、配置及人工维护的 `units.json` 纳入版本管理，预处理仅输出 `output/feihua/` 中的三个最终文件，编码和优化产物分别位于 `output/feihua-encoding/` 和 `output/feihua-optimization/`，由 `.gitignore` 忽略。
