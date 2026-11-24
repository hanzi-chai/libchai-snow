import { readFileSync } from "node:fs";
import {
	GF0014映射,
	type 决策,
	type 决策空间,
	type 原始词典,
	合并拼写运算,
	type 基本部件数据,
	type 复合体数据,
	type 字库,
	type 字形拼写运算,
	应用拼写运算,
	type 模式,
	type 矢量图形数据,
	type 结构描述字符,
	type 配置,
} from "hanzi-chai";

const 小集合 = "eiuoav;/";
const 补集合 = "bpmfdtnlgkhjqxzcsryw,.";
const 键集 = { 小集合, 全集合: 小集合 + 补集合, 补集合 };
const 根名 = (字形: number) => `字形-${字形.toString(16).padStart(5, "0")}`;

type 笔画引用 = { element: "1" | "2" | "3" | "4" | "5" };
type 字根安排 = string | number | 笔画引用;
// 每行依次为字形 ID、初始安排及决策空间；数字安排表示归并到对应字形。
// 「自由」开放字形允许的全部按键；「小集合自由」另外要求该部件始终在小集合。
// 也可列出部分键和形似目标。声母锚点在预处理确定最终键域后生成。
const 字根配置: [
	字形: number,
	初始: 字根安排,
	...空间: (
		| "自由"
		| "小集合自由"
		| "保留小集合"
		| number
		| 笔画引用
		| readonly string[]
	)[],
][] = [
	[0x2a41f, "j", "自由"],
	[0x48000, "d", "自由"],
	[0x0036a, ";", "小集合自由"], // 日
	[0x0036b, 0x0036a], // 曰
	[0x00371, 0x0036a], // 冒字头
	[0x003b2, "y", "自由"],
	[0x00368, 0x003b2, "自由"],
	[0x04656, "j", "自由"],
	[0x00486, 0x04656],
	[0x0032b, "i", "自由"],
	[0x0032d, 0x0032b],
	[0x0037c, "t", "自由"],
	[0x002ae, 0x0037c],
	[0x003d9, "h", "自由"],
	[0x003e0, 0xd89fc],
	[0x0020e, "e", "小集合自由"],
	[0x00145, "v", "小集合自由"],
	[0x001ae, 0x00145], // 足字底（00359）末两笔
	[0x00137, "d", "自由"],
	[0x00258, "o", "小集合自由"],
	[0x00259, 0x00258],
	[0x00385, "f", "自由"],
	[0x002da, "n", "自由"],
	[0x0027e, "c", "自由"],
	[0x002ca, "g", "自由"],
	[0x1c60c, "g", "自由"],
	[0x00493, "b", "自由"],
	[0x004ae, "b", "自由"],
	[0xa8200, "b", "自由"],
	[0x84705, "b", "自由"],
	[0x003a1, "p", "自由"],
	[0x1d217, "p", "自由"],
	[0x002a8, "m", "自由"],
	[0x0038e, "m", "自由"],
	[0x00451, "m", "自由"],
	[0x00479, "m", "自由"],
	[0x00572, "m", "自由"],
	[0x0054e, "f", "自由"],
	[0x1010f, "f", "自由"],
	[0x00188, "d", "自由"],
	[0x00124, 0x00188],
	[0x01544, "d", "自由"],
	[0x00464, "t", "自由"],
	[0x00386, "n", "自由"],
	[0x01391, "n", "自由"],
	[0x00409, "s", "自由"],
	[0x08324, "y", "自由"],
	[0x004b9, 0x08324],
	[0x00189, "l", "自由"],
	[0x00500, "l", "自由"],
	[0x08217, "l", "自由"],
	[0x00803, "g", "自由"],
	[0x4a413, "g", "自由"],
	[0x002a5, "g", "自由"],
	[0x00347, "g", "自由"],
	[0x00499, "g", "自由"],
	[0x002b5, "a", "小集合自由"],
	[0x00175, 0x002b5, "小集合自由"],
	[0x00130, "v", "小集合自由"],
	[0x00112, "o", "小集合自由"],
	[0x0017e, "v", "小集合自由"],
	[0x003e3, "h", "自由"],
	[0x0048d, "h", "自由"],
	[0x56727, "h", "自由"],
	[0x1930b, "h", "自由"],
	[0x00262, "j", "自由"],
	[0x02224, "j", "自由"],
	[0x1243f, "j", "自由"],
	[0x002f0, 0x34204],
	[0x0038f, "q", "自由"],
	[0x09139, "q", "自由"],
	[0x00339, "q", "自由"],
	[0x0028d, 0x00339],
	[0x003e9, "x", "自由"],
	[0x002af, 0x003e9],
	[0x28104, "x", "自由"],
	[0x002ce, "z", "自由"],
	[0x2032d, "z", "自由"],
	[0x1e355, "z", "自由"],
	[0x00721, "z", "自由"], // 隹按左右结构处理，不能进入 B。
	[0x1e353, 0x1e355],
	[0x00647, "z", "自由"],
	[0x0053c, "c", "自由"],
	[0x00340, "c", "自由"],
	[0x4d324, "c", "自由"],
	[0x00215, "c", "自由"],
	[0x20348, "c", "自由"],
	[0x00261, ".", "自由"],
	[0x002c2, "s", "自由"],
	[0x1e103, "s", "自由"],
	[0x25116, "s", "自由"],
	[0x1e263, "s", "自由"],
	[0x0063c, "s", "自由"],
	[0x86618, "s", "自由"],
	[0x8a204, "s", "自由"],
	[0x1011b, "s", "自由"],
	[0x9a100, "y", "自由"],
	[0x08325, 0x08324],
	[0x75311, "y", "自由"],
	[0x08405, "y", "自由"],
	[0x0341b, "y", "自由"],
	[0x0341c, 0x0341b],
	[0x0061c, "y", "自由"],
	[0x1d100, "y", "自由"], // 友；爱以友为初始正根。
	[0x00307, "w", "自由"],
	[0x016f1, "k", "自由"],
	[0x0442b, 0x04414],
	[0x00342, 0x00340],
	[0x00146, 0x00145],
	[0x0050b, "r", "自由"],
	[0x0070c, "k", "自由"],
	[0x00711, 0x0070c],
	[0x00366, 0x003e9],
	[0x0028f, 0x0017f],
	[0x0017f, 0x0017d, "自由"],
	[0x0017d, "r", "自由"],
	[0x34204, "r", "自由"],
	[0x0034f, "w", "自由"],
	[0x00436, "w", "自由"], // 戊
	[0x3c200, "x", "自由"],
	[0x0033d, "d", "自由"],
	[0x0016e, ";", "小集合自由"],
	[0x00170, 0x00143, "小集合自由"],
	[0x00144, "/", "小集合自由", 0x00105],
	[0x00153, "u", "小集合自由"], // 勹
	[0x00224, 0x00215, "自由"],
	[0x00105, ";", "小集合自由"],
	[0x00197, "/", "小集合自由"],
	[0x0024a, 0x00197], // 禹末三笔归并到厶
	[0x0010a, "c", "自由"],
	[0x00143, "/", "小集合自由"],
	[0x0f325, "k", "自由"],
	[0x00399, 0x00386],
	[0x00564, "z", "自由"],
	[0x00100, "i", "小集合自由"],
	[0x00198, ";", "小集合自由"],
	[0x00308, ";", "小集合自由"],
	[0x0012f, 0x00130, "小集合自由"],
	[0x00478, "l", "自由"],
	[0x0010b, 0x00105, "小集合自由"],
	[0x00133, 0x0010b], // 毋末两笔
	[0x0016d, "e", "小集合自由"], // 冫
	[0x00148, 0x0016d], // 飞末两笔归并到冫
	[0x0014a, 0x00148], // 永末两笔归并到飞末两笔
	[0x00151, "e", "小集合自由"],
	[0x0027f, 0x00200],
	[0x0044e, 0x0037c],
	[0x00405, 0x00307, "自由"],
	[0x00134, "a", "小集合自由"],
	[0x00156, 0x00112],
	[0x0011f, "a", "小集合自由"],
	[0x00227, "a", "小集合自由"],
	[0x00316, "w", "自由"],
	[0x00519, "i", "小集合自由"],
	[0x5160b, "l", "自由"],
	[0x002bb, "e", "小集合自由"],
	[0x00200, "u", "小集合自由"],
	[0x0038a, 0x00385],
	[0x0044f, 0x0044e],
	[0x0019b, 0x00197, "小集合自由"],
	[0x003d1, "w", "自由"],
	[0x0930e, "r", "自由"],
	[0x0021d, "c", "自由"],
	[0x2a009, "z", "自由", 0x003a9],
	[0x001a1, 0x00197],
	[0x00250, "u", "小集合自由"],
	[0x00254, 0x00250],
	[0x00178, "o", "小集合自由"],
	[0x00109, "a", "小集合自由"], // 丆
	[0x0026a, "i", "小集合自由"],
	[0x00291, 0x003d1, "自由"],
	[0x04208, 0x003d1, "自由"],
	[0x0020f, 0x0020e],
	[0x0020d, 0x00385],
	[0x0221f, "b", "自由"],
	[0x002b7, ",", "自由"],
	[0x002f1, "m", "自由"],
	[0x003e6, 0x00409],
	[0x002a2, 0x8a204],
	[0x00176, 0x08405],
	[0x0014c, 0x00130, "小集合自由"],
	[0x0022e, "y", "自由"],
	[0x0014d, 0x00151],
	[0x0014e, "u", "小集合自由"],
	[0x00448, "i", "小集合自由"],
	[0x002bc, 0x002bb],
	[0x0014f, "v", "小集合自由"],
	[0x00401, 0x00308, "小集合自由"],
	[0x0422b, "f", "自由"],
	[0x00201, 0x0020d],
	[0x0048e, 0x0048d],
	[0x0032c, 0x0032b],
	[0x00229, 0x00227],
	[0x1850e, 0x00340],
	[0x00287, 0x00284],
	[0x001a5, 0x0017f, "自由"],
	[0x0013f, "i", "小集合自由", 0x0010a], // ⺁
	[0x002b0, 0x00254],
	[0x00290, "x", "自由"],
	[0x00337, "b", "自由"],
	[0x00395, 0x00386],
	[0x0031c, 0x00307],
	[0x002c7, "s", "自由"],
	[0x00127, 0x00124],
	[0x00171, 0x00124],
	[0x002be, 0x002bb],
	[0x1b111, "j", "自由"],
	[0x1910a, 0x04208],
	[0x00355, "z", "自由"],
	[0x003a9, "z", "自由"],
	[0x0536a, "/", "小集合自由", 0x00143],
	[0x60118, 0x0341b],
	[0x2a210, "g", "自由"],
	[0x0031e, 0x7d104],
	[0x00622, "s", "自由"],
	[0x7331b, "x", "自由"],
	[0x00173, 0x00100],
	[0x00183, 0x0017f],
	[0x00139, "u", "小集合自由"],
	[0x00279, "u", "小集合自由"],
	[0x00125, 0x00100],
	[0x00251, 0x00250],
	[0x0027b, 0x00279],
	[0x0026c, 0x0026a],
	[0x0024d, 0x00200],
	[0x00104, 0x00105, "小集合自由"],
	[0x0013a, 0x00139],
	[0x0015b, 0x0014f, "小集合自由"],
	[0x51609, 0x5160b],
	[0x00533, "a", "小集合自由"],
	[0x1940a, "l", "自由"],
	[0x22300, "m", "自由"],
	[0x00646, "b", "自由"],
	[0x4a301, "q", "自由"],
	[0x1f500, "m", "自由"],
	[0x02607, "f", "自由"],
	[0x01933, "h", "自由"],
	[0x7d104, "l", "自由"],
	[0x00252, 0x00250],
	[0x001aa, 0x00100],
	[0x002f4, 0x00200],
	[0x002a9, "/", "小集合自由", 0x00170],
	[0x21507, "z", "自由"],
	[0x1282f, "x", "自由"],
	[0x004c2, "m", "自由"],
	[0x00246, "b", "自由"],
	[0x00115, 0x00154],
	[0x00154, "b", "自由"],
	[0x00247, 0x0020e],
	[0x20243, "z", "自由"],
	[0x04476, 0x0536a],
	[0x0019c, 0x001a1],
	[0x1e45f, "m", "自由"],
	[0x003dd, "d", "自由"],
	[0x01868, "g", "自由"],
	[0x02a1a, "z", "自由"],
	[0x0027d, 0x0013f],
	[0x0027c, 0x00130, "小集合自由"],
	[0x003a7, 0x00259],
	[0x002ef, 0x002bb],
	[0x00568, "e", "小集合自由"],
	[0x6610c, "n", "自由"],
	[0x004cd, "m", "自由"],
	[0x00552, 0x00622],
	[0x15306, "s", "自由"],
	[0x0055b, "x", "自由"],
	[0x0055c, 0x0055b],
	[0x1e3ed, "g", "自由"],
	[0x08421, "x", "自由"],
	[0x08422, 0x08421],
	[0x8420a, "s", "自由"],
	[0x0012c, "b", "自由"],
	[0x04414, "z", "自由"],
	[0x8e53e, 0x0053c],
	[0x12338, "s", "自由"],
	[0x41800, "g", "自由"],
	[0x73639, "s", "自由"],
	[0x6d678, "d", "自由"],
	[0x00147, 0x00145],
	[0x00210, "g", "自由"],
	[0xd7001, 0x003a1],
	[0x0041d, "g", "自由"],
	[0x1c400, "z", "自由"],
	[0x01f82, "y", "自由"],
	[0x00140, 0x00100],
	[0x00141, 0x00140],
	[0x00326, "a", "小集合自由"],
	[0x00325, 0x00326, "小集合自由"],
	[0x00174, 0x00173],
	[0x00407, 0x00308, "小集合自由"],
	[0x00507, 0x04656],
	[0x001a3, 0x002b7, "自由"],
	[0x46105, "l", "自由"],
	[0x003f6, "c", "自由"],
	[0x1a402, "l", "自由"],
	[0x2420f, "d", "自由"],
	[0x00435, "l", "自由"],
	[0x8d105, "h", "自由"],
	[0x001a6, 0x00100],
	[0x00445, 0x00258],
	[0x0047c, 0x00258],
	[0x00551, "e", "小集合自由"],
	[0x003d2, "i", "小集合自由"],
	[0x001ac, 0x00258],
	[0x002e8, 0x003d2],
	[0x001ad, 0x001ac],
	[0x003ac, 0x003d2],
	[0x003e1, 0x003d2],
	[0x19104, 0x0011f],
	[0x0035e, 0xd89fc],
	[0x0451f, 0x00551],
	[0x002b2, 0x00200],
	[0x00121, 0x0011f],
	[0xd89fc, "o", "小集合自由"],
	[0xd89fb, 0x003d2],
	[0x00284, 0x00146, "小集合自由"],
	[0x00122, 0x00105],
];

