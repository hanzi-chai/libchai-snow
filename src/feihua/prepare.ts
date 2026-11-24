import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import {
	创建字形ID分配器,
	合并分类器,
	合并拼写运算,
	type 基本部件数据,
	type 字形,
	字集过滤查找表,
	type 广义安排,
	应用拼写运算,
	拓扑,
	是部件,
	格式化字形ID,
	生成笔画路径,
	type 矢量图形数据,
	获取原始词典,
	获取字库,
	获取融合字形集合,
	部件,
	type 配置,
	默认退化配置,
} from "hanzi-chai";
import { dump } from "js-yaml";
import 方案, { 保留小集合, 生成声母归并 } from "./config.ts";
import { 比较笔画顺序 } from "./report.ts";

const 小集合 = "aoeiuv;/";
const 补集合 = "bpmfdtnlgkhjqxzcsryw,.";
const 全集合 = 小集合 + 补集合;
const 输出目录 = resolve("output/feihua");
const 意符优先字 =
	"丕刈划刖列刘刮劝功叩召吝否告和垄墨姜孜帛志恙悲戏戕攻敉敌斐斛斧斫柔欢欣欲歆汞畋盘瞽砻硕紊翊翔翡翮聋致舅蜚蟊袭裴观贡败赦辈邓邛邝邯邺郅郝釜难雀雉靓项顺顼颀颇飒魄鸡鸹鹆鹘默";
const 是根名 = (名: string) => /^[1-5]$|^字形-[0-9a-f]{5}$/.test(名);
const 根名 = (id: number) => `字形-${格式化字形ID(id)}`;
const 相同 = (甲: unknown, 乙: unknown) =>
	JSON.stringify(甲) === JSON.stringify(乙);

export function 确保存在<T>(值: T | undefined | null, 说明: string): T {
	if (值 === undefined || 值 === null) throw Error(说明);
	return 值;
}

type 安排 = string | { element: string };
const 键域 = { 小集合, 补集合, 全集合 };
type 根 = {
	名称: string;
	落键: keyof typeof 键域;
	核心: boolean;
	初始: 安排 | null;
	空间: 安排[];
};
type 单元 = { type: string; strokes: number[] };
type 请求 = { 字根: string } | { 部件: number };
type 分部 = {
	id: number;
	正部: number;
	正根: 请求;
	余部: number[];
	余根: { 部件: number; 数量: number }[];
};
type 候选 = { 笔画: number[]; 字根: string[] };
type 部件输入 = { id: number; 单元: 单元[]; 候选: 候选[]; 拆分: number[][] };
type 字输入 = { 字: string; 字形: number; 声母: string[] };
export type 飞花输入 = {
	版本: 3;
	局部约束: number[][][];
	初始拆分: number[];
	配置: 配置;
	字根: 根[];
	部件: 部件输入[];
	复合体: 分部[];
	字: 字输入[];
	字根字: { 字形: number; 字根: string[]; 重复: number }[];
};

function 展开安排(值: 广义安排, 方案: 配置): 安排[] {
	if (typeof 值 === "string") {
		if (值.length !== 1 || !全集合.includes(值)) throw Error(`无效单键：${值}`);
		return [值];
	}
	if (值 === null) throw Error("手写配置不能包含未启用或允许删除的字根");
	if (!Array.isArray(值)) return [{ element: 值.element }];
	if (值.length !== 1) throw Error("字根安排必须恰好一码");
	const 位 = 值[0];
	if (位 !== null && typeof 位 === "object" && "variable" in 位) {
		const 变量 = 确保存在(
			方案.form?.mapping_variables?.[位.variable],
			`未知变量：${位.variable}`,
		);
		return 变量.keys.flatMap((键) => 展开安排(键, 方案));
	}
	if (typeof 位 === "string") return 展开安排(位, 方案);
	throw Error("只支持单键、单键变量及 element 归并");
}

