import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";
import {
	合并分类器,
	合并拼写运算,
	type 字形,
	type 字形拼写运算,
	应用拼写运算,
	是部件,
	type 模式,
	生成笔画路径,
	获取原始词典,
	获取字库,
	获取融合字形集合,
	读取配置,
	部件,
} from "hanzi-chai";
import { 确保存在, 笔形签名, type 飞花输入 } from "./prepare.ts";

const 用法 =
	"bun src/feihua/export-bot.ts --solution <solution.json> [--tree-data output/feihua/input.json] [--out output/feihua-beta.json]";
const 参数 = new Map<string, string>();
for (let i = 2; i < process.argv.length; i += 2) {
	const 名 = process.argv[i];
	const 值 = process.argv[i + 1];
	if (!["--solution", "--tree-data", "--out"].includes(名) || 值 === undefined)
		throw Error(用法);
	参数.set(名, 值);
}
const 输入路径 = resolve(参数.get("--tree-data") ?? "output/feihua/input.json");
const 快照路径 = resolve(确保存在(参数.get("--solution"), 用法));
const 输出路径 = resolve(参数.get("--out") ?? "output/feihua-beta.json");
const 相对 = relative(resolve("output"), 输出路径);
if (
	!相对 ||
	相对.startsWith("..") ||
	!输出路径.endsWith(".json") ||
	[输入路径, 快照路径].includes(输出路径)
)
	throw Error("--out 必须是 output/ 中独立的 JSON 文件");
const 编码目录 = `${输出路径.slice(0, -5)}-encoding`;
if (
	[输入路径, 快照路径].some(
		(路径) => !relative(编码目录, 路径).startsWith(".."),
	)
)
	throw Error("导出目录不能包含输入文件");
// 由 Rust 检查输入校验、索引、最少根约束，再生成本次的权威码表与键位配置。
const 编码 = spawnSync(
	resolve("target/release/feihua"),
	[
		"--tree-data",
		输入路径,
		"encode",
		"--solution",
		快照路径,
		"--out",
		编码目录,
	],
	{ stdio: "inherit" },
);
if (编码.error) throw 编码.error;
if (编码.status !== 0) throw Error("Rust 快照校验或编码失败");
const 输入 = JSON.parse(readFileSync(输入路径, "utf8")) as 飞花输入;
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
const 原快照: unknown = JSON.parse(readFileSync(快照路径, "utf8"));
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
function 是准包围({ from, to }: 字形拼写运算): boolean {
	if (
		typeof from === "number" ||
		!("references" in from) ||
		typeof to === "number" ||
		!("references" in to)
	)
		return false;
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
const 取形 = (id: number) =>
	融合(确保存在(字库.获取字形(id), `缺少字形 ${id.toString(16)}`));
const 键缓存 = new Map<string, string>();
function 取键(名: string, 链 = new Set<string>()): string {
	const 旧 = 键缓存.get(名);
	if (旧 !== undefined) return 旧;
	if (链.has(名)) throw Error(`归并成环：${名}`);
	const 值 = 确保存在(配置.form?.mapping[名], `未启用字根 ${名}`);
	let 键: string;
	if (typeof 值 === "string" && 值.length === 1) 键 = 值;
	else if (!Array.isArray(值) && typeof 值 === "object")
		键 = 取键(值.element, new Set([...链, 名]));
	else throw Error(`不支持的字根安排：${名}`);
	键缓存.set(名, 键);
	return 键;
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
	const 列 = 字音.get(条.词) ?? [];
	if (!列.includes(条.拼音[0])) 列.push(条.拼音[0]);
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
function 根元素(根: 字根出现, colorIndex: number, colorRole: 配色角色) {
	let paths: string[] | undefined;
	if (根.名称.startsWith("字形-")) {
		paths = 根图.get(根.名称);
		if (paths === undefined) {
			paths = 生成笔画路径(
				确保存在(
					显示字库.获取字形(Number.parseInt(根.名称.slice(3), 16)),
					"缺少显示字根",
				).图形盒子.获取笔画列表(),
			);
			根图.set(根.名称, paths);
		}
	}
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
const characters: Record<string, 演示> = {};
for (const 字 of 输入.字) {
	if (字.字 in characters) throw Error("暂不支持同一字的多个地区字形");
	const 形 = 取形(字.字形);
	const 笔画 = 形.图形盒子.获取笔画列表();
	const 树 = 定位(
		形,
		笔画.map((_, i) => i),
	);
	const 原分析形 = 确保存在(字库.获取字形(字.字形), "缺少分析字形");
	const 显示形 = 确保存在(显示字库.获取字形(字.字形), "缺少显示字形");
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
	if (是部件(形)) {
		根 = 部件取根(树).slice(0, 3);
		if (根.length === 1)
			次数 = 重复[确保存在(部件表.get(形.id), "缺少部件").序号];
	} else {
		const 分部 = 确保存在(分部表.get(形.id), "缺少冻结分部");
		const 正部位 = 树.子项.findIndex(
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
	const 形码 = 根.flatMap((项, i) =>
		Array.from({ length: 次数 }, () => 根元素(项, i + 2, 根角色[i])),
	);
	const rows = 确保存在(字音.get(字.字), "缺少读音").map((pinyin) => {
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
				...形码,
			],
		};
	});
	const 实际 = new Set(
		rows.map((行) => 行.elements.map((项) => 项.key).join("")),
	);
	const 预期 = 确保存在(权威码.get(字.字), "码表缺字");
	if (实际.size !== 预期.size || [...实际].some((码) => !预期.has(码)))
		throw Error(`展示与 Rust 码表不一致：${字.字}`);
	characters[字.字] = {
		paths: 生成笔画路径(显示笔画),
		roots: 根.map((项, i) => ({
			strokes: 项.笔画.map((笔) => 对应[笔]),
			name: 项.名称,
			colorRole: 根角色[i],
		})),
		rows,
	};
}
mkdirSync(dirname(输出路径), { recursive: true });
writeFileSync(
	`${输出路径}.tmp`,
	JSON.stringify({
		version: 2,
		name: "冰雪飞花 beta",
		source: 快照路径,
		metrics: JSON.parse(readFileSync(resolve(编码目录, "分析.json"), "utf8")),
		characters,
	}),
);
renameSync(`${输出路径}.tmp`, 输出路径);
console.log(
	`导出 ${Object.keys(characters).length} 字，全部编码与 Rust 一致：${输出路径}`,
);