const 同名映射 = (前缀: string, 字母: string) =>
	Object.fromEntries([...字母].map((键) => [`${前缀}-${键}`, 键]));
const 映射: 决策 = {
	...Object.fromEntries([..."eiuoa"].map((键, 序号) => [序号 + 1, 键])),
	...同名映射("声", "bpmfdtnlgkhjqxzcsryw"),
	"声-0": "r",
	...同名映射("韵", "aoeiuv"),
	...同名映射("形", "abcdefghijklmnopqrstuvwxyz"),
};
const 空间: 决策空间 = {};
function 映射值(值: 字根安排) {
	if (typeof 值 === "number") return { element: 根名(值) };
	if (typeof 值 === "string" && 值.startsWith("声-")) return { element: 值 };
	return 值;
}
export const 保留小集合 = new Set<string>();
for (const [字形, 初始, ...备选] of 字根配置) {
	const 名称 = 根名(字形);
	映射[名称] = 映射值(初始);
	if (备选.includes("小集合自由") || 备选.includes("保留小集合"))
		保留小集合.add(名称);
	const 安排 = 备选.filter((目标) => 目标 !== "保留小集合");
	if (安排.length) {
		空间[名称] = 安排.flatMap((目标): 决策空间[string] => {
			if (typeof 目标 === "object" && !("element" in 目标))
				return 目标.map((value) => ({ value, score: 0 }));
			return [
				{
					value:
						目标 === "自由" || 目标 === "小集合自由"
							? [{ variable: 目标 === "自由" ? "全集合" : "小集合" }]
							: 映射值(目标),
					score: 0,
				},
			];
		});
	}
}