function 检查空间(方案: 配置, 取字形: (id: number) => 字形) {
	const 键盘 = 确保存在(方案.form, "缺少 form");
	if (
		!相同(
			[...确保存在(键盘.alphabet, "缺少 alphabet")].sort(),
			[...全集合].sort(),
		)
	)
		throw Error("alphabet 必须恰好包含方案的 30 个键");
	for (const [名, 键集] of Object.entries({ 小集合, 补集合, 全集合 })) {
		const 变量 = 确保存在(键盘.mapping_variables?.[名], `缺少变量：${名}`);
		if (!相同([...变量.keys].sort(), [...键集].sort()))
			throw Error(`变量 ${名} 的键集不正确`);
	}
	const 字根 = new Map<string, 根>();
	for (const 名 of Object.keys(键盘.mapping_space ?? {})) {
		if (是根名(名) && !(名 in 键盘.mapping))
			throw Error(`手写配置包含备选字根：${名}`);
	}
	for (const [名, 当前] of Object.entries(键盘.mapping)) {
		if (!是根名(名)) continue;
		const 描述 = 键盘.mapping_space?.[名] ?? [];
		if (描述.some((项) => 项.condition?.length))
			throw Error(`不支持条件空间：${名}`);
		const 空间 = [当前, ...描述.map((项) => 项.value)].flatMap((值) =>
			展开安排(值, 方案),
		);
		const 唯一空间 = [
			...new Map(空间.map((值) => [JSON.stringify(值), 值])).values(),
		];
		const 初始列表 = 展开安排(当前, 方案);
		if (初始列表.length !== 1) throw Error(`当前安排不是单一决策：${名}`);
		const 笔画 = /^[1-5]$/.test(名);
		const 复合体 = !笔画 && !是部件(取字形(Number.parseInt(名.slice(3), 16)));
		if (复合体 && 保留小集合.has(名))
			throw Error(`复合体不能保留在小集合：${名}`);
		const 落键 = 复合体
			? "补集合"
			: 笔画 || 保留小集合.has(名)
				? "小集合"
				: "全集合";
		字根.set(名, {
			名称: 名,
			落键,
			核心: 笔画,
			初始: 初始列表[0],
			空间: 唯一空间.filter(
				(项) => typeof 项 !== "string" || 键域[落键].includes(项),
			),
		});
	}
	for (let 笔 = 1; 笔 <= 5; 笔++) {
		const 根 = 确保存在(字根.get(String(笔)), `缺少基础笔画 ${笔}`);
		if (根.空间.length !== 1 || typeof 根.空间[0] !== "string")
			throw Error(`笔画必须固定在小集合：${笔}`);
	}
	for (const 根 of 字根.values()) {
		for (const 项 of 根.空间) {
			if (typeof 项 === "string") continue;
			const 名 = 项.element;
			if (名.startsWith("声-")) {
				const 键 = 键盘.mapping[名];
				if (typeof 键 !== "string" || 键.length !== 1 || !补集合.includes(键))
					throw Error(`声母锚点必须固定在补集合：${名}`);
				if (键盘.mapping_space?.[名]?.length)
					throw Error(`声母锚点不能移动：${名}`);
			} else if (!字根.has(名)) throw Error(`归并到未声明字根：${名}`);
		}
	}
	// 只有每一种安排都落在小集合时，才沿归并推断小集合身份。
	let 新增 = true;
	while (新增) {
		新增 = false;
		for (const 根 of 字根.values()) {
			if (
				根.落键 === "全集合" &&
				根.空间.length > 0 &&
				根.空间.every((项) =>
					typeof 项 === "string"
						? 小集合.includes(项)
						: 字根.get(项.element)?.落键 === "小集合",
				)
			) {
				根.落键 = "小集合";
				新增 = true;
			}
		}
	}
	const 初始键 = new Map<string, string>();
	const 解析中 = new Set<string>();
	function 解析初始(根: 根): string {
		const 已有 = 初始键.get(根.名称);
		if (已有 !== undefined) return 已有;
		if (解析中.has(根.名称))
			throw Error(`初始归并成环：${[...解析中, 根.名称].join(" → ")}`);
		const 安排 = 确保存在(根.初始, `缺少初始安排：${根.名称}`);
		if (!根.空间.some((项) => 相同(项, 安排)))
			throw Error(`初始安排超出允许键域：${根.名称}`);
		解析中.add(根.名称);
		const 键 =
			typeof 安排 === "string"
				? 安排
				: 安排.element.startsWith("声-")
					? 键盘.mapping[安排.element]
					: 解析初始(
							确保存在(
								字根.get(安排.element),
								`未声明归并目标：${安排.element}`,
							),
						);
		if (typeof 键 !== "string" || !键域[根.落键].includes(键))
			throw Error(`初始归并超出允许键域：${根.名称} → ${键}`);
		解析中.delete(根.名称);
		初始键.set(根.名称, 键);
		return 键;
	}
	for (const 根 of 字根.values()) 解析初始(根);
	return { 字根, 初始键 };
}

function 选核心根(字根: Map<string, 根>, 部件表: 部件输入[]) {
	const 核心 = new Set(
		[...字根.values()]
			.filter((根) => 根.落键 === "小集合")
			.map((根) => 根.名称),
	);
	const 有拆分 = (件: 部件输入) =>
		件.拆分.some((分法) =>
			分法.every((i) => 件.候选[i].字根.some((名) => 核心.has(名))),
		);
	const 缺失 = 部件表.filter((件) => !有拆分(件));
	if (缺失.length)
		throw Error(
			`核心根覆盖不足（${缺失.length} 个部件），未写出产物：\n${缺失.map((件) => 根名(件.id)).join("\n")}`,
		);
	const 相关部件 = new Map<string, 部件输入[]>();
	for (const 件 of 部件表) {
		const 相关根 = new Set(件.候选.flatMap((项) => 项.字根));
		for (const 名 of 相关根) {
			if (!核心.has(名)) continue;
			const 列表 = 相关部件.get(名) ?? [];
			列表.push(件);
			相关部件.set(名, 列表);
		}
	}
	// 按字形 ID 稳定地剔除可省略的根。其余小集合根仍然保留为准核心。
	for (const 名 of [...核心].sort()) {
		if (字根.get(名)?.核心) continue;
		核心.delete(名);
		if (相关部件.get(名)?.some((件) => !有拆分(件))) 核心.add(名);
	}
	for (const 名 of 核心) 确保存在(字根.get(名), 名).核心 = true;
}

function 读取单元(部件列表: 部件[]): Map<number, 单元[]> {
	const 输入: unknown = JSON.parse(
		readFileSync("src/feihua/units.json", "utf8"),
	);
	if (!Array.isArray(输入)) throw Error("units.json 必须是数组");
	const 记录 = new Map<number, 单元[]>();
	for (const 条 of 输入) {
		if (
			typeof 条 !== "object" ||
			条 === null ||
			!("id" in 条) ||
			typeof 条.id !== "string" ||
			!/^[0-9a-f]{5}$/.test(条.id) ||
			!("units" in 条) ||
			!Array.isArray(条.units)
		)
			throw Error("无效单元记录");
		const id = Number.parseInt(条.id, 16);
		if (记录.has(id)) throw Error(`重复单元记录：${条.id}`);
		const 单元列表: 单元[] = [];
		for (const 组 of 条.units) {
			if (
				typeof 组 !== "object" ||
				组 === null ||
				!("type" in 组) ||
				typeof 组.type !== "string" ||
				!("strokes" in 组) ||
				!Array.isArray(组.strokes)
			)
				throw Error(`无效单元：${条.id}`);
			const 笔画: number[] = [];
			for (const 笔 of 组.strokes) {
				if (typeof 笔 !== "number" || !Number.isInteger(笔))
					throw Error(`无效笔序：${条.id}`);
				笔画.push(笔);
			}
			单元列表.push({ type: 组.type, strokes: 笔画 });
		}
		记录.set(id, 单元列表);
	}
	const 错误: string[] = [];
	for (const 件 of 部件列表) {
		const 分组 = 记录.get(件.id);
		if (!分组) {
			错误.push(`缺少记录：${根名(件.id)}`);
			continue;
		}
		const 已用 = new Set<number>();
		for (const 组 of 分组) {
			if (组.strokes.length < 2) 错误.push(`单元不足两笔：${根名(件.id)}`);
			for (const 笔 of 组.strokes) {
				if (笔 < 1 || 笔 > 件.笔画数() || 已用.has(笔))
					错误.push(`笔序越界或重复：${根名(件.id)} 第 ${笔} 笔`);
				已用.add(笔);
			}
		}
	}
	if (错误.length)
		throw Error(
			`单元输入不完整或无效（${错误.length} 项）：\n${错误.join("\n")}`,
		);
	return 记录;
}

