import { spawnSync } from "node:child_process";
import {
	mkdirSync,
	mkdtempSync,
	readFileSync,
	renameSync,
	rmSync,
	writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import {
	合并分类器,
	合并拼写运算,
	type 字形,
	type 字形拼写运算,
	字符,
	应用拼写运算,
	是部件,
	type 模式,
	type 笔画名称,
	生成笔画路径,
	获取原始词典,
	获取字库,
	获取融合字形集合,
	读取配置,
	部件,
} from "hanzi-chai";
import { 查重排, 确保存在, 笔形签名, 重排特征, type 飞花输入 } from "./prepare.ts";
import { 生成基本拆分分页图 } from "./report-pdf.ts";
import { type 报告, type 报告字形, 比较笔画顺序, 生成报告 } from "./report.ts";

const 用法 =
	"bun src/feihua/postprocess.ts --solution <solution.json> [--tree-data output/feihua/input.json] [--encoder target/release/feihua] [--inkscape inkscape] [--b-budget 50] [--complement-budget 30] [--budget-penalty 1]\n固定输出至 output/feihua-current/：feihua-beta.json、report.html、report.json、root-chart.svg/png/pdf、component-splits.pdf 及当前解文件";
if (process.argv.slice(2).includes("--help")) {
	console.log(用法);
	process.exit(0);
}
const 参数 = new Map<string, string>();
for (let i = 2; i < process.argv.length; i += 2) {
	const 名 = process.argv[i];
	const 值 = process.argv[i + 1];
	if (
		![
			"--solution",
			"--tree-data",
			"--encoder",
			"--inkscape",
			"--b-budget",
			"--complement-budget",
			"--budget-penalty",
		].includes(名) ||
		值 === undefined ||
		参数.has(名)
	)
		throw Error(用法);
	参数.set(名, 值);
}
const 输入路径 = resolve(参数.get("--tree-data") ?? "output/feihua/input.json");
const 快照路径 = resolve(确保存在(参数.get("--solution"), 用法));
const 输出目录 = resolve("output/feihua-current");
const 编码目录 = mkdtempSync(join(tmpdir(), "feihua-postprocess-"));
process.on("exit", () => rmSync(编码目录, { recursive: true, force: true }));
const 输入字节 = readFileSync(输入路径);
const 快照字节 = readFileSync(快照路径);
// 由 Rust 检查输入校验、索引、最少根约束，再生成本次的权威码表与键位配置。
const 编码 = spawnSync(
	resolve(参数.get("--encoder") ?? "target/release/feihua"),
	[
		"--tree-data",
		输入路径,
		"encode",
		"--solution",
		快照路径,
		"--out",
		编码目录,
		...(
			["--b-budget", "--complement-budget", "--budget-penalty"] as const
		).flatMap((名) => (参数.has(名) ? [名, 确保存在(参数.get(名), 名)] : [])),
	],
	{ stdio: "inherit" },
);
if (编码.error) throw 编码.error;
if (编码.status !== 0) throw Error("Rust 快照校验或编码失败");
const 输入 = JSON.parse(输入字节.toString()) as 飞花输入;
function 记录(值: unknown): Record<string, unknown> {
	if (typeof 值 !== "object" || 值 === null || Array.isArray(值))
		throw Error("无效快照记录");
	return 值 as Record<string, unknown>;
}
function 数组(值: unknown): number[] {
	if (!Array.isArray(值)) throw Error("快照缺少决策数组");
	return 值.map((项: unknown) => {
		if (typeof 项 !== "number" || !Number.isSafeInteger(项) || 项 < 0)
			throw Error("无效快照索引");
		return 项;
	});
}
const 原快照: unknown = JSON.parse(快照字节.toString());
const 决策 = 记录(记录(原快照).决策);
const 拆分 = 数组(决策.拆分);
const 选根 = 数组(决策.选根);
const 重复 = 数组(决策.重复);
const 配置 = 读取配置(resolve(编码目录, "config.yaml"));
const 字库 = 获取字库(配置);
// 准包围是取码视角：显示仍保留原操作数的上下／三分排版。
// 只移除保留操作数的这两类变换，其余方案字形修订正常应用。
function 变量(项: 模式): number | undefined {
	return typeof 项 !== "number" && "variable" in 项 ? 项.variable : undefined;
}
const 掰字形ID = 0xd772f;
function 是准包围({ from, to }: 字形拼写运算): boolean {
	if (typeof to === "number" || !("references" in to)) return false;
	if (typeof from === "number")
		return (
			from === 掰字形ID &&
			to.operator === "⿴" &&
			to.references[0] === 0x403a3 &&
			to.references[1] === 0x0c112
		);
	if (!("references" in from)) return false;
	const 前 = from.references.map(变量);
	if (前.some((项) => 项 === undefined)) return false;
	if (from.operator === "⿱" && to.operator === "⿵")
		return JSON.stringify(前) === JSON.stringify(to.references.map(变量));
	if (
		!["⿲", "⿳"].includes(String(from.operator)) ||
		to.operator !== "⿴" ||
		to.references.length !== 2
	)
		return false;
	const [外, 内] = to.references;
	return (
		typeof 外 !== "number" &&
		"references" in 外 &&
		外.operator === (from.operator === "⿲" ? "⿰" : "⿱") &&
		JSON.stringify(外.references.map(变量)) ===
			JSON.stringify([前[0], 前[2]]) &&
		变量(内) === 前[1]
	);
}
const 准包围运算 = 配置.data?.glyph_algebra?.filter(是准包围) ?? [];
function 匹配范围包含(外: 模式, 内: 模式): boolean {
	if (typeof 外 === "number") return 外 === 内;
	if (!("variable" in 外)) return false;
	if (!外.id_set) return true;
	if (typeof 内 === "number") return 外.id_set.includes(内);
	return (
		"variable" in 内 &&
		内.id_set !== undefined &&
		内.id_set.every((id) => 外.id_set?.includes(id))
	);
}
// 下游运算可能匹配已经准包围化的子形，例如谷的 ⿱⿵八𠆢口 → ⿳八𠆢口。
// 显示移除准包围后，也恢复其匹配模式；仍执行原来的三分转换。
function 显示匹配模式(项: 模式): 模式 {
	if (typeof 项 === "number" || !("references" in 项)) return 项;
	const references = 项.references.map(显示匹配模式);
	const 恢复上下 =
		项.operator === "⿵" &&
		准包围运算.some(({ from, to }) => {
			if (
				typeof from === "number" ||
				!("references" in from) ||
				typeof to === "number" ||
				!("references" in to) ||
				from.operator !== "⿱" ||
				to.operator !== "⿵" ||
				from.references.length !== references.length
			)
				return false;
			return from.references.every((子项, i) =>
				匹配范围包含(子项, references[i]),
			);
		});
	return { ...项, operator: 恢复上下 ? "⿱" : 项.operator, references };
}
const 显示字库 = 获取字库({
	...配置,
	data: {
		...配置.data,
		// 兼容仍在 glyphs 中保存「掰」的准包围覆盖的旧快照。
		glyphs: 配置.data?.glyphs?.filter(({ id }) => id !== 掰字形ID),
		glyph_algebra: 配置.data?.glyph_algebra
			?.filter((运算) => !是准包围(运算))
			.map((运算) => ({ ...运算, from: 显示匹配模式(运算.from) })),
	},
});
const 融合表 = 获取融合字形集合({}, 配置.data);
const 字形缓存 = new Map<number, 字形>();
function 融合(形: 字形): 字形 {
	const 旧 = 字形缓存.get(形.id);
	if (旧) return 旧;
	const 新 = 是部件(形)
		? 形
		: 融合表.has(形.id)
			? new 部件({
					id: 形.id,
					type: "component",
					operator: undefined,
					references: undefined,
					strokes: 形.图形盒子.获取笔画列表(),
				})
			: 形.替换分析部分(形.部分列表.map(融合));
	字形缓存.set(形.id, 新);
	return 新;
}
// 字形运算生成的临时分部只存在于实际树中，不一定有可按 ID 读取的字库记录。
const 原形索引 = new Map<number, 字形>();
function 收集原形(形: 字形) {
	if (原形索引.has(形.id)) return;
	原形索引.set(形.id, 形);
	if (!是部件(形)) for (const 子 of 形.部分列表) 收集原形(子);
}
for (const 字 of 输入.字)
	收集原形(确保存在(字库.获取字形(字.字形), `缺少 minimal 字形 ${字.字}`));
const 取形 = (id: number) =>
	融合(
		确保存在(
			原形索引.get(id) ?? 字库.获取字形(id),
			`缺少字形 ${id.toString(16)}`,
		),
	);
const 键序 = ["AOEIUV;/", "BPMFDTNL", "GKHWJQXY", "ZCSR,."];
const 全键 = 键序.join("").toLowerCase();
const 是根名 = (名称: string) => /^字形-[0-9a-f]{5}$|^[1-5]$/.test(名称);
const 归并缓存 = new Map<string, { 名称: string; 键: string }>();
function 取归并终点(名: string, 链 = new Set<string>()): { 名称: string; 键: string } {
	const 旧 = 归并缓存.get(名);
	if (旧 !== undefined) return 旧;
	if (链.has(名)) throw Error(`归并成环：${名}`);
	const 值 = 确保存在(配置.form?.mapping[名], `未启用字根 ${名}`);
	let 终点: { 名称: string; 键: string };
	if (typeof 值 === "string" && 值.length === 1 && 全键.includes(值)) {
		终点 = { 名称: 名, 键: 值 };
	} else if (!Array.isArray(值) && typeof 值 === "object") {
		const 目标 = 取归并终点(值.element, new Set([...链, 名]));
		// 声母锚点等非字根元素只提供键位。
		终点 = 是根名(值.element) ? 目标 : { 名称: 名, 键: 目标.键 };
	} else throw Error(`不支持的字根安排：${名}`);
	归并缓存.set(名, 终点);
	return 终点;
}
function 取键(名: string): string {
	return 取归并终点(名).键;
}
const 部件表 = new Map<number, { 序号: number; 偏移: number }>();
let 偏移 = 0;
for (const [序号, 件] of 输入.部件.entries()) {
	部件表.set(件.id, { 序号, 偏移 });
	偏移 += 件.候选.length;
}
const 分部表 = new Map(输入.复合体.map((项) => [项.id, 项]));
const 分类器 = 合并分类器(配置.analysis?.classifier);
const 声规则 = 确保存在(合并拼写运算(配置.algebra).get("声"), "缺少声母规则");
const 字音 = new Map<string, string[]>();
for (const 条 of 获取原始词典()) {
	if ([...条.词].length !== 1 || 条.拼音.length !== 1) continue;
	const 拼音 = 条.拼音[0].trim();
	// 笔画、部首等符号可能带空拼音占位；未收读音不能按零声母编码。
	if (!拼音) continue;
	const 列 = 字音.get(条.词) ?? [];
	if (!列.includes(拼音)) 列.push(拼音);
	字音.set(条.词, 列);
}
const 权威码 = new Map<string, Set<string>>();
for (const 行 of readFileSync(resolve(编码目录, "code.txt"), "utf8")
	.trim()
	.split("\n")) {
	const [字, 码] = 行.split("\t");
	const 集 = 权威码.get(字) ?? new Set<string>();
	集.add(码);
	权威码.set(字, 集);
}

type 节点 = { 形: 字形; 笔画: number[]; 子项: 节点[] };
function 定位(形: 字形, 笔画: number[]): 节点 {
	if (是部件(形)) return { 形, 笔画, 子项: [] };
	const 子笔 = 形.部分列表.map((子) =>
		Array<number>(子.图形盒子.获取笔画列表().length).fill(-1),
	);
	let 位 = 0;
	for (const 块 of 形.笔画列表) {
		const 映射 = 子笔[块.index];
		for (let i = 块.from ?? 0; i <= (块.to ?? 映射.length - 1); i++)
			映射[i] = 确保存在(笔画[位++], "复合体笔序超出字形");
	}
	if (位 !== 笔画.length) throw Error("复合体笔序不完整");
	return { 形, 笔画, 子项: 形.部分列表.map((子, i) => 定位(子, 子笔[i])) };
}
function 显示笔序(分析: 节点, 显示: 节点): number[] {
	const 对应 = Array<number>(分析.笔画.length).fill(-1);
	const 已用 = new Set<number>();
	function 配对(前: 节点, 后: 节点) {
		if (是部件(前.形) && 是部件(后.形)) {
			if (前.形.id !== 后.形.id || 前.笔画.length !== 后.笔画.length)
				throw Error(
					`显示部件不对应：${前.形.id.toString(16)} → ${后.形.id.toString(16)}`,
				);
			for (let i = 0; i < 前.笔画.length; i++) {
				const a = 前.笔画[i];
				const b = 后.笔画[i];
				if (a < 0 && b < 0) continue;
				if (a < 0 || b < 0 || 对应[a] !== -1 || 已用.has(b))
					throw Error("显示笔画重复或缺失");
				对应[a] = b;
				已用.add(b);
			}
			return;
		}
		if (前.子项.length === 后.子项.length && 前.子项.length > 0) {
			for (let i = 0; i < 前.子项.length; i++) 配对(前.子项[i], 后.子项[i]);
			return;
		}
		// ⿲／⿳(甲、乙、丙) → ⿴(⿰／⿱(甲、丙)、乙)。按操作数位置还原，不能按同形根出现次序猜测。
		if (
			!是部件(前.形) &&
			!是部件(后.形) &&
			前.形.结构描述字符 === "⿴" &&
			["⿲", "⿳"].includes(后.形.结构描述字符) &&
			前.子项.length === 2 &&
			后.子项.length === 3 &&
			前.子项[0].子项.length === 2
		) {
			配对(前.子项[0].子项[0], 后.子项[0]);
			配对(前.子项[1], 后.子项[1]);
			配对(前.子项[0].子项[1], 后.子项[2]);
			return;
		}
		throw Error(`无法对应分析与显示结构：${前.形.id.toString(16)}`);
	}
	配对(分析, 显示);
	if (对应.some((笔) => 笔 < 0) || 已用.size !== 显示.笔画.length)
		throw Error("显示笔画未完整对应");
	return 对应;
}
type 配色角色 =
	| "initial"
	| "component1"
	| "component2"
	| "component3"
	| "positive"
	| "remainder1"
	| "remainder2";
type 字根出现 = { 名称: string; 笔画: number[] };
function 部件取根(节点: 节点): 字根出现[] {
	const { 序号, 偏移 } = 确保存在(部件表.get(节点.形.id), "找不到预处理部件");
	const 件 = 输入.部件[序号];
	return 件.拆分[拆分[序号]].map((i) => {
		const 候选 = 件.候选[i];
		const 名称 = 输入.字根[选根[偏移 + i]].名称;
		if (!候选.字根.includes(名称) || !"aoeiuv;/".includes(取键(名称)))
			throw Error("所选候选不是 B 字根");
		return {
			名称,
			笔画: 候选.笔画.map((笔) => 节点.笔画[笔 - 1]).filter((笔) => 笔 >= 0),
		};
	});
}
const 签名缓存 = new Map<number, string>();
function 全形签名(件: 部件) {
	let 值 = 签名缓存.get(件.id);
	if (值 === undefined) {
		值 = 笔形签名(
			件,
			件.笔画列表.map((_, i) => i),
		);
		签名缓存.set(件.id, 值);
	}
	return 值;
}
function 固定正根(正部: 节点, 名称: string): 字根出现 {
	const 原形 = 名称.startsWith("字形-")
		? 取形(Number.parseInt(名称.slice(3), 16))
		: undefined;
	let 当前 = 正部;
	while (true) {
		const 匹配 =
			原形 === undefined
				? 是部件(当前.形) &&
					当前.形.笔画数() === 1 &&
					String(分类器[当前.形.笔画列表[0].feature]) === 名称
				: 当前.形.id === 原形.id ||
					(是部件(当前.形) &&
						是部件(原形) &&
						全形签名(当前.形) === 全形签名(原形));
		if (匹配) return { 名称, 笔画: 当前.笔画.filter((笔) => 笔 >= 0) };
		当前 = 确保存在(当前.子项[0], `无法定位完整正根 ${名称}`);
	}
}
function 首部件(节点: 节点): 节点 {
	let 当前 = 节点;
	while (!是部件(当前.形)) 当前 = 确保存在(当前.子项[0], "缺少首部件");
	return 当前;
}
const 根图 = new Map<string, string[]>();
const 笔画名 = ["横", "竖", "撇", "点", "折"];
function 取显示根图(名称: string) {
	let paths = 根图.get(名称);
	if (paths === undefined) {
		const 形 = 确保存在(
			显示字库.获取字形(Number.parseInt(名称.slice(3), 16)),
			`缺少显示字根：${名称}`,
		);
		paths = 生成笔画路径(形.图形盒子.获取笔画列表());
		根图.set(名称, paths);
	}
	return paths;
}
function 根元素(根: 字根出现, colorIndex: number, colorRole: 配色角色) {
	const paths = 根.名称.startsWith("字形-") ? 取显示根图(根.名称) : undefined;
	return {
		label: {
			initial: "声母",
			component1: "字根1",
			component2: "字根2",
			component3: "字根3",
			positive: "正根",
			remainder1: "余根1",
			remainder2: "余根2",
		}[colorRole],
		name: 根.名称.startsWith("字形-") ? 根.名称 : 笔画名[Number(根.名称) - 1],
		key: 取键(根.名称),
		colorIndex,
		colorRole,
		...(paths ? { paths } : {}),
	};
}
type 演示 = {
	paths: string[];
	roots: { strokes: number[]; name: string; colorRole: 配色角色 }[];
	rows: {
		pinyin: string;
		elements: {
			label: string;
			name: string;
			key: string;
			colorIndex: number;
			colorRole: 配色角色;
			paths?: string[];
		}[];
	}[];
};
function 分析字形(id: number) {
	const 形 = 取形(id);
	const 笔画 = 形.图形盒子.获取笔画列表();
	const 树 = 定位(
		形,
		笔画.map((_, i) => i),
	);
	const 原分析形 = 确保存在(原形索引.get(id), "缺少分析字形");
	const 显示形 = 字库.获取字形(id)
		? (显示字库.获取字形(id) ?? 原分析形)
		: 原分析形;
	const 显示笔画 = 显示形.图形盒子.获取笔画列表();
	const 对应 = 显示笔序(
		定位(
			原分析形,
			笔画.map((_, i) => i),
		),
		定位(
			显示形,
			显示笔画.map((_, i) => i),
		),
	);
	let 根: 字根出现[];
	let 次数 = 1;
	let 正部位 = -1;
	const 基本根 = 是部件(形) ? 部件取根(树) : [];
	if (是部件(形)) {
		const 覆盖 = 基本根.flatMap((项) => 项.笔画).sort((甲, 乙) => 甲 - 乙);
		if (JSON.stringify(覆盖) !== JSON.stringify(笔画.map((_, i) => i)))
			throw Error(
				`部件笔画未完整覆盖，请核对输入与当前字库：${id.toString(16)}`,
			);
		根 = 基本根.slice(0, 3);
		if (根.length === 1)
			次数 = 重复[确保存在(部件表.get(形.id), "缺少部件").序号];
	} else {
		const 分部 = 确保存在(分部表.get(形.id), "缺少冻结分部");
		正部位 = 树.子项.findIndex(
			(子, i) =>
				子.形.id === 分部.正部 &&
				JSON.stringify(
					树.子项.filter((_, j) => i !== j).map((项) => 项.形.id),
				) === JSON.stringify(分部.余部),
		);
		if (正部位 < 0) throw Error("冻结正部不匹配");
		const 正部 = 树.子项[正部位];
		if ("字根" in 分部.正根) 根 = [固定正根(正部, 分部.正根.字根)];
		else {
			const 件 = 首部件(正部);
			if (件.形.id !== 分部.正根.部件) throw Error("不完整正根来源不匹配");
			根 = 部件取根(件).slice(0, 1);
		}
		const 余部 = 树.子项.filter((_, i) => i !== 正部位);
		const 余节点 =
			余部.length === 1 && 是部件(余部[0].形)
				? 余部
				: (余部.length === 1 ? 余部[0].子项.slice(0, 2) : 余部).map(首部件);
		if (余节点.length !== 分部.余根.length) throw Error("余根来源数量不匹配");
		for (const [i, 请求] of 分部.余根.entries()) {
			if (余节点[i].形.id !== 请求.部件) throw Error("余根来源部件不匹配");
			根.push(...部件取根(余节点[i]).slice(0, 请求.数量));
		}
	}
	const 根角色: 配色角色[] = 是部件(形)
		? ["component1", "component2", "component3"]
		: ["positive", "remainder1", "remainder2"];
	const 基础形码 = 根.map((项, i) => 根元素(项, i + 2, 根角色[i]));
	const 形码 = 基础形码.flatMap((项) => Array.from({ length: 次数 }, () => 项));
	return {
		形,
		正部位,
		根,
		基本根,
		基础形码,
		形码,
		paths: 生成笔画路径(显示笔画),
		显示笔画,
		roots: 根.map((项, i) => ({
			strokes: 项.笔画.map((笔) => 对应[笔]),
			name: 项.名称,
			colorRole: 根角色[i],
		})),
		完整拆分: 基本根.map((项) => {
			根元素(项, 2, "component1");
			return {
				名称: 项.名称,
				键: 取键(项.名称),
				笔画: 项.笔画.map((笔) => 对应[笔]),
			};
		}),
	};
}

const characters: Record<string, 演示> = {};
const minimal = new Map<number, string[]>();
for (const 字 of 输入.字) {
	const 字列 = minimal.get(字.字形) ?? [];
	if (!字列.includes(字.字)) 字列.push(字.字);
	minimal.set(字.字形, 字列);
}
const 目标字形 = new Set([
	...输入.部件.map((件) => 件.id),
	...输入.复合体.map((形) => 形.id),
]);
const 别名 = new Map<number, Set<string>>();
for (const [, 记录] of 字库) {
	for (const { id } of 记录.final_glyphs) {
		if (!目标字形.has(id)) continue;
		const 字列 = 别名.get(id) ?? new Set<string>();
		字列.add(记录.character.获取名称());
		别名.set(id, 字列);
	}
}
const 重码表 = new Map<string, string[]>();
for (const [字, 码集] of 权威码) {
	for (const 码 of 码集) {
		const 同组 = 重码表.get(码) ?? [];
		同组.push(字);
		重码表.set(码, 同组);
	}
}
for (const [码, 同组] of 重码表) if (同组.length < 2) 重码表.delete(码);
const 指标 = JSON.parse(
	readFileSync(resolve(编码目录, "分析.json"), "utf8"),
) as {
	重码: number;
	小集合组数: number;
	补集合例外组数: number;
	分数: number;
};
if (
	[...重码表.values()].reduce((和, 同组) => 和 + 同组.length - 1, 0) !==
	指标.重码
)
	throw Error("报告的重码统计与 Rust 不一致");
const 报告字形表: 报告字形[] = [];
const 重码元素 = new Map<
	string,
	{ 成员: Map<string, string>; 位置: Set<string>[] }
>();
const 部件排序键: { id: string; 笔画名称: 笔画名称[] }[] = [];
const 形名 = (id: number) => id.toString(16).padStart(5, "0");
for (const id of [...目标字形].sort(
	(甲, 乙) => Number(minimal.has(乙)) - Number(minimal.has(甲)) || 甲 - 乙,
)) {
	const 结果 = 分析字形(id);
	const 主字 = minimal.get(id) ?? [];
	const 全别名 = [...new Set([...主字, ...(别名.get(id) ?? [])])];
	const 报告行: 报告字形["行"] = [];
	for (const 字 of 全别名) {
		const 读音 = 字音.get(字);
		if (!读音) {
			if (主字.includes(字)) throw Error(`minimal 字缺少读音：${字}`);
			continue;
		}
		const rows = 读音.map((pinyin) => {
			const 声 = 应用拼写运算(声规则, pinyin);
			return {
				pinyin,
				elements: [
					{
						label: "声母",
						name: 声 === "0" ? "零声母" : 声,
						key: 取键(`声-${声}`),
						colorIndex: 0,
						colorRole: "initial" as const,
					},
					...结果.形码,
				],
			};
		});
		for (const 行 of rows) {
			const 编码 = 行.elements.map((项) => 项.key).join("");
			const 同组 = 主字.includes(字) ? (重码表.get(编码) ?? []) : [];
			if (同组.length) {
				const 组 = 重码元素.get(编码) ?? {
					成员: new Map<string, string>(),
					位置: 行.elements.map(() => new Set<string>()),
				};
				组.成员.set(字, 形名(id));
				for (const [位置, 项] of 行.elements.entries()) {
					const 名 = 位置 === 0
						? `声-${项.name === "零声母" ? "0" : 项.name}`
						: 项.name.startsWith("字形-") ? 项.name : String(笔画名.indexOf(项.name) + 1);
					组.位置[位置].add(取归并终点(名).名称);
				}
				重码元素.set(编码, 组);
			}
			报告行.push({
				字,
				拼音: 行.pinyin,
				编码,
				元素: 行.elements.map(({ label, name, key, colorRole }) => ({
					label,
					name,
					key,
					colorRole,
				})),
				重码同组: 同组,
			});
		}
		if (主字.includes(字)) {
			if (字 in characters) throw Error(`同一字有多个不同字形：${字}`);
			const 实际 = new Set(
				rows.map((行) => 行.elements.map((项) => 项.key).join("")),
			);
			const 预期 = 确保存在(权威码.get(字), "码表缺字");
			if (实际.size !== 预期.size || [...实际].some((码) => !预期.has(码)))
				throw Error(`展示与 Rust 码表不一致：${字}`);
			characters[字] = { paths: 结果.paths, roots: 结果.roots, rows };
		}
	}
	if (!报告行.length)
		报告行.push({
			字: "",
			拼音: "",
			编码: 结果.基础形码.map((项) => 项.key).join(""),
			元素: 结果.基础形码,
			重码同组: [],
		});
	const 报告ID = 形名(id);
	const { 形 } = 结果;
	const 是部件字形 = 是部件(形);
	if (是部件字形)
		部件排序键.push({
			id: 报告ID,
			笔画名称: 结果.显示笔画.map((笔) => 笔.feature),
		});
	报告字形表.push({
		id: 报告ID,
		名称: 原形索引.get(id)?.name ?? "",
		别名: 全别名,
		minimal: 主字,
		类别: 是部件字形 ? "部件" : "复合体",
		笔顺: 是部件字形
			? 结果.显示笔画.map((笔) => 分类器[笔.feature]).join("")
			: undefined,
		paths: 结果.paths,
		roots: 结果.roots,
		作为部件的形码: 结果.基础形码.map((项) => 项.key).join(""),
		完整拆分: 结果.完整拆分,
		分部: 是部件字形
			? []
			: 形.部分列表.map((子, i) => ({
					id: 形名(子.id),
					正部: i === 结果.正部位,
				})),
		行: 报告行,
	});
}
if (Object.keys(characters).length !== 权威码.size)
	throw Error("小鱼数据未完整覆盖 minimal");
for (const 项 of 报告字形表)
	for (const 子 of 项.分部)
		if (!目标字形.has(Number.parseInt(子.id, 16)))
			throw Error(`递归分部未收录：${子.id}`);
const 当前快照路径 = resolve(输出目录, "solution.json");
// 只补报告图标。ID 依次对应横、竖、撇、点、乙。
const 单笔图 = [0x00000, 0x00002, 0x00004, 0x00006, 0x00015].map((id, i) => {
	const 形 = 确保存在(显示字库.获取字形(id), `缺少代表笔画 ${形名(id)}`);
	const 笔画 = 形.图形盒子.获取笔画列表();
	if (笔画.length !== 1) throw Error(`代表笔画不是单笔：${形名(id)}`);
	return [String(i + 1), 生成笔画路径(笔画)] as const;
});
const 部件顺序 = 部件排序键.sort(比较笔画顺序).map(({ id }) => id);
const 报告数据: 报告 = {
	版本: 1 as const,
	来源: 当前快照路径,
	指标,
	统计: {
		minimal字数: 权威码.size,
		字声母数: [...权威码.values()].reduce((和, 集) => 和 + 集.size, 0),
		字形数: 报告字形表.length,
		部件数: 输入.部件.length,
		重码组数: 重码表.size,
	},
	字根图: Object.fromEntries([...根图, ...单笔图]),
	部件顺序,
	字形: 报告字形表,
	重码组: [...重码表]
		.sort(([甲码, 甲组], [乙码, 乙组]) =>
			乙组.length - 甲组.length || (甲码 < 乙码 ? -1 : Number(甲码 > 乙码)),
		)
		.map(([编码, 同组]) => {
			const { 成员, 位置 } = 确保存在(重码元素.get(编码), `报告缺少重码组：${编码}`);
			return {
				编码,
				差异位置: 位置.flatMap((元素, i) => 元素.size > 1 ? [i + 1] : []),
				成员: 同组.map((字) => ({ id: 确保存在(成员.get(字), `重码组缺字：${字} ${编码}`), 字 })),
			};
		}),
};
const 小鱼数据 = {
	version: 2,
	name: "冰雪飞花 beta",
	source: 当前快照路径,
	metrics: 指标,
	characters,
};
const 字根产物 = 生成字根图();
const 拆分页图 = resolve(编码目录, "component-splits");
writeFileSync(`${拆分页图}.svg`, 生成基本拆分分页图(报告数据, 部件排序键));
const 产物: Record<string, string | Buffer> = {
	"input.json": 输入字节,
	"solution.json": 快照字节,
	"feihua-beta.json": JSON.stringify(小鱼数据),
	"report.json": JSON.stringify(报告数据),
	"report.html": 生成报告(报告数据),
	"root-chart.svg": 字根产物.svg,
	"root-chart.png": 字根产物.png,
	"root-chart.pdf": 字根产物.pdf,
	"component-splits.pdf": 转换SVG(拆分页图, "pdf"),
};
for (const 名 of ["config.yaml", "code.txt", "splits.json", "分析.json"])
	产物[名] = readFileSync(resolve(编码目录, 名));
// 所有数据、码表、报告和字根图验证成功后，才替换当前产物。
mkdirSync(输出目录, { recursive: true });
const 写入锁 = resolve(输出目录, ".postprocess-lock");
try {
	mkdirSync(写入锁);
} catch (错误) {
	if (错误 instanceof Error && "code" in 错误 && 错误.code === "EEXIST")
		throw Error("当前产物正在写入；请等待另一个后处理结束后重试");
	throw 错误;
}
try {
	for (const [名, 数据] of Object.entries(产物))
		writeFileSync(resolve(输出目录, `${名}.${process.pid}.tmp`), 数据);
	for (const 名 of Object.keys(产物))
		renameSync(
			resolve(输出目录, `${名}.${process.pid}.tmp`),
			resolve(输出目录, 名),
		);
} finally {
	for (const 名 of Object.keys(产物))
		rmSync(resolve(输出目录, `${名}.${process.pid}.tmp`), { force: true });
	rmSync(写入锁, { recursive: true, force: true });
}
console.log(
	`后处理完成：${权威码.size} 个 minimal 字、${报告字形表.length} 个字形，全部 minimal 编码与 Rust 一致。\n${输出目录}/feihua-beta.json\n${输出目录}/report.html\n${输出目录}/root-chart.png\n${输出目录}/component-splits.pdf`,
);

function 转换SVG(前缀: string, 格式: "png" | "pdf"): Buffer {
	const 转换 = spawnSync(
		参数.get("--inkscape") ?? "inkscape",
		[
			`${前缀}.svg`,
			"--export-area-page",
			`--export-type=${格式}`,
			格式 === "pdf" ? "--export-text-to-path" : "--export-width=2520",
			`--export-filename=${前缀}.${格式}`,
		],
		{ encoding: "utf8" },
	);
	if (转换.error || 转换.status !== 0)
		throw Error(
			`SVG 已写入 ${前缀}.svg；${格式.toUpperCase()} 转换失败（退出码 ${转换.status}，信号 ${转换.signal}）。\n${转换.error?.message ?? 转换.stderr}`,
		);
	return readFileSync(`${前缀}.${格式}`);
}

function 生成字根图() {
	const 方案 = 配置;
	const 前缀 = resolve(编码目录, "root-chart");
	const 映射 = 方案.form.mapping;

	const 笔画 = ["⼀", "⼁", "⼃", "⼂", "乙"];
	const 配色 = {
		B: { 字: "#7950a4", 底: "#faf7fd", 边: "#d7c7e7" },
		A: { 字: "#3065a0", 底: "#f7faff", 边: "#c5d6ea" },
		绿: { 字: "#29816a", 底: "#f3fbf7", 边: "#bfdfd1" },
		灰: { 字: "#626b76", 底: "#f4f5f6", 边: "#d0d4d9" },
		无理: "#c83e49",
	};
	const 逃逸 = (文本: string) =>
		文本.replace(
			/[&<>"']/g,
			(字) =>
				({
					"&": "&amp;",
					"<": "&lt;",
					">": "&gt;",
					'"': "&quot;",
					"'": "&apos;",
				})[字] ?? 字,
		);

	// 士、囗、雨、竹即使没有直接取码，也在图中显示。
	const 白名单 = new Set(["字形-0020f", "字形-00259", "字形-0070c", "字形-04414"]);
	const 展示字根 = new Set(白名单);
	for (const 字 of Object.values(characters)) {
		for (const 根 of 字.roots) 展示字根.add(根.name);
	}

	const 同笔符号 = new Map((输入.同笔主根 ?? []).map((名, i) => [名, ["２", "３", "４"][i]]));
	for (const 名 of 展示字根) {
		if (同笔符号.has(取归并终点(名).名称)) 展示字根.delete(名);
	}
	const 笔画顺位 = new Map(Object.keys(合并分类器()).map((笔, i) => [笔, i]));
	const 小集合根 = [...展示字根]
		.filter((名) => 名.startsWith("字形-") && 键序[0].toLowerCase().includes(取键(名)))
		.map((id) => {
			const 件 = 取形(Number.parseInt(id.slice(3), 16));
			if (!是部件(件)) throw Error(`小集合字根应为部件：${id}`);
			const 笔画名称 = 件.笔画列表.map((笔) => 笔.feature);
			return { id, 笔画名称, 特征: 重排特征(件), 签名: 全形签名(件) };
		})
		.sort((甲, 乙) => 比较笔画顺序(甲, 乙, 笔画顺位));
	const 同类根 = new Map<string, typeof 小集合根>();
	for (const 根 of 小集合根) {
		const 已见 = 同类根.get(根.特征.分组) ?? [];
		const 在前 = 已见.find((前) =>
			前.笔画名称.some((笔, i) => 笔 !== 根.笔画名称[i]) &&
			查重排(根.特征, 前.特征, 前.签名),
		);
		if (在前 && !白名单.has(根.id)) {
			展示字根.delete(根.id);
			console.log(`字根图隐藏 ${根.id}，重排同形的 ${在前.id} 笔顺在前`);
		} else {
			已见.push(根);
			同类根.set(根.特征.分组, 已见);
		}
	}

	const 键笔画 = new Map<string, string[]>();
	for (const [位, 符号] of 笔画.entries()) {
		if (!展示字根.has(String(位 + 1))) continue;
		const 键 = 取键(String(位 + 1));
		const 符号组 = 键笔画.get(键) ?? [];
		符号组.push(符号);
		键笔画.set(键, 符号组);
	}

	// 与当前预算同口径：只有主根实际归并到声母锚点才免占例外额度。
	function 是声母归并(名称: string): boolean {
		const 安排 = 映射[名称];
		return (
			!!安排 &&
			typeof 安排 === "object" &&
			!Array.isArray(安排) &&
			安排.element.startsWith("声-")
		);
	}
	type 根组 = { 终点: string; 键: string; 字根: string[]; 无理: boolean };
	const 分组 = new Map<string, 根组>();
	for (const [名称, 安排] of Object.entries(映射)) {
		if (安排 == null || !是根名(名称)) continue;
		const 终点 = 取归并终点(名称);
		const 组 = 分组.get(终点.名称) ?? {
			终点: 终点.名称,
			键: 终点.键,
			字根: [],
			无理: !键序[0].toLowerCase().includes(终点.键) && !是声母归并(终点.名称),
		};
		组.字根.push(名称);
		分组.set(终点.名称, 组);
	}
	if (!分组.size) throw Error("配置中没有启用的字形字根");
	// Rust 的预算按所有启用组计数，先核对，再筛选图中实际使用的成员。
	const 启用组 = [...分组.values()];
	if (
		启用组.filter((组) => 键序[0].toLowerCase().includes(组.键)).length !== 指标.小集合组数 ||
		启用组.filter((组) => 组.无理).length !== 指标.补集合例外组数
	)
		throw Error("启用字根的组数与优化指标不一致");
	const 同笔图例 = new Map<string, string[]>();
	for (const 组 of 启用组) {
		const 符号 = 同笔符号.get(组.终点);
		if (符号) {
			同笔图例.set(符号, [组.终点, ...组.字根.filter((名) => 名 !== 组.终点)]);
			组.字根 = [符号];
			键笔画.set(组.键, [...(键笔画.get(组.键) ?? []), 符号]);
			continue;
		}
		组.字根 = 组.字根.filter((名) => 展示字根.has(名));
		if (!组.字根.length) {
			分组.delete(组.终点);
			continue;
		}
		// 主根优先，其余附形按配置声明顺序紧随。
		组.字根.sort((甲, 乙) => Number(乙 === 组.终点) - Number(甲 === 组.终点));
	}

	const 宽 = 1680,
		页高 = 1188,
		边距 = 40,
		间距 = 10;
	const 格宽 = (宽 - 边距 * 2 - 间距 * 7) / 8;
	const 字号 = 26,
		格距 = 32,
		正文左距 = 12 + (格距 - 字号) / 2,
		正文顶距 = 49 + (格距 - 字号) / 2,
		内容宽 = 格宽 - 24;
	const 每行根数 = Math.floor(内容宽 / 格距);
	type 排版组 = 根组 & { 起格: number };
	function 排版(键: string) {
		const 组列表: 排版组[] = [];
		let 起格 = 0,
			上组无理 = false;
		const 本键组 = [...分组.values()]
			.filter((组) => 组.键 === 键)
			// 五笔画根已显示在键格右上角，参与统计但不重复占用字根网格。
			.map((组) => ({
				...组,
				字根: 组.字根.filter((名) => 名.startsWith("字形-")),
			}))
			.filter((组) => 组.字根.length > 0)
			.sort((甲, 乙) => Number(甲.无理) - Number(乙.无理));
		for (const 组 of 本键组) {
			const 剩余 = 每行根数 - (起格 % 每行根数);
			// 字根统一落在网格上；短归并组不跨行，无理区从新行开始。
			if (剩余 < 每行根数 && (组.字根.length > 剩余 || (组.无理 && !上组无理)))
				起格 += 剩余;
			组列表.push({ ...组, 起格 });
			起格 += 组.字根.length;
			上组无理 = 组.无理;
		}
		return { 组列表, 高: Math.ceil(起格 / 每行根数) * 格距 };
	}
	const 键布局 = new Map([...全键].map((键) => [键, 排版(键)]));
	function 取布局(键: string) {
		const 布局 = 键布局.get(键);
		if (!布局) throw Error(`缺少键布局：${键}`);
		return 布局;
	}
	const 行高 = 键序.map((行) =>
		Math.max(
			156,
			...[...行.toLowerCase()].map((键) => 取布局(键).高 + 66),
		),
	);
	const 图高 = 160 + 行高.reduce((和, 高) => 和 + 高, 0) + 间距 * 3 + 40;
	// 过密的方案整体等比缩小；始终保持 A4 横版，字根组不拆到其他键格。
	const 缩放 = Math.min(1, (页高 - 32) / 图高);
	const 平移横 = (宽 - 宽 * 缩放) / 2;
	const 平移纵 = (页高 - 图高 * 缩放) / 2;
	const 片段: string[] = [];
	function 文本(
		横: number,
		纵: number,
		内容: string,
		大小: number,
		色 = "#23384a",
		附加 = "",
	) {
		片段.push(
			`<text x="${横}" y="${纵}" font-size="${大小}" fill="${色}" ${附加}>${逃逸(内容)}</text>`,
		);
	}
	function 矩形(
		横: number,
		纵: number,
		格宽: number,
		高: number,
		填色: string,
		边色: string,
		圆角 = 10,
	) {
		片段.push(
			`<rect x="${横}" y="${纵}" width="${格宽}" height="${高}" rx="${圆角}" fill="${填色}" stroke="${边色}"/>`,
		);
	}
	function 字形文字(横: number, 顶: number, 内容: string, 大小: number, 色: string, 字距 = 大小) {
		片段.push(`<g aria-label="${逃逸(内容)}" fill="none" stroke="${色}" stroke-width="5.4" stroke-linecap="round" stroke-linejoin="round">`);
		for (const [序, 字] of [...内容].entries()) {
			const 单笔位 = 笔画.indexOf(字);
			let 路径;
			if (单笔位 >= 0) {
				路径 = 单笔图[单笔位][1];
			} else {
				const 如字符 = 字符.从码位创建(字.codePointAt(0)!);
				if (!如字符.ok) throw 如字符.error;
				const 形 = 确保存在(显示字库.查询字符的字形(如字符.value)?.[0], `字库缺少图中文字：${字}`);
				路径 = 生成笔画路径(形.图形盒子.获取笔画列表());
			}
			片段.push(`<svg x="${横 + 序 * 字距}" y="${顶}" width="${大小}" height="${大小}" viewBox="0 0 100 100">${路径.map((径) => `<path d="${逃逸(径)}"/>`).join("")}</svg>`);
		}
		片段.push("</g>");
	}
	片段.push('<g data-heading="冰雪飞花">');
	字形文字(边距, 30, "冰雪飞花", 48, "#17364a");
	片段.push("</g>");
	矩形(边距 + 222, 45, 66, 29, "#e5f4f0", "#b8dbd1", 14);
	文本(
		边距 + 255,
		65,
		"beta",
		17,
		"#287762",
		'text-anchor="middle" font-weight="600"',
	);
	文本(
		边距,
		114,
		"B 集合 · 紫色　 /　 A 集合 · 蓝色　 /　 非声母归并 · 红色　 /　 归并字根相邻排列",
		17,
		"#617380",
	);
	for (const [声, 标记] of [["z", "zh"], ["c", "ch"], ["s", "sh"], ["0", "零声母"]]) {
		const 键 = 取键(`声-${声}`);
		键笔画.set(键, [...(键笔画.get(键) ?? []), 标记]);
	}
	let 纵 = 146;
	for (const [行号, 行] of 键序.entries()) {
		const 高 = 行高[行号];
		for (const [列号, 大写键] of [...行].entries()) {
			const 横 = 边距 + 列号 * (格宽 + 间距),
				键 = 大写键.toLowerCase();
			const 色 = 行号 === 0 ? 配色.B : ",.".includes(键) ? 配色.绿 : 配色.A;
			片段.push(`<g data-key="${逃逸(大写键)}">`);
			矩形(横, 纵, 格宽, 高, 色.底, 色.边);
			文本(
				横 + 15,
				纵 + 33,
				大写键,
				27,
				色.字,
				'font-family="Arial, sans-serif" font-weight="700"',
			);
			const 笔画标记 = 键笔画.get(键);
			if (笔画标记) {
				const 大小 = 笔画标记.length > 1 ? 18 : 27;
				let 右端 = 横 + 格宽 - 15;
				for (const 标记 of [...笔画标记].reverse()) {
					if (笔画.includes(标记) || 标记 === "零声母") {
						字形文字(右端 - 标记.length * 大小, 纵 + 34 - 大小, 标记, 大小, 色.字);
					} else {
						文本(右端, 纵 + 32, 标记, 大小, 色.字, 'text-anchor="end"');
					}
					右端 -= 标记.length * 大小 + 8;
				}
			}
			for (const 组 of 取布局(键).组列表) {
				片段.push(
					`<g data-group="${逃逸(组.终点)}" data-irrational="${组.无理}"><title>${逃逸(组.字根.join("、"))}</title>`,
				);
				for (const [序, 名称] of 组.字根.entries()) {
					const 路径 = 取显示根图(名称);
					const 格号 = 组.起格 + 序;
					const 字横 = 横 + 正文左距 + (格号 % 每行根数) * 格距;
					const 字纵 =
						纵 + 正文顶距 + Math.floor(格号 / 每行根数) * 格距;
					片段.push(
						`<svg data-root="${名称}" x="${字横}" y="${字纵}" width="${字号}" height="${字号}" viewBox="0 0 100 100"><g fill="none" stroke="${组.无理 ? 配色.无理 : 色.字}" stroke-width="5.4" stroke-linecap="round" stroke-linejoin="round">${路径
							.map((径) => `<path d="${逃逸(径)}"/>`)
							.join("")}</g></svg>`,
					);
				}
				片段.push("</g>");
			}
			片段.push("</g>");
		}
		if (行号 === 3) {
			const 规则 = [
				{ 标题: "部件字", 行: ["声母", "字根"], 次数: 3 },
				{ 标题: "复合体字", 行: ["声母", "正根", "余根"], 次数: 2 },
			];
			for (const [序, { 标题, 行, 次数 }] of 规则.entries()) {
				const 横 = 边距 + (6 + 序) * (格宽 + 间距);
				片段.push(`<g data-rule="${标题}">`);
				矩形(横, 纵, 格宽, 高, 配色.灰.底, 配色.灰.边);
				// 键名字面中心在基线以上约 9.75；标题和正文共用同一字格。
				const 标题顶 = 纵 + 33 - 9.75 - 字号 / 2;
				字形文字(横 + 正文左距, 标题顶, 标题, 字号, 配色.灰.字, 格距);
				for (const [位, 词] of 行.entries()) {
					const 顶 = 纵 + 正文顶距 + 位 * 格距;
					字形文字(横 + 正文左距, 顶, 词, 字号, "#23384a", 格距);
					if (位 === 行.length - 1)
						文本(横 + 正文左距 + 词.length * 格距, 顶 + 字号 * 0.85, `× ${次数}`, 字号);
				}
				片段.push("</g>");
			}
		}
		纵 += 高 + 间距;
	}
	function 统计字根(基本: boolean) {
		const 组 = [...分组.values()].filter(
			(组) => 键序[0].toLowerCase().includes(组.键) === 基本,
		);
		return `${组.length} 组 ${组.reduce((和, 项) => 和 + 项.字根.length, 0)} 个`;
	}
	const 例外组数 = [...分组.values()].filter((组) => 组.无理).length;
	const 统计文本 = `基本字根 ${统计字根(true)}・扩展字根 ${统计字根(false)}（非声母归并 ${例外组数} 组）`;
	文本(边距, 纵 + 20, 统计文本, 15, "#617380");
	// 与左侧统计共用页脚，按 ２、３、４ 的顺序从右边界反推各组位置。
	let 图例右端 = 宽 - 边距;
	for (const 符号 of [...同笔符号.values()].reverse()) {
		const 成员 = 确保存在(同笔图例.get(符号), `缺少同笔图例：${符号}`);
		const 起点 = 图例右端 - 28 - 成员.length * 26;
		片段.push(`<g data-legend="${符号}" fill="none" stroke="${配色.B.字}" stroke-width="5.4" stroke-linecap="round" stroke-linejoin="round">`);
		文本(起点, 纵 + 21, 符号, 22, 配色.B.字, 'stroke="none"');
		for (const [i, 名] of 成员.entries()) {
			const 路径 = 取显示根图(名).map((径) => `<path d="${逃逸(径)}"/>`).join("");
			片段.push(`<svg data-legend-root="${名}" x="${起点 + 28 + i * 26}" y="${纵 + 1}" width="24" height="24" viewBox="0 0 100 100">${路径}</svg>`);
		}
		片段.push("</g>");
		图例右端 = 起点 - 24;
	}

	const SVG = `<svg xmlns="http://www.w3.org/2000/svg" width="297mm" height="210mm" viewBox="0 0 ${宽} ${页高}" role="img" aria-labelledby="title desc">
<title id="title">冰雪飞花 beta 字根图</title><desc id="desc">键序 AOEIUV;/、BPMFDTNL、GKHWJQXY、ZCSR,.。归并字根同组排列，右下两格为取码规则。</desc>
<rect width="${宽}" height="${页高}" fill="#ffffff"/>
<g font-family="PingFang SC, Noto Sans CJK SC, Source Han Sans SC, sans-serif" transform="translate(${平移横} ${平移纵}) scale(${缩放})">${片段.join("\n")}</g></svg>\n`;
	writeFileSync(`${前缀}.svg`, SVG);
	return {
		svg: SVG,
		png: 转换SVG(前缀, "png"),
		pdf: 转换SVG(前缀, "pdf"),
	};
}