const 结构 = (运算符: 结构描述字符, ...引用: 模式[]): 模式 => ({
	operator: 运算符,
	references: 引用,
});
const 变量 = (编号: number, 字形集合?: number[]): 模式 =>
	字形集合 ? { variable: 编号, id_set: 字形集合 } : { variable: 编号 };

// 三分结构的同形两端合为外框；辛、手、小的附形与本形也视作相同。
function 两端包围(三分: "⿲" | "⿳", 两端?: number[]): 字形拼写运算 {
	const 前 = 变量(1, 两端);
	const 后 = 变量(两端 ? 3 : 1, 两端);
	return {
		from: 结构(三分, 前, 变量(2), 后),
		to: 结构(
			"⿴",
			结构(三分 === "⿲" ? "⿰" : "⿱", 变量(1), 变量(两端 ? 3 : 1)),
			变量(2),
		),
	};
}

function 包围字形(
	字形: number,
	内部: number,
	笔顺: 复合体数据["strokes"] = [
		{ index: 0, from: 0, to: 3 },
		{ index: 1 },
		{ index: 0, from: 4 },
	],
): 复合体数据 {
	return {
		id: 字形,
		type: "compound",
		operator: "⿴",
		references: [
			{ id: 0x0489b, xbegin: 0, xend: 100, ybegin: 0, yend: 100 },
			{ id: 内部, xbegin: 23, xend: 77, ybegin: 15, yend: 65 },
		],
		strokes: 笔顺,
	};
}