// 四部分组成完整认同签名，等号同时保证两个方向匹配；不调用含特殊根规则的切片接口。
export function 笔形签名(件: 部件, 索引: number[]): string {
	const 笔形 = 索引.map((i) => 件.笔画列表[i]);
	const 笔名 = 笔形.map(
		(笔) => 默认退化配置.feature?.[笔.feature] ?? 笔.feature,
	);
	const 矩阵 = 索引.map((i) => 索引.map((j) => 件.拓扑.新关系矩阵[i][j]));
	const 划分 = 拓扑.划分连通子图(矩阵);
	return JSON.stringify([笔名, 矩阵, 划分, 拓扑.计算子图关系(划分, 笔形)]);
}

// 每类保存按局部拆法分组的全局拆分编号，Rust 在加载时编译冲突索引。
function 局部约束(部件列表: 部件输入[], 字形: 部件[]): number[][][] {
	const 类表 = new Map<string, Map<string, Set<number>>>();
	let 编号 = 0;
	for (const [i, 件] of 部件列表.entries()) {
		const 缓存 = new Map<number, string>();
		for (const 分法 of 件.拆分) {
			const 根组 = 分法.map((j) => 件.候选[j]);
			const 笔数 = 字形[i].笔画数();
			// 连续区间的首笔必为某个根的首笔。相交根的总笔数等于区间长度时，
			// 这些根恰好完整覆盖区间（包括彼此笔顺交错的根）。
			const 归属: 候选[] = [];
			for (const 根 of 根组) for (const 笔 of 根.笔画) 归属[笔] = 根;
			for (const 首根 of 根组) {
				const 首 = 首根.笔画[0];
				const 所见 = new Set<候选>();
				let 覆盖 = 0;
				for (let 末 = 首; 末 <= 笔数; 末++) {
					const 根 = 归属[末];
					if (根.笔画[0] < 首) break;
					if (!所见.has(根)) {
						所见.add(根);
						覆盖 += 根.笔画.length;
					}
					if (覆盖 !== 末 - 首 + 1) continue;
					const 键 = 首 * (笔数 + 1) + 末;
					let 签名 = 缓存.get(键);
					if (签名 === undefined) {
						const 拓扑 = 笔形签名(
							字形[i],
							Array.from({ length: 覆盖 }, (_, 位) => 首 - 1 + 位),
						);
						// 合法拆分中的每个单元总在一个根内；完整根区间不会切开单元。
						const 单元 = 件.单元
							.filter((组) => 首 <= 组.strokes[0] && 组.strokes[0] <= 末)
							.map((组) => ({
								类别: 组.type,
								笔位: 组.strokes.map((笔) => 笔 - 首).sort((a, b) => a - b),
							}))
							.sort((a, b) => a.笔位[0] - b.笔位[0]);
						签名 = JSON.stringify([拓扑, 单元]);
						缓存.set(键, 签名);
					}
					const 拆法 = JSON.stringify(
						[...所见].map((根) => [根.字根[0], 根.笔画.map((笔) => 笔 - 首)]),
					);
					const 类 = 类表.get(签名) ?? new Map<string, Set<number>>();
					类表.set(签名, 类);
					const 组 = 类.get(拆法) ?? new Set<number>();
					类.set(拆法, 组);
					组.add(编号);
				}
			}
			编号++;
		}
	}
	return [...类表.values()]
		.filter((类) => 类.size > 1)
		.map((类) => [...类.values()].map((组) => [...组]));
}

// 输入中的拆分按根数升序排列。同根数时优先原有枚举顺序。
function 选择初始拆分(
	部件: 部件输入[],
	初始键: Map<string, string>,
	约束: number[][][],
): number[] {
	const 起点: number[] = [];
	let 总数 = 0;
	const 域 = 部件.map((件) => {
		const 起 = 总数;
		起点.push(起);
		总数 += 件.拆分.length;
		const 可用 = 件.候选.map((候) =>
			候.字根.some((名) => {
				const 键 = 初始键.get(名);
				return 键 !== undefined && 小集合.includes(键);
			}),
		);
		const 选择: number[] = [];
		let 最短 = Infinity;
		for (const [i, 分] of 件.拆分.entries()) {
			if (分.length > 最短) break;
			if (分.every((根) => 可用[根])) {
				最短 = 分.length;
				选择.push(起 + i);
			}
		}
		if (!选择.length) throw Error(`初始键位无法拆分 ${根名(件.id)}`);
		return 选择;
	});
	const 有效 = new Set(域.flat());
	const 冲突 = new Map<number, Set<number>>();
	for (const 类 of 约束) {
		const 各组 = 类
			.map((组) => 组.filter((号) => 有效.has(号)))
			.filter((组) => 组.length);
		for (const [i, 组] of 各组.entries())
			for (const 他组 of 各组.slice(i + 1)) {
				for (const 甲 of 组)
					for (const 乙 of 他组) {
						if (!冲突.has(甲)) 冲突.set(甲, new Set());
						if (!冲突.has(乙)) 冲突.set(乙, new Set());
						冲突.get(甲)?.add(乙);
						冲突.get(乙)?.add(甲);
					}
			}
	}
	const 顺序 = 域.map((_, i) => i).sort((a, b) => 域[a].length - 域[b].length);
	const 禁用数 = new Uint32Array(总数);
	const 解: number[] = [];
	function 搜索(步: number): boolean {
		if (步 === 顺序.length) return true;
		const 件 = 顺序[步];
		for (const 号 of 域[件]) {
			if (禁用数[号]) continue;
			解[件] = 号 - 起点[件];
			const 邻居 = 冲突.get(号) ?? [];
			for (const 邻 of 邻居) 禁用数[邻]++;
			if (搜索(步 + 1)) return true;
			for (const 邻 of 邻居) 禁用数[邻]--;
		}
		return false;
	}
	if (!搜索(0))
		throw Error("初始键位下没有满足连续局部及单元一致约束的最少根拆分组合");
	return 解;
}

