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
import 方案 from "./config.ts";

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
type 类别 = 1 | 2 | 3 | 4 | 5;
type 根 = { 名称: string; 类别: 类别; 初始: 安排 | null; 空间: 安排[] };
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
	版本: 1;
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
		字根.set(名, { 名称: 名, 类别: 4, 初始: 初始列表[0], 空间: 唯一空间 });
	}
	const 已分 = new Set<string>();
	function 分类(名: string, 路径 = new Set<string>()): 类别 {
		const 根 = 确保存在(字根.get(名), `归并到未声明字根：${名}`);
		if (已分.has(名)) return 根.类别;
		if (路径.has(名)) throw Error(`空间归并成环：${[...路径, 名].join(" → ")}`);
		const 下层 = new Set([...路径, 名]);
		const 直接 = 根.空间.filter((项): 项 is string => typeof 项 === "string");
		const 归并 = 根.空间.filter(
			(项): 项 is { element: string } => typeof 项 !== "string",
		);
		const 声母 = 归并.filter((项) => 项.element.startsWith("声-"));
		for (const { element } of 声母) {
			const 键 = 键盘.mapping[element];
			if (typeof 键 !== "string" || 键.length !== 1 || !补集合.includes(键))
				throw Error(`声母锚点必须固定在补集合：${element}`);
			if (键盘.mapping_space?.[element]?.length)
				throw Error(`声母锚点不能移动：${element}`);
		}
		const 目标 = 归并
			.filter((项) => !项.element.startsWith("声-"))
			.map((项) => 分类(项.element, 下层));
		const 是全键集 = (键集: string) => 相同([...直接].sort(), [...键集].sort());
		if (/^[1-5]$/.test(名)) {
			if (直接.length !== 1 || 归并.length || !小集合.includes(直接[0]))
				throw Error(`笔画必须固定在 B：${名}`);
			根.类别 = 1;
		} else if (
			!声母.length &&
			(!直接.length || 是全键集(小集合)) &&
			目标.every((类) => 类 <= 2)
		)
			根.类别 = 2;
		else if (
			((!直接.length && !声母.length) ||
				(是全键集(补集合) && 声母.length > 0)) &&
			目标.every((类) => 类 === 3)
		)
			根.类别 = 3;
		else if (
			((!直接.length && !声母.length) ||
				(是全键集(全集合) && 声母.length > 0)) &&
			目标.every((类) => 类 >= 2 && 类 <= 4)
		)
			根.类别 = 4;
		else throw Error(`字根空间不属于五类：${名}`);
		if (
			根.类别 !== 3 &&
			!/^[1-5]$/.test(名) &&
			!是部件(取字形(Number.parseInt(名.slice(3), 16)))
		)
			throw Error(`复合体不能进入 B：${名}`);
		已分.add(名);
		return 根.类别;
	}
	for (const 名 of 字根.keys()) 分类(名);
	for (let 笔 = 1; 笔 <= 5; 笔++)
		确保存在(字根.get(String(笔)), `缺少基础笔画 ${笔}`);
	return 字根;
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
	优先: Set<number>,
) {
	const 危险 = new Map<string, number[]>();
	let 分支数 = 0;
	for (const 形 of 复合体) {
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
				if (!相同(结果, 基线.get(形.id))) {
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
	function 融合(形: 字形): 字形 {
		const 已有 = 分析缓存.get(形.id);
		if (已有) return 已有;
		let 结果: 字形 = 形;
		if (!是部件(形)) {
			if (融合集合.has(形.id))
				结果 = new 部件({
					id: 形.id,
					type: "component",
					operator: undefined,
					references: undefined,
					strokes: 形.图形盒子.获取笔画列表(),
				});
			else 结果 = 形.替换分析部分(形.部分列表.map(融合));
		}
		分析缓存.set(形.id, 结果);
		return 结果;
	}
	const 取字形 = (id: number) =>
		融合(确保存在(字库.获取字形(id), `缺少字形：${根名(id)}`));
	const 字根 = 检查空间(方案, 取字形);
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
	const 已配置认同 = new Map<string, string[]>();
	for (const 根 of 字根.values()) {
		if (/^[1-5]$/.test(根.名称)) continue;
		const 形 = 取字形(Number.parseInt(根.名称.slice(3), 16));
		if (!是部件(形)) continue;
		const 键 = 签名(形);
		const 列表 = 已配置认同.get(键) ?? [];
		列表.push(根.名称);
		已配置认同.set(键, 列表.sort());
	}
	function 查初始根(形: 字形): string | undefined {
		if (字根.has(根名(形.id))) return 根名(形.id);
		if (是部件(形)) {
			if (形.笔画数() === 1) return String(分类器[形.笔画列表[0].feature]);
			return 已配置认同.get(签名(形))?.[0];
		}
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
	for (const { 件, 候选 } of 所有候选)
		for (const 项 of 候选) {
			if (项.笔画.length > 1 && !认同类.has(项.签名))
				认同类.set(项.签名, { 件, 笔画: 项.笔画, 已有: [] });
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
	const 证明 = 证明备选(分部列表, 冻结, 查初始根, 签名, 备选, 优先);
	const 安全备选 = new Map([...备选].filter(([键]) => !证明.危险.has(键)));
	const 复核 = 证明备选(分部列表, 冻结, 查初始根, 签名, 安全备选, 优先);
	if (复核.危险.size) throw Error("备选子集证明失败");
	console.log(
		`字根认同完成：${认同类.size} 类，备选保留 ${安全备选.size}，剔除 ${证明.危险.size}，证明 ${证明.分支数 + 复核.分支数} 条分支`,
	);
	const 部件输入列表: 部件输入[] = [];
	const 覆盖不足: string[] = [];
	for (const { 件, 候选 } of 所有候选) {
		const 可用候选: 候选[] = [];
		for (const 项 of 候选) {
			const 根列表 =
				项.笔画.length === 1
					? [String(分类器[件.笔画列表[项.笔画[0]].feature])]
					: (已配置认同.get(项.签名) ??
						(安全备选.has(项.签名)
							? [确保存在(安全备选.get(项.签名), "备选缺失")]
							: []));
			if (根列表.length)
				可用候选.push({ 笔画: 项.笔画.map((i) => i + 1), 字根: 根列表 });
		}
		const 拆分 = 合法拆分(件.笔画数(), 可用候选);
		if (
			!拆分.some((分法) =>
				分法.every((i) =>
					可用候选[i].字根.some((名) => {
						const 类 = 字根.get(名)?.类别;
						return 类 === 1 || 类 === 2;
					}),
				),
			)
		)
			覆盖不足.push(`${根名(件.id)}（${件.笔画数()} 笔）`);
		部件输入列表.push({
			id: 件.id,
			单元: 确保存在(单元记录.get(件.id), "缺少单元"),
			候选: 可用候选,
			拆分,
		});
	}
	if (覆盖不足.length)
		throw Error(
			`永久 B 覆盖不足（${覆盖不足.length} 个部件），未写出产物：\n${覆盖不足.join("\n")}`,
		);
	const 生成配置 = structuredClone(方案);
	const 表 = 确保存在(生成配置.form, "缺少 form");
	表.mapping_space ??= {};
	生成配置.data ??= {};
	生成配置.data.glyphs ??= [];
	for (const [键, 名] of 安全备选) {
		表.mapping_space[名] = [{ value: [{ variable: "小集合" }], score: 0 }];
		字根.set(名, { 名称: 名, 类别: 5, 初始: null, 空间: [...小集合] });
		const 形 = 自定义.get(键);
		if (形) 生成配置.data.glyphs.push(形);
	}
	const 声规则 = 确保存在(
		合并拼写运算(方案.algebra).get("声"),
		"缺少声母拼写运算",
	);
	const 字音 = new Map<string, Set<string>>();
	for (const 条 of 获取原始词典()) {
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
		const 根列表 =
			件.笔画数() === 1
				? [String(分类器[件.笔画列表[0].feature])]
				: (已配置认同.get(签名(件)) ??
					(安全备选.has(签名(件))
						? [确保存在(安全备选.get(签名(件)), "备选缺失")]
						: []));
		return 根列表.length ? [{ 字形: 件.id, 字根: 根列表, 重复: 1 }] : [];
	});
	const 输入: 飞花输入 = {
		版本: 1,
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
	const 图形 = [...报告图形].map(([id, 笔画]) => ({
		id,
		路径: 生成笔画路径(笔画),
	}));
	const 报告 = 生成报告(
		输入,
		图形,
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
	图形: { id: number; 路径: string[] }[],
	危险: { 字根: string; 影响: number[] }[],
) {
	const 数据 = JSON.stringify({
		...输入,
		配置: undefined,
		图形,
		危险,
	}).replaceAll("<", "\\u003c");
	const 模板 = `<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>飞花分部、字根与拆分核查</title>
<style>body{font:16px system-ui;max-width:1100px;margin:32px auto;padding:0 20px;color:#223}input,select,button{font:inherit;padding:8px;margin:6px}svg{width:110px;height:110px;vertical-align:middle}summary{cursor:pointer;padding:10px 0}details{border-bottom:1px solid #ddd}code{overflow-wrap:anywhere}pre{white-space:pre-wrap}nav{position:sticky;top:0;background:white;padding:12px;z-index:1}path{fill:none;stroke-linecap:round;stroke-linejoin:round;stroke-width:2}.card{border-bottom:1px solid #ddd;padding:12px 0}.preview{display:inline-block;text-align:center;margin-right:20px}.choices{max-height:500px;overflow:auto}</style>
<h1>飞花分部、字根与拆分核查</h1><p id="stats"></p><nav><select id="mode" aria-label="核查内容"><option value="部件">部件与合法拆分</option><option value="复合体">冻结分部</option><option value="字根">字根与决策空间</option><option value="危险">剔除备选</option><option value="字">字符与声母</option></select><input id="query" aria-label="搜索" placeholder="输入汉字、字形 ID 或字根"><button type="button" id="prev">上一页</button><button type="button" id="next">下一页</button><span id="page"></span></nav><main id="rows"></main>
<script>
const data = "{{data}}";
const byId = new Map(data.图形.map((x) => [x.id, x.路径]));
const chars = new Map();
for (const x of data.字) chars.set(x.字形, (chars.get(x.字形) || "") + x.字);
const colors = ["#b44", "#37a", "#398", "#a73", "#859", "#688"];
let page = 0;
const mode = document.getElementById("mode"),
	query = document.getElementById("query"),
	rows = document.getElementById("rows");
const name = (id) => "字形-" + id.toString(16).padStart(5, "0");
const escape = (s) =>
	String(s).replace(
		/[&<>"']/g,
		(c) =>
			({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
				c
			],
	);
const svg = (id, units = []) =>
	'<svg viewBox="0 0 100 100">' +
	(byId.get(id) || [])
		.map(
			(d, i) =>
				'<path stroke="' +
				(colors[units.findIndex((u) => u.strokes.includes(i + 1))] || "#222") +
				'" d="' +
				escape(d) +
				'"><title>第 ' +
				(i + 1) +
				" 笔</title></path>",
		)
		.join("") +
	"</svg>";
const preview = (id) =>
	'<span class="preview">' +
	svg(id) +
	"<br>" +
	name(id) +
	" " +
	escape(chars.get(id) || "") +
	"</span>";
const rootSvg = (r) =>
	r.startsWith("字形-")
		? svg(parseInt(r.slice(3), 16))
		: ["横", "竖", "撇", "点", "折"][Number(r) - 1];
document.getElementById("stats").textContent =
	data.字.length +
	" 条字符字形；" +
	data.部件.length +
	" 个部件；" +
	data.复合体.length +
	" 个复合体；" +
	data.字根.length +
	" 个字根；剔除 " +
	data.危险.length +
	" 个备选。";
function row(x) {
	if (mode.value === "部件")
		return (
			"<details><summary>" +
			name(x.id) +
			" " +
			escape(chars.get(x.id) || "") +
			" · " +
			x.候选.length +
			" 个候选 · " +
			x.拆分.length +
			" 种拆分</summary>" +
			svg(x.id, x.单元) +
			"<p>单元：" +
			escape(JSON.stringify(x.单元)) +
			'</p><details><summary>字根候选</summary><div class="choices">' +
			x.候选
				.map(
					(c, i) =>
						"<p>#" +
						i +
						" " +
						svg(x.id, [{ strokes: c.笔画 }]) +
						" " +
						escape(c.字根.join(" / ")) +
						"（第 " +
						c.笔画.join("、") +
						" 笔）</p>",
				)
				.join("") +
			'</div></details><details><summary>全部合法拆分（按根数排序）</summary><div class="choices">' +
			x.拆分
				.map(
					(s) =>
						"<p>" +
						s
							.map((i) => escape(x.候选[i].字根.join("/")) + " [#" + i + "]")
							.join(" + ") +
						"</p>",
				)
				.join("") +
			"</div></details></details>"
		);
	if (mode.value === "复合体")
		return (
			'<div class="card">' +
			preview(x.id) +
			"<p>正部：" +
			preview(x.正部) +
			"</p><p>正根：" +
			("字根" in x.正根
				? rootSvg(x.正根.字根) + " " + x.正根.字根
				: "取 " + name(x.正根.部件) + " 的首根") +
			"</p><p>余部：" +
			x.余部.map(preview).join("") +
			"</p><p>余根：" +
			x.余根
				.map((r) => "取 " + name(r.部件) + " 的前 " + r.数量 + " 根")
				.join("；") +
			"</p></div>"
		);
	if (mode.value === "字根")
		return (
			'<div class="card">' +
			rootSvg(x.名称) +
			" <b>" +
			x.名称 +
			"</b><p>第 " +
			x.类别 +
			" 类；初始安排：" +
			(x.初始 === null ? "未启用" : escape(JSON.stringify(x.初始))) +
			"</p><p>空间：" +
			escape(JSON.stringify(x.空间)) +
			"</p></div>"
		);
	if (mode.value === "危险")
		return (
			"<details><summary>" +
			rootSvg(x.字根) +
			" " +
			x.字根 +
			" · 影响 " +
			x.影响.length +
			" 个复合体</summary>" +
			x.影响.map(preview).join("") +
			"</details>"
		);
	return (
		"<p>" +
		escape(x.字) +
		" · " +
		name(x.字形) +
		" · 声母键 " +
		x.声母.join("、") +
		"</p>"
	);
}
function render() {
	const q = query.value.trim();
	const ids = new Set(data.字.filter((x) => x.字 === q).map((x) => x.字形));
	const names = [...ids].map(name);
	const all = data[mode.value].filter(
		(x) =>
			!q ||
			JSON.stringify(x).includes(q) ||
			(x.id !== undefined && (name(x.id).includes(q) || ids.has(x.id))) ||
			names.includes(x.名称) ||
			names.includes(x.字根),
	);
	const total = Math.max(1, Math.ceil(all.length / 40));
	page = Math.max(0, Math.min(page, total - 1));
	document.getElementById("page").textContent =
		page + 1 + " / " + total + "（" + all.length + " 项）";
	rows.innerHTML = all
		.slice(page * 40, (page + 1) * 40)
		.map(row)
		.join("");
}
const restart = () => {
	page = 0;
	render();
};
mode.onchange = restart;
query.oninput = restart;
document.getElementById("prev").onclick = () => {
	page--;
	render();
};
document.getElementById("next").onclick = () => {
	page++;
	render();
};
render();
</script></html>`;
	return 模板.replace('"{{data}}"', 数据);
}

if (
	process.argv[1] &&
	pathToFileURL(resolve(process.argv[1])).href === import.meta.url
) {
	if (process.argv.length !== 2)
		throw Error("用法：bun src/feihua/prepare.ts（无参数）");
	准备飞花();
}