function 三分字形(
	字形: number,
	上: number,
	中: number,
	下: number,
	名称?: string,
): 复合体数据 {
	return {
		id: 字形,
		...(名称 ? { name: 名称 } : {}),
		type: "compound",
		operator: "⿳",
		references: [{ id: 上 }, { id: 中 }, { id: 下 }],
	};
}

// 参照思源黑体 Light 的比例，人冠两脚收在字高 39–46%，下部按各字展开。
function 人冠融合字形(
	字形: number,
	冠底: number,
	其余笔画: 矢量图形数据,
): 基本部件数据 {
	const 冠高 = 冠底 - 8;
	return {
		id: 字形,
		type: "component",
		operator: undefined,
		references: undefined,
		strokes: [
			{
				feature: "撇",
				start: [50, 8],
				curveList: [
					{
						command: "c",
						parameterList: [
							-12,
							Math.round(冠高 * 0.45),
							-28,
							Math.round(冠高 * 0.8),
							-45,
							冠高,
						],
					},
				],
			},
			{
				feature: "捺",
				start: [50, 8],
				curveList: [
					{
						command: "c",
						parameterList: [
							12,
							Math.round(冠高 * 0.45),
							28,
							Math.round(冠高 * 0.8),
							45,
							冠高,
						],
					},
				],
			},
			...其余笔画,
		],
	};
}