function 重排特征(件: 部件) {
	const { 新关系矩阵, 划分, 子图关系矩阵 } = 件.拓扑;
	const 笔名 = 件.笔画列表.map(
		({ feature }) => 默认退化配置.feature?.[feature] ?? feature,
	);
	const 子图: number[] = [];
	for (const [i, 笔画] of 划分.entries()) for (const 笔 of 笔画) 子图[笔] = i;
	// 只有边的存在性与方向无关时，才能用原笔序的子图关系剪枝。
	const 子图可重排 = 新关系矩阵.every((行, i) =>
		行.every(
			(值, j) => (值 === undefined) === (新关系矩阵[j][i] === undefined),
		),
	);
	const 关系 = 新关系矩阵.map((行, i) =>
		行.map((值, j) => {
			const 前 = 子图[i];
			const 后 = 子图[j];
			return JSON.stringify([
				值,
				子图可重排 && 前 !== 后 ? 子图关系矩阵[前][后] : null,
			]);
		}),
	);
	const 指纹 = 笔名.map((名, i) =>
		JSON.stringify([
			名,
			笔名.map((邻, j) => JSON.stringify([邻, 关系[i][j], 关系[j][i]])).sort(),
		]),
	);
	return { 件, 关系, 指纹, 分组: JSON.stringify([...指纹].sort()) };
}

type 重排字形 = ReturnType<typeof 重排特征>;

function 查重排(
	原: 重排字形,
	目标: 重排字形,
	目标签名: string,
): number[] | undefined {
	const 笔顺: number[] = [];
	const 已用 = new Set<number>();
	const 对应 = 目标.指纹.map((值) =>
		原.指纹.flatMap((指纹, i) => (值 === 指纹 ? [i] : [])),
	);
	function 搜索(): boolean {
		const 位 = 笔顺.length;
		if (位 === 对应.length) return 笔形签名(原.件, 笔顺) === 目标签名;
		for (const 笔 of 对应[位]) {
			if (
				已用.has(笔) ||
				!笔顺.every(
					(前笔, i) =>
						原.关系[笔][前笔] === 目标.关系[位][i] &&
						原.关系[前笔][笔] === 目标.关系[i][位],
				)
			)
				continue;
			笔顺.push(笔);
			已用.add(笔);
			if (搜索()) return true;
			已用.delete(笔);
			笔顺.pop();
		}
		return false;
	}
	return 搜索() ? 笔顺 : undefined;
}

function 检查重排归并(
	部件根: Iterable<[string, 部件]>,
	字根: ReadonlyMap<string, 根>,
	签名: (件: 部件) => string,
) {
	function 归并终点(名称: string): string {
		const 空间 = 字根.get(名称)?.空间;
		if (空间?.length === 1 && typeof 空间[0] !== "string")
			return 归并终点(空间[0].element);
		return 名称;
	}
	const 分组 = new Map<
		string,
		{ 名称: string; 字形: 重排字形; 终点: string; 签名: string }[]
	>();
	const 冲突: string[] = [];
	for (const [名称, 件] of 部件根) {
		if (件.笔画数() < 2) continue;
		const 字形 = 重排特征(件);
		const 原签名 = 签名(件);
		const 终点 = 归并终点(名称);
		const 列表 = 分组.get(字形.分组) ?? [];
		for (const 目标 of 列表) {
			if (终点 === 目标.终点 || 原签名 === 目标.签名) continue;
			const 笔顺 = 查重排(字形, 目标.字形, 目标.签名);
			if (笔顺)
				冲突.push(
					`${名称} 按 ${笔顺.map((笔) => 笔 + 1).join("、")} 重排，与 ${目标.名称} 签名相同`,
				);
		}
		列表.push({ 名称, 字形, 终点, 签名: 原签名 });
		分组.set(字形.分组, 列表);
	}
	if (冲突.length)
		console.warn(
			`警告：字根笔顺重排冲突（${冲突.length} 对），可在 config.ts 中固定归并到同一主根：\n${冲突.join("\n")}`,
		);
}

function 笔顺连通(件: 部件, 笔画: number[]): boolean {
	const 块: number[][] = [];
	for (const 笔 of 笔画) {
		const 尾 = 块.at(-1);
		if (尾 && 尾.at(-1) === 笔 - 1) 尾.push(笔);
		else 块.push([笔]);
	}
	const 可达 = new Set([0]);
	const 队列 = [0];
	for (const 当前 of 队列) {
		for (let i = 0; i < 块.length; i++) {
			if (
				!可达.has(i) &&
				块[当前].some((甲) =>
					块[i].some((乙) => 件.拓扑.新关系矩阵[甲][乙] !== undefined),
				)
			) {
				可达.add(i);
				队列.push(i);
			}
		}
	}
	return 可达.size === 块.length;
}

function 穷举候选(件: 部件, 单元列表: 单元[]) {
	if (件.笔画数() < 1 || 件.笔画数() > 30)
		throw Error(`部件须有 1–30 笔：${根名(件.id)}`);
	const 原子 = 单元列表.map((组) => 组.strokes.map((笔) => 笔 - 1));
	const 已用 = new Set(原子.flat());
	for (let 笔 = 0; 笔 < 件.笔画数(); 笔++) if (!已用.has(笔)) 原子.push([笔]);
	const 候选列表: { 笔画: number[]; 签名: string }[] = [];
	function 遍历(位: number, 已选: number[]) {
		if (位 < 原子.length) {
			遍历(位 + 1, 已选);
			遍历(位 + 1, [...已选, ...原子[位]]);
			return;
		}
		if (!已选.length) return;
		const 笔画 = [...已选].sort((a, b) => a - b);
		if (笔顺连通(件, 笔画)) 候选列表.push({ 笔画, 签名: 笔形签名(件, 笔画) });
	}
	遍历(0, []);
	return 候选列表.sort(
		(a, b) =>
			a.笔画[0] - b.笔画[0] ||
			b.笔画.length - a.笔画.length ||
			a.签名.localeCompare(b.签名),
	);
}

function 合法拆分(笔数: number, 候选列表: 候选[]): number[][] {
	const 掩码 = 候选列表.map((项) =>
		项.笔画.reduce((值, 笔) => 值 | (2 ** (笔 - 1)), 0),
	);
	const 按首笔 = Array.from({ length: 笔数 }, (_, 笔) =>
		掩码.flatMap((码, i) => (码 & (2 ** 笔) ? [i] : [])),
	);
	const 缓存 = new Map<number, number[][]>([[0, [[]]]]);
	function 枚举(剩余: number): number[][] {
		const 已有 = 缓存.get(剩余);
		if (已有) return 已有;
		const 首笔 = 31 - Math.clz32(剩余 & -剩余);
		const 结果: number[][] = [];
		for (const i of 按首笔[首笔]) {
			const 码 = 掩码[i];
			if ((剩余 & 码) === 码)
				for (const 后续 of 枚举(剩余 ^ 码)) 结果.push([i, ...后续]);
		}
		缓存.set(剩余, 结果);
		return 结果;
	}
	return 枚举(2 ** 笔数 - 1).sort((a, b) => a.length - b.length);
}

function 分部取根(
	形: Exclude<字形, 部件>,
	查根: (形: 字形) => string | undefined,
	优先: Set<number>,
): 分部 {
	const 部分 = 形.部分列表;
	const 首 = 部分[0];
	const 末 = 部分[部分.length - 1];
	const 包围 = !/[⿰⿲⿱⿳]/.test(形.结构描述字符);
	const 首根 = 查根(首);
	const 末根 = 查根(末);
	const 取末 =
		首根 !== undefined
			? 末根 !== undefined && 部分.length === 2 && !包围 && 优先.has(形.id)
			: 末根 !== undefined || 包围;
	const 位置 = 取末 ? 部分.length - 1 : 0;
	const 正部 = 部分[位置];
	let 正形 = 正部;
	let 正根 = 查根(正形);
	while (正根 === undefined && !是部件(正形)) {
		正形 = 正形.部分列表[0];
		正根 = 查根(正形);
	}
	const 余部 = 部分.filter((_, i) => i !== 位置);
	function 首部件(部分: 字形): number {
		let 当前 = 部分;
		while (!是部件(当前)) 当前 = 当前.部分列表[0];
		return 当前.id;
	}
	let 余根: 分部["余根"];
	if (余部.length === 1 && 是部件(余部[0]))
		余根 = [{ 部件: 余部[0].id, 数量: 2 }];
	else {
		const 两部 =
			余部.length === 1 && !是部件(余部[0])
				? 余部[0].部分列表.slice(0, 2)
				: 余部.slice(0, 2);
		余根 = 两部.map((部) => ({ 部件: 首部件(部), 数量: 1 }));
	}
	return {
		id: 形.id,
		正部: 正部.id,
		正根: 正根 === undefined ? { 部件: 正形.id } : { 字根: 正根 },
		余部: 余部.map((部) => 部.id),
		余根,
	};
}

// 按实际查根分支展开布尔条件，覆盖任意启用子集，不以全开或逐根试跑代替证明。
class 未定条件 extends Error {
	constructor(public 签名: string) {
		super("未定备选条件");
	}
}
function 证明备选(
	复合体: Exclude<字形, 部件>[],
	基线: Map<number, 分部>,
	查初始根: (形: 字形) => string | undefined,
	签名: (件: 部件) => string,
	备选: Map<string, string>,
	整件签名: ReadonlyMap<number, string>,
	优先: Set<number>,
) {
	const 危险 = new Map<string, number[]>();
	let 分支数 = 0;
	for (const 形 of 复合体) {
		const 冻结分部 = 确保存在(基线.get(形.id), "缺少冻结分部");
		const 冻结正根 = 冻结分部.正根;
		const 冻结部件签名 =
			"部件" in 冻结正根 ? 整件签名.get(冻结正根.部件) : undefined;
		function 验证(条件: Map<string, boolean>) {
			try {
				const 结果 = 分部取根(
					形,
					(子形) => {
						const 初始 = 查初始根(子形);
						if (初始 !== undefined || !是部件(子形)) return 初始;
						const 键 = 签名(子形);
						const 新根 = 备选.get(键);
						if (新根 === undefined) return;
						const 启用 = 条件.get(键);
						if (启用 === undefined) throw new 未定条件(键);
						return 启用 ? 新根 : undefined;
					},
					优先,
				);
				分支数++;
				// 启用的备选必在小集合；它若覆盖冻结部件的全部笔画，
				// 该部件唯一的最少根拆分就是这个根，仍可沿用首根请求。
				if (
					冻结部件签名 !== undefined &&
					条件.get(冻结部件签名) === true &&
					"字根" in 结果.正根 &&
					结果.正根.字根 === 备选.get(冻结部件签名)
				)
					结果.正根 = 冻结正根;
				if (!相同(结果, 冻结分部)) {
					for (const [键, 启用] of 条件)
						if (启用) {
							const 列表 = 危险.get(键) ?? [];
							if (!列表.includes(形.id)) 列表.push(形.id);
							危险.set(键, 列表);
						}
				}
			} catch (错) {
				if (!(错 instanceof 未定条件)) throw 错;
				验证(new Map([...条件, [错.签名, false]]));
				验证(new Map([...条件, [错.签名, true]]));
			}
		}
		验证(new Map());
	}
	return { 危险, 分支数 };
}