const 方案: 配置 = {
	version: "0.3",
	source: null,
	info: {
		name: "冰雪飞花",
		author: "谭淞宸",
		version: "0.4.0",
		description: "让二码顶再次伟大",
	},
	form: {
		alphabet: 补集合 + 小集合,
		mapping: 映射,
		mapping_variables: Object.fromEntries(
			Object.entries(键集).map(([名称, 键]) => [名称, { keys: [...键] }]),
		),
		mapping_space: 空间,
	},
	encoder: {
		max_length: 4,
		select_keys: ["_"],
		sources: {
			s0: { object: null, next: "s1" },
			s1: { object: { type: "字音", subtype: "声" }, next: "s2" },
			s2: { object: { type: "字根", rootIndex: 1 }, next: "s3" },
			s3: { object: { type: "字根", rootIndex: 2 }, next: "s4" },
			s4: { object: { type: "字根", rootIndex: 3 }, next: null },
		},
		conditions: {},
		rules: [
			{ length_equal: 2, formula: "AaAbBaBb" },
			{ length_equal: 3, formula: "AaBaCaCb" },
			{ length_in_range: [4, 10], formula: "AaBaCaDa" },
		],
	},
	algebra: {
		声: [
			{ type: "xform", from: "^([bpmfdtnlgkhjqxzcsr]|y|w|^).+$", to: "$1" },
			{ type: "xform", from: "^$", to: "0" },
		],
		韵: [
			{ type: "xform", from: "\\d", to: "" },
			{ type: "xform", from: "^([bpmfdtnlgkhjqxzcsryw])h?", to: "" },
			{ type: "xform", from: "^(.).+", to: "$1" },
		],
		形: [
			{ type: "xform", from: "^yi", to: "i" },
			{ type: "xform", from: "^wu", to: "u" },
			{ type: "xform", from: "^yu", to: "v" },
			{ type: "xform", from: "^(.).+", to: "$1" },
		],
	},
	data: {
		glyph_algebra: [
			{ from: 0xaae06, to: 结构("⿳", 0xaa817, 0x00146, 0x00366) },
			{ from: 0x38500, to: 结构("⿹", 0x38100, 0x0221f) },
			// 只拆当前层的亠口，不跨层拆散亮字头等完整字头。
			{
				from: 结构("⿱", 结构("⿱", 0x0016e, 0x00258), 变量(1)),
				to: 结构("⿳", 0x0016e, 0x00258, 变量(1)),
			},
			// 谷改为三分；子字头在递归加载时已由准包围转为 ⿵。
			{
				from: 结构("⿱", 结构("⿵", 0x00143, 0x00146), 0x00258),
				to: 结构("⿳", 0x00143, 0x00146, 0x00258),
			},
			{
				from: 结构("⿱", 结构("⿱", 0x002a6, 0x00258), 变量(1)),
				to: 结构("⿳", 0x002a6, 0x00258, 变量(1)),
			},
			{
				from: 结构("⿸", 0x35305, 变量(1)),
				to: 结构("⿸", 0x002a5, 结构("⿱", 0x00326, 变量(1))),
			},
			// 衔等先分出行框。
			{
				from: 结构("⿲", 0x0027e, 变量(1), 0x00203),
				to: 结构("⿴", 0x3c200, 变量(1)),
			},
			// 哀等先分出衣框。
			{
				from: 结构("⿳", 0x0016e, 变量(1), 0x003bd),
				to: 结构("⿴", 0x08324, 变量(1)),
			},
			// 旅、族等先分出旅字框。
			{
				from: 结构("⿰", 0x0810e, 结构("⿱", 0x00134, 变量(1))),
				to: 结构("⿸", 0x46105, 变量(1)),
			},
			// 放的右部是已融合的字根，保留左右分部。
			{ from: 0x46307, to: 结构("⿰", 0x0810e, 0x04208) },
			// 准包围：用 ID 绑定完整字头，保留其字根和融合身份。
			{
				from: 结构(
					"⿱",
					变量(
						1,
						[
							0x00146, 0x00143, 0x00227, 0x00175, 0x016c8, 0x017b7, 0x0536f,
							0x0832b, 0x0c10c, 0x0c10d, 0x0c119, 0x0c210, 0x0cf1c, 0x0e10a,
							0x0e200, 0x0e201, 0x0e300, 0x0e400, 0x0e500, 0x0e503, 0x0e507,
							0x0e518, 0x0e601, 0x0e60d, 0x0e704, 0x0e713, 0x0e80b, 0x0e913,
							0x0ea0e, 0x0ea0f, 0x0eb1b, 0x0ec0e, 0x0ec10, 0x0ee12, 0x12328,
							0x1c209, 0x20115, 0x21104, 0x2510c, 0x25116, 0x25209, 0x2521f,
							0x25701, 0x25829, 0x25832, 0x25919, 0x2591e, 0x25b03, 0x25b1d,
							0x25b1e, 0x25c1c, 0x28104, 0x28e06, 0x2a107, 0x2a10e, 0x2a206,
							0x2f404, 0x48116, 0x48214, 0x57101, 0x57106, 0x59f03, 0x60504,
							0x6a207, 0x6d20d, 0x7663b, 0x7674a, 0x7674b, 0x7a20a, 0x7a400,
							0x7a52c, 0x7b108, 0x7b20a, 0x86214, 0x8c106, 0x8c113, 0x8c118,
							0x8c220, 0x8c61c, 0x8c627, 0x8c6af, 0x92205, 0xd7f1e,
						],
					),
					变量(2),
				),
				to: 结构("⿵", 变量(1), 变量(2)),
			},
			...[
				undefined,
				[0x08422, 0x08421],
				[0x0038a, 0x00385],
				[0x00251, 0x00250],
			].flatMap((两端) =>
				(["⿲", "⿳"] as const).map((三分) => 两端包围(三分, 两端)),
			),
			// 莽、葬等：艹与廾作为约定同形的上下两端，取中间为正部。
			两端包围("⿳", [0x00215, 0x00224]),
		],
		glyphs: [
			// 兆：四笔率字腰包围儿，取码外先，绘图仍按原六笔笔顺。
			{
				id: 0x0445f,
				type: "compound",
				operator: "⿴",
				references: [
					{ id: 0x003d2, xbegin: -5, xend: 105, ybegin: 8, yend: 72 },
					{ id: 0x0014f, xbegin: 26, xend: 74, ybegin: -4, yend: 100 },
				],
				strokes: [
					{ index: 1, from: 0, to: 0 },
					{ index: 0, from: 0, to: 1 },
					{ index: 1, from: 1, to: 1 },
					{ index: 0, from: 2, to: 3 },
				],
			},
			// 六字按思源黑体 Light 调整比例；会、佘两横仍为不可拆平行单元。
			人冠融合字形(0x60116, 46, [
				{
					feature: "横",
					start: [20, 44],
					curveList: [{ command: "h", parameterList: [60] }],
				},
				{
					feature: "横",
					start: [20, 66],
					curveList: [{ command: "h", parameterList: [60] }],
				},
				{
					feature: "竖",
					start: [50, 44],
					curveList: [{ command: "v", parameterList: [46] }],
				},
				{
					feature: "横",
					start: [8, 90],
					curveList: [{ command: "h", parameterList: [84] }],
				},
			]), // 全
			人冠融合字形(0x0445b, 42, [
				{
					feature: "横",
					start: [26, 38],
					curveList: [{ command: "h", parameterList: [48] }],
				},
				{
					feature: "横",
					start: [9, 58],
					curveList: [{ command: "h", parameterList: [82] }],
				},
				{
					feature: "撇折",
					start: [43, 58],
					curveList: [
						{ command: "c", parameterList: [-7, 10, -18, 22, -27, 32] },
						{ command: "c", parameterList: [21, -2, 43, -4, 64, -6] },
					],
				},
				{
					feature: "点",
					start: [64, 68],
					curveList: [{ command: "c", parameterList: [9, 8, 17, 17, 23, 26] }],
				},
			]), // 会
			人冠融合字形(0x04543, 42, [
				{
					feature: "横",
					start: [24, 38],
					curveList: [{ command: "h", parameterList: [52] }],
				},
				{
					feature: "横",
					start: [9, 57],
					curveList: [{ command: "h", parameterList: [82] }],
				},
				{
					feature: "竖钩",
					start: [50, 38],
					curveList: [{ command: "v", parameterList: [54] }],
				},
				{
					feature: "撇",
					start: [31, 68],
					curveList: [
						{ command: "c", parameterList: [-6, 9, -14, 17, -22, 22] },
					],
				},
				{
					feature: "点",
					start: [68, 68],
					curveList: [{ command: "c", parameterList: [10, 7, 18, 15, 23, 22] }],
				},
			]), // 余
			人冠融合字形(0x71108, 44, [
				{
					feature: "横",
					start: [28, 39],
					curveList: [{ command: "h", parameterList: [44] }],
				},
				{
					feature: "横",
					start: [11, 56],
					curveList: [{ command: "h", parameterList: [78] }],
				},
				{
					feature: "竖钩",
					start: [50, 56],
					curveList: [{ command: "v", parameterList: [36] }],
				},
				{
					feature: "撇",
					start: [30, 66],
					curveList: [
						{ command: "c", parameterList: [-5, 8, -12, 16, -19, 22] },
					],
				},
				{
					feature: "点",
					start: [68, 66],
					curveList: [{ command: "c", parameterList: [9, 8, 16, 16, 20, 22] }],
				},
			]), // 佘
			人冠融合字形(0x04656, 40, [
				{
					feature: "横",
					start: [25, 38],
					curveList: [{ command: "h", parameterList: [50] }],
				},
				{
					feature: "横",
					start: [11, 58],
					curveList: [{ command: "h", parameterList: [78] }],
				},
				{
					feature: "竖",
					start: [50, 38],
					curveList: [{ command: "v", parameterList: [52] }],
				},
				{
					feature: "点",
					start: [23, 65],
					curveList: [{ command: "c", parameterList: [4, 8, 8, 18, 10, 25] }],
				},
				{
					feature: "撇",
					start: [77, 65],
					curveList: [
						{ command: "c", parameterList: [-3, 9, -8, 20, -12, 25] },
					],
				},
				{
					feature: "横",
					start: [7, 90],
					curveList: [{ command: "h", parameterList: [86] }],
				},
			]), // 金
			人冠融合字形(0x87106, 39, [
				{
					feature: "横",
					start: [26, 33],
					curveList: [{ command: "h", parameterList: [48] }],
				},
				{
					feature: "横",
					start: [10, 49],
					curveList: [{ command: "h", parameterList: [80] }],
				},
				{
					feature: "竖",
					start: [50, 33],
					curveList: [{ command: "v", parameterList: [33] }],
				},
				{
					feature: "竖",
					start: [22, 66],
					curveList: [{ command: "v", parameterList: [24] }],
				},
				{
					feature: "横折",
					start: [22, 66],
					curveList: [
						{ command: "h", parameterList: [56] },
						{ command: "v", parameterList: [24] },
					],
				},
				{
					feature: "横",
					start: [22, 90],
					curveList: [{ command: "h", parameterList: [56] }],
				},
			]), // 舍
			{
				id: 0x004ab, // 刍：负字头、彐
				type: "compound",
				operator: "⿱",
				references: [{ id: 0x0014e }, { id: 0x002bb }],
			},
			// 仅展开所列字形；亘、龺另按同端取中显式保留笔顺。
			三分字形(0x016f1, 0x0014e, 0x00464, 0x00000), // 鱼：负字头、田、一
			三分字形(0x21507, 0x0020f, 0x00258, 0x002a9), // 壴：士、口、䒑
			三分字形(0x19507, 0x0011f, 0x0036a, 0x00105), // 卓：卜、日、十
			三分字形(0x19707, 0x0011f, 0x0036a, 0x0032b), // 桌：卜、日、木
			三分字形(0x01933, 0x00325, 0x00465, 0x00142), // 黄：龷、由、八
			三分字形(0x66304, 0x00000, 0x00258, 0x00464), // 畐：一、口、田
			三分字形(0x8cd85, 0x00215, 0x1e22a, 0x00721), // 雚：艹、双口、隹
			三分字形(0x0e704, 0x00215, 0x00478, 0x00175, "瞢字头"), // 艹、罒、冖
			三分字形(0xd832f, 0x00215, 0x00721, 0x00198), // 蒦：艹、隹、又
			三分字形(0x2871d, 0x002b5, 0x00512, 0x00142), // 寅：宀、寅中、八
			三分字形(0x2fb05, 0x002f4, 0x1f104, 0x05576), // 巤：巛、龱、鼠字底
			三分字形(0x57c0f, 0x2a009, 0x00478, 0x2940a), // 爵：爪头、罒、爵字底
			三分字形(0x3d42f, 0x00170, 0x00258, 0x003e9), // 总：丷、口、心
			三分字形(0x0e500, 0x0020f, 0x00258, 0x00175, "薹字腰"), // 士、口、冖
			三分字形(0x1e75e, 0x00258, 0x00105, 0x1f20e), // 啚：口、十、回
			三分字形(0x28878, 0x002b5, 0x003e9, 0x00459), // 甯：宀、心、甬字底
			// 亘：一、日、一；同端合外部，笔顺仍为上端、内部、下端。
			{
				id: 0xffffd,
				name: "亘两端",
				type: "compound",
				operator: "⿱",
				references: [
					{ id: 0x00000, ybegin: -45, yend: 55 },
					{ id: 0x00000, ybegin: 45, yend: 145 },
				],
			},
			{
				id: 0x0143e,
				type: "compound",
				operator: "⿴",
				references: [
					{ id: 0xffffd, xbegin: 0, xend: 100, ybegin: 0, yend: 100 },
					{ id: 0x0036a, xbegin: 0, xend: 100, ybegin: 4, yend: 97 },
				],
				strokes: [
					{ index: 0, from: 0, to: 0 },
					{ index: 1 },
					{ index: 0, from: 1 },
				],
			},
			// 龺：十、日、十；两个十共享外部，但不提前书写下面的十。
			{
				id: 0xffffc,
				name: "龺两端",
				type: "compound",
				operator: "⿱",
				references: [
					{ id: 0x00105, ybegin: 9, yend: 39 },
					{ id: 0x00105, ybegin: 62, yend: 96 },
				],
			},
			{
				id: 0x1850e,
				type: "compound",
				operator: "⿴",
				references: [
					{ id: 0xffffc, xbegin: 0, xend: 100, ybegin: 0, yend: 100 },
					{ id: 0x0036a, xbegin: 0, xend: 100, ybegin: 33, yend: 69 },
				],
				strokes: [
					{ index: 0, from: 0, to: 1 },
					{ index: 1 },
					{ index: 0, from: 2 },
				],
			},
			// 璺和爨共用的字头：仍先写臼裂左四笔，再写内件和右三笔。
			{
				id: 0x0da10,
				name: "璺字头无冖",
				type: "compound",
				operator: "⿴",
				references: [
					{ id: 0x0451f },
					{ id: 0x0d30d, xbegin: 21, xend: 79, ybegin: 3, yend: 92 },
				],
				strokes: [{ index: 0, to: 3 }, { index: 1 }, { index: 0, from: 4 }],
			},
			// 爨的上部整体作为准包围外部，火为内部。
			{
				id: 0xffffe,
				name: "爨字上部",
				type: "compound",
				operator: "⿳",
				references: [
					{ id: 0x0ec0e, ybegin: 0, yend: 39 },
					{ id: 0x4b315, ybegin: 40, yend: 67 },
					{ id: 0x00227, ybegin: 67, yend: 100 },
				],
			},
			{
				id: 0x0ef4d,
				type: "compound",
				operator: "⿵",
				references: [
					{ id: 0xffffe, xbegin: 0, xend: 100, ybegin: 0, yend: 100 },
					{ id: 0x003d9, xbegin: 25, xend: 75, ybegin: 80, yend: 97 },
				],
			},
			// 寡字底：頁变包刀；先写外部九笔（寡字腰、八），再写刀。
			{
				id: 0x0c808,
				name: "寡字底",
				type: "compound",
				operator: "⿵",
				references: [{ id: 0x0c603 }, { id: 0x00188 }],
			},
			三分字形(0x6653f, 0x00170, 0x00464, 0x0121e), // 兽：丷、田、𠮛
			// 率无亠无十：四笔率字腰围住幺，仍先写幺。
			{
				id: 0x0f427,
				name: "率无亠无十",
				type: "compound",
				operator: "⿴",
				references: [
					{ id: 0x003d2, xbegin: 0, xend: 100, ybegin: 0, yend: 100 },
					{ id: 0x002f3, xbegin: 33, xend: 66, ybegin: 0, yend: 100 },
				],
				strokes: [{ index: 1 }, { index: 0 }],
			},
			// 隹与隺共用字库已有的六笔「隹字边」（隹右）。
			{
				id: 0x00721,
				type: "compound",
				operator: "⿰",
				references: [
					{ id: 0x00137, xbegin: -16, xend: 61, ybegin: 0, yend: 100 },
					{ id: 0x0056a, xbegin: 13, xend: 106, ybegin: 7, yend: 97 },
				],
			},
			{
				id: 0x00909,
				type: "compound",
				operator: "⿸",
				references: [
					{ id: 0xfffff, xbegin: 0, xend: 100, ybegin: 0, yend: 100 },
					{ id: 0x0056a, xbegin: 13, xend: 106, ybegin: 24, yend: 98 },
				],
			},
			// 隺外框保留原字形前四笔：冖与亻，不丢失包围部分的两笔亻。
			{
				id: 0xfffff,
				name: "隺外框",
				type: "component",
				operator: undefined,
				references: undefined,
				strokes: [
					{
						feature: "点",
						start: [13, 19],
						curveList: [{ command: "c", parameterList: [0, 5, 0, 10, 0, 16] }],
					},
					{
						feature: "横钩",
						start: [13, 19],
						curveList: [{ command: "h", parameterList: [74] }],
					},
					{
						feature: "撇",
						start: [44, 6],
						curveList: [
							{ command: "c", parameterList: [-7, 15, -27, 44, -39, 55] },
						],
					},
					{
						feature: "竖",
						start: [22, 40],
						curveList: [{ command: "v", parameterList: [56] }],
					},
				],
			},
			// 章先分成音和十；竟在字库中已经以音为上部。
			{
				id: 0x7550c,
				type: "compound",
				operator: "⿱",
				references: [{ id: 0x75311 }, { id: 0x00105 }],
			},
			包围字形(0x01c6e, 0x00340),
			包围字形(0x01b88, 0x0023a),
			包围字形(0x01e45, 0x0d30d, [
				{ index: 0, from: 0, to: 3 },
				{ index: 0, from: 7, to: 7 },
				{ index: 1 },
				{ index: 0, from: 4, to: 6 },
				{ index: 0, from: 8 },
			]),
			包围字形(0x01f6c, 0x0060e),
		],
		fusion: [
			0x60116,
			0x0445b,
			0x04543,
			0x71108,
			0x04656,
			0x87106, // 全、会、余、佘、金（⾦）、舍
			0x03525, // 羌
			0x01142,
			0x0536a,
			0x04476,
			0x19104,
			0x0451f,
			0xa4106,
			0x05706, // 甚（G、V）
			0x0a601, // 甚（H、T、J、K、N）
			0x0810c,
			0x0341c,
			0x2a009,
			0x0422b,
			0x04202,
			0x19201,
			0x1852d,
			0x0831f,
			0x08313,
			0x08329, // 产
			0x60118,
			0x0341d,
			0x08422,
			0x03522,
			0x0341f,
			0x1e103,
			0x04208,
			0x0f005,
			0x08217,
			0x0810e,
			0x1b111,
			0x18207,
			0x3310c,
			0x25116,
			0x08421,
			0x0341b,
			0x1850c,
			0x0832b,
			0x18111,
			0x1e263,
			0x2d203,
			0x0489b,
			0x1840b,
			0x1c108,
			0x08114, // 充字头
			0x2011e,
			0x1e125,
			0x2a210,
			0x5d104,
			0x1c400,
			0x1c405,
			0x1d217,
			0x0163a, // 其
			0x1870e,
			0x08107,
			0x1d100,
			0x04521,
			0x0142f,
			0x1d203,
			0x02622,
			0x0262f,
			0x0c221,
			0x6a10b,
			0x1e11f,
			0x1d201,
			0x1b117,
			0x86116,
			0x1d218,
			0x0343a,
			0x1011c,
			0x03321,
			0x03420,
			0x01391,
			0x04432,
			0x03410,
			0x1910a,
			0x03626,
		],
		character_set: "minimal",
		glyph_sources: ["G"],
	},
	optimization: {
		objective: { characters_full: { duplication: 1 } },
		metaheuristic: {
			algorithm: "SimulatedAnnealing",
			update_interval: 10000,
			parameters: { t_max: 0.8, t_min: 0.003, steps: 10_000_000 },
		},
	},
};