export function 准备飞花(): 飞花输入 {
	const 开始 = performance.now();
	const 字库 = 获取字库(方案);
	const 融合集合 = 获取融合字形集合({}, 方案.data);
	const 分析缓存 = new Map<number, 字形>();
	const 展开笔画 = (形: 字形) =>
		new 部件({
			id: 形.id,
			type: "component",
			operator: undefined,
			references: undefined,
			strokes: 形.图形盒子.获取笔画列表(),
		});
	function 融合(形: 字形): 字形 {
		const 已有 = 分析缓存.get(形.id);
		if (已有) return 已有;
		let 结果: 字形 = 形;
		if (!是部件(形)) {
			if (融合集合.has(形.id)) 结果 = 展开笔画(形);
			else 结果 = 形.替换分析部分(形.部分列表.map(融合));
		}
		分析缓存.set(形.id, 结果);
		return 结果;
	}
	const 取字形 = (id: number) =>
		融合(确保存在(字库.获取字形(id), `缺少字形：${根名(id)}`));
	const { 字根, 初始键 } = 检查空间(方案, 取字形);
	const 分类器 = 合并分类器(方案.analysis?.classifier);
	const 全形签名 = new Map<number, string>();
	function 签名(件: 部件): string {
		let 值 = 全形签名.get(件.id);
		if (值 === undefined) {
			值 = 笔形签名(
				件,
				件.笔画列表.map((_, i) => i),
			);
			全形签名.set(件.id, 值);
		}
		return 值;
	}
	const 同形根 = new Map<string, 根[]>();
	const 重排根 = new Map<string, 部件>();
	for (const 根 of 字根.values()) {
		if (/^[1-5]$/.test(根.名称)) continue;
		const 形 = 取字形(Number.parseInt(根.名称.slice(3), 16));
		重排根.set(根.名称, 是部件(形) ? 形 : 展开笔画(形));
		if (!是部件(形)) continue;
		const 键 = 签名(形);
		const 列表 = 同形根.get(键) ?? [];
		列表.push(根);
		同形根.set(键, 列表);
	}
	const 已配置认同 = new Map<string, string>();
	const 同形附形 = new Set<string>();
	for (const [键, 列表] of 同形根) {
		const 同形名称 = new Set(列表.map((根) => 根.名称));
		const 主根 = 列表.filter(
			({ 空间: [安排, ...其余] }) =>
				其余.length > 0 ||
				typeof 安排 === "string" ||
				!同形名称.has(安排.element),
		);
		if (主根.length !== 1)
			throw Error(
				`同形部件字根必须固定归并到唯一主根：${[...同形名称].join("、")}；当前主根：${主根.map((根) => 根.名称).join("、")}`,
			);
		const 主根名称 = 主根[0].名称;
		已配置认同.set(键, 主根名称);
		for (const 根 of 列表) {
			if (根.名称 !== 主根名称) 同形附形.add(根.名称);
		}
	}
	const 词典 = 获取原始词典();
	const 可声母归并 = new Set(
		[...字根.values()]
			.filter((根) => 根.落键 !== "小集合" && !同形附形.has(根.名称))
			.map((根) => 根.名称),
	);
	for (const [名称, 安排列表] of 生成声母归并(字库, 词典, 可声母归并)) {
		const 根 = 确保存在(字根.get(名称), `缺少字根：${名称}`);
		for (const 安排 of 安排列表) {
			if (!根.空间.some((项) => 相同(项, 安排))) 根.空间.push(安排);
		}
	}
	function 查初始根(形: 字形): string | undefined {
		if (是部件(形)) {
			if (形.笔画数() === 1) return String(分类器[形.笔画列表[0].feature]);
			return 已配置认同.get(签名(形));
		}
		const 名 = 根名(形.id);
		return 字根.has(名) ? 名 : undefined;
	}
	const 字列表: 字输入[] = [];
	const 部件集合 = new Map<number, 部件>();
	const 复合体集合 = new Map<number, Exclude<字形, 部件>>();
	const 优先 = new Set<number>();
	for (const 字 of 意符优先字) {
		const 记录 = 确保存在(字库.校验字符(字), `意符优先字缺失：${字}`);
		for (const { id } of 记录.final_glyphs) 优先.add(id);
	}
	function 收集(形: 字形) {
		if (是部件(形)) 部件集合.set(形.id, 形);
		else if (!复合体集合.has(形.id)) {
			复合体集合.set(形.id, 形);
			for (const 子形 of 形.部分列表) 收集(子形);
		}
	}
	for (const [, 记录] of 字库) {
		if (!字集过滤查找表.minimal(记录.character, 记录)) continue;
		if (!记录.final_glyphs.length)
			throw Error(`minimal 字无可用字形：${记录.character.获取名称()}`);
		for (const { id } of 记录.final_glyphs) {
			const 形 = 取字形(id);
			收集(形);
			字列表.push({ 字: 记录.character.获取名称(), 字形: id, 声母: [] });
		}
	}
	const 分部列表 = [...复合体集合.values()].sort((a, b) => a.id - b.id);
	const 冻结 = new Map(
		分部列表.map((形) => [形.id, 分部取根(形, 查初始根, 优先)]),
	);
	// 编码只读取字本身的冻结分部；子结构直接从字形树定位部件。
	const 编码字形 = new Set(字列表.map((字) => 字.字形));
	const 编码复合体 = 分部列表.filter((形) => 编码字形.has(形.id));
	const 部件列表 = [...部件集合.values()].sort((a, b) => a.id - b.id);
	const 单元记录 = 读取单元(部件列表);
	console.log(
		`空间校验、分部冻结及单元读取完成：${字列表.length} 字形，${部件列表.length} 部件，${分部列表.length} 复合体`,
	);

	const 所有候选 = 部件列表.map((件) => ({
		件,
		候选: 穷举候选(件, 确保存在(单元记录.get(件.id), "缺少单元")),
	}));
	const 认同类 = new Map<
		string,
		{ 件: 部件; 笔画: number[]; 已有: number[] }
	>();
	const 整件签名 = new Map<number, string>();
	for (const { 件, 候选 } of 所有候选) {
		for (const 项 of 候选) {
			if (项.笔画.length === 件.笔画数()) 整件签名.set(件.id, 项.签名);
			if (项.笔画.length > 1 && !认同类.has(项.签名))
				认同类.set(项.签名, { 件, 笔画: 项.笔画, 已有: [] });
		}
	}
	// 全库仅扫描一次。配置的融合件采用融合后的完整笔形。
	for (const [id, 原形] of 字库.字形迭代器()) {
		const 形 = 是部件(原形) || 融合集合.has(id) ? 融合(原形) : undefined;
		if (!形 || !是部件(形)) continue;
		const 类 = 认同类.get(签名(形));
		if (类) 类.已有.push(id);
	}
	const 分配ID = 创建字形ID分配器(
		[...字库.字形迭代器()].map(([id]) => id),
		0xf0000,
		0xfffff,
	);
	const 备选 = new Map<string, string>();
	const 自定义 = new Map<string, 基本部件数据>();
	for (const [键, 类] of [...认同类].sort(([a], [b]) => a.localeCompare(b))) {
		if (已配置认同.has(键)) continue;
		const id = 类.已有.length ? Math.min(...类.已有) : 分配ID();
		const 名 = 根名(id);
		备选.set(键, 名);
		if (!类.已有.length)
			自定义.set(键, {
				id,
				type: "component",
				operator: undefined,
				references: undefined,
				strokes: 类.笔画.map((i) => 类.件.矢量图形[i]),
			});
	}
	const 证明 = 证明备选(编码复合体, 冻结, 查初始根, 签名, 备选, 整件签名, 优先);
	const 安全备选 = new Map([...备选].filter(([键]) => !证明.危险.has(键)));
	const 复核 = 证明备选(
		编码复合体,
		冻结,
		查初始根,
		签名,
		安全备选,
		整件签名,
		优先,
	);
	if (复核.危险.size) throw Error("备选子集证明失败");
	for (const [键, 名] of 安全备选) {
		const 自定义字形 = 自定义.get(键);
		const 形 = 自定义字形
			? new 部件(自定义字形)
			: 取字形(Number.parseInt(名.slice(3), 16));
		if (!是部件(形)) throw Error(`备选字根必须是部件：${名}`);
		重排根.set(名, 形);
	}
	检查重排归并(重排根, 字根, 签名);
	console.log(
		`字根认同完成：${认同类.size} 类，备选保留 ${安全备选.size}，剔除 ${证明.危险.size}，证明 ${证明.分支数 + 复核.分支数} 条分支`,
	);
	const 取多笔根 = (签名: string) => 已配置认同.get(签名) ?? 安全备选.get(签名);
	const 部件输入列表: 部件输入[] = [];
	for (const { 件, 候选 } of 所有候选) {
		const 可用候选: 候选[] = [];
		for (const 项 of 候选) {
			const 名 =
				项.笔画.length === 1
					? String(分类器[件.笔画列表[项.笔画[0]].feature])
					: 取多笔根(项.签名);
			if (名 !== undefined)
				可用候选.push({ 笔画: 项.笔画.map((i) => i + 1), 字根: [名] });
		}
		const 拆分 = 合法拆分(件.笔画数(), 可用候选);
		部件输入列表.push({
			id: 件.id,
			单元: 确保存在(单元记录.get(件.id), "缺少单元"),
			候选: 可用候选,
			拆分,
		});
	}
	选核心根(字根, 部件输入列表);
	const 生成配置 = structuredClone(方案);
	const 表 = 确保存在(生成配置.form, "缺少 form");
	表.mapping_space ??= {};
	for (const 根 of 字根.values()) {
		const 键 = 根.空间.filter((项): 项 is string => typeof 项 === "string");
		const 范围 = 键域[根.落键];
		const 安排: 广义安排[] =
			键.length === 范围.length
				? [[{ variable: 根.落键 }]]
				: 键.filter((项) => 项 !== 根.初始);
		安排.push(
			...根.空间.filter((项) => typeof 项 !== "string" && !相同(项, 根.初始)),
		);
		if (安排.length)
			表.mapping_space[根.名称] = 安排.map((value) => ({ value, score: 0 }));
		else delete 表.mapping_space[根.名称];
	}
	生成配置.data ??= {};
	生成配置.data.glyphs ??= [];
	for (const [键, 名] of 安全备选) {
		表.mapping_space[名] = [{ value: [{ variable: "小集合" }], score: 0 }];
		字根.set(名, {
			名称: 名,
			落键: "小集合",
			核心: false,
			初始: null,
			空间: [...小集合],
		});
		const 形 = 自定义.get(键);
		if (形) 生成配置.data.glyphs.push(形);
	}
	const 声规则 = 确保存在(
		合并拼写运算(方案.algebra).get("声"),
		"缺少声母拼写运算",
	);
	const 字音 = new Map<string, Set<string>>();
	for (const 条 of 词典) {
		if ([...条.词].length !== 1 || 条.拼音.length !== 1) continue;
		const 声 = 应用拼写运算(声规则, 条.拼音[0]);
		const 键 = 表.mapping[`声-${声}`];
		if (typeof 键 !== "string" || 键.length !== 1 || !补集合.includes(键))
			throw Error(`声母没有固定键位：${条.词} ${条.拼音[0]}`);
		const 集 = 字音.get(条.词) ?? new Set<string>();
		集.add(键);
		字音.set(条.词, 集);
	}
	for (const 字 of 字列表)
		字.声母 = [...确保存在(字音.get(字.字), `字缺少读音：${字.字}`)].sort();
	const 字根字 = 部件列表.flatMap((件) => {
		const 名 =
			件.笔画数() === 1
				? String(分类器[件.笔画列表[0].feature])
				: 取多笔根(签名(件));
		return 名 === undefined ? [] : [{ 字形: 件.id, 字根: [名], 重复: 1 }];
	});
	const 约束 = 局部约束(
		部件输入列表,
		所有候选.map(({ 件 }) => 件),
	);
	const 输入: 飞花输入 = {
		版本: 3,
		局部约束: 约束,
		初始拆分: 选择初始拆分(部件输入列表, 初始键, 约束),
		配置: 生成配置,
		字根: [...字根.values()],
		部件: 部件输入列表,
		复合体: [...冻结.values()],
		字: 字列表,
		字根字,
	};
	const 报告图形 = new Map<number, 矢量图形数据>();
	for (const 形 of [...部件列表, ...分部列表])
		报告图形.set(形.id, 是部件(形) ? 形.矢量图形 : 形.图形盒子.获取笔画列表());
	const 自定义图形 = new Map(
		[...自定义.values()].map((形) => [形.id, 形.strokes]),
	);
	for (const 名称 of new Set([...字根.keys(), ...备选.values()])) {
		if (!名称.startsWith("字形-")) continue;
		const id = Number.parseInt(名称.slice(3), 16);
		if (报告图形.has(id)) continue;
		const 自定 = 自定义图形.get(id);
		if (自定 !== undefined) 报告图形.set(id, 自定);
		else {
			const 形 = 取字形(id);
			报告图形.set(id, 是部件(形) ? 形.矢量图形 : 形.图形盒子.获取笔画列表());
		}
	}
	const 报告 = 生成报告(
		输入,
		报告图形,
		[...证明.危险].map(([键, 影响]) => ({
			字根: 确保存在(备选.get(键), "备选缺失"),
			影响,
		})),
	);
	mkdirSync(输出目录, { recursive: true });
	writeFileSync(
		resolve(输出目录, "generated.yaml"),
		dump(生成配置, { noRefs: true, lineWidth: 120 }),
	);
	writeFileSync(resolve(输出目录, "input.json"), JSON.stringify(输入));
	writeFileSync(resolve(输出目录, "index.html"), 报告);
	console.log(
		`完成：${((performance.now() - 开始) / 1000).toFixed(1)} 秒，输出 ${输出目录}`,
	);
	return 输入;
}

function 生成报告(
	输入: 飞花输入,
	报告图形: ReadonlyMap<number, 矢量图形数据>,
	危险: { 字根: string; 影响: number[] }[],
) {
	const 来源部件 = new Map<string, Set<number>>();
	for (const 件 of 输入.部件)
		for (const 候选 of 件.候选)
			for (const 名 of 候选.字根) {
				if (!名.startsWith("字形-f")) continue;
				const 来源 = 来源部件.get(名) ?? new Set<number>();
				来源.add(件.id);
				来源部件.set(名, 来源);
			}
	const 默认笔画 = Object.entries(合并分类器());
	const 笔画顺位 = new Map(默认笔画.map(([笔], 序) => [笔, 序]));
	const 单笔名称 = [1, 2, 3, 4, 5].map(
		(类) =>
			确保存在(
				默认笔画.find(([, 值]) => 值 === 类),
				"缺少单笔类别",
			)[0],
	);
	const 字根 = 输入.字根.map((根) => {
		const id = 根.名称;
		const 笔画名称 = id.startsWith("字形-")
			? 确保存在(
					报告图形.get(Number.parseInt(id.slice(3), 16)),
					`缺少字根图形：${id}`,
				).map((笔) => 笔.feature)
			: [单笔名称[Number(id) - 1]];
		return { id, 笔画名称, 根 };
	});
	const 数据 = JSON.stringify({
		...输入,
		配置: undefined,
		键域,
		字根: 字根
			.sort((甲, 乙) => 比较笔画顺序(甲, 乙, 笔画顺位))
			.map(({ 根 }) => ({
				...根,
				来源: 根.名称.startsWith("字形-f")
					? [...(来源部件.get(根.名称) ?? [])]
					: undefined,
			})),
		图形: [...报告图形].map(([id, 笔画]) => ({
			id,
			路径: 生成笔画路径(笔画),
		})),
		危险,
	}).replaceAll("<", "\\u003c");
	const 模板 = readFileSync(
		new URL("./inspection.html", import.meta.url),
		"utf8",
	);
	return 模板.replace('"{{data}}"', () => 数据);
}

if (
	process.argv[1] &&
	pathToFileURL(resolve(process.argv[1])).href === import.meta.url
) {
	if (process.argv.length !== 2)
		throw Error("用法：bun src/feihua/prepare.ts（无参数）");
	准备飞花();
}