const 声母例外 = new Map([
	[0xa8200, "声-b"], // 髟
	[0x21507, "声-z"], // 壴
	[0x46105, "声-y"], // 旅字框
]);

export function* 生成声母归并(
	字库: 字库,
	词典: 原始词典,
	可声母归并: ReadonlySet<string>,
): Iterable<[string, { element: string }[]]> {
	const 声规则 = 合并拼写运算(方案.algebra).get("声");
	if (!声规则) throw Error("缺少声母拼写运算");

	const 通用字形 = new Map<string, string[]>();
	const 通用字音 = new Map<string, Set<string>>();
	for (const [, { character, final_glyphs, tygf }] of 字库) {
		if (tygf === undefined) continue;
		const 名称 = final_glyphs
			.map(({ id }) => 根名(id))
			.filter((根) => 可声母归并.has(根));
		if (!名称.length) continue;
		通用字形.set(character.获取名称(), 名称);
		for (const 根 of 名称) 通用字音.set(根, new Set());
	}
	for (const { 词, 拼音 } of 词典) {
		const 名称 = 通用字形.get(词);
		if (!名称 || 拼音.length !== 1) continue;
		const 音 = 拼音[0].trim();
		if (!音) continue;
		for (const 根 of 名称) 通用字音.get(根)?.add(音);
	}

	const 路径 = new URL("../../../hanzi-chai/data/gf0014.txt", import.meta.url);
	const GF读音 = readFileSync(路径, "utf8")
		.replace(/\r?\n$/, "")
		.split(/\r?\n/);
	for (const [id] of 字根配置) {
		const 名称 = 根名(id);
		if (!可声母归并.has(名称)) continue;
		let 读音 = 通用字音.get(名称);
		const GF编号 = GF0014映射.get(id);
		if (!读音 && GF编号 !== undefined) {
			const 行 = GF读音[GF编号 - 1];
			const 两列 = 行?.split("\t");
			if (两列?.length !== 2)
				throw Error(`GF0014 第 ${GF编号} 行不是两列：${路径.pathname}`);
			读音 = new Set(
				两列.flatMap((列) => 列.trim().split(/\s+/)).filter(Boolean),
			);
		}

		const 锚点 = new Set<string>();
		for (const 音 of 读音 ?? []) {
			const element = `声-${应用拼写运算(声规则, 音)}`;
			const 键 = 映射[element];
			if (typeof 键 !== "string" || 键.length !== 1 || !补集合.includes(键))
				throw Error(`字根 ${名称} 的读音 ${音} 没有补集合声母锚点`);
			锚点.add(element);
		}
		const 例外 = 声母例外.get(id);
		if (例外) 锚点.add(例外);
		if (锚点.size)
			yield [名称, [...锚点].sort().map((element) => ({ element }))];
	}
}

export default 方案;
