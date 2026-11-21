import type {
	决策,
	决策空间,
	基本部件数据,
	复合体数据,
	字形拼写运算,
	模式,
	矢量图形数据,
	结构描述字符,
	配置,
} from "hanzi-chai";

const 小集合 = "eiuoav;/";
const 补集合 = "bpmfdtnlgkhjqxzcsryw,.";
const 键集 = { 小集合, 全集合: 小集合 + 补集合, 补集合 };
const 根名 = (字形: number) => `字形-${字形.toString(16).padStart(5, "0")}`;

// 每行依次为字形 ID、初始安排及决策空间；数字安排表示归并到对应字形。
// 空间可填键集名、声母元素名或归并目标的字形 ID，省略时仅保留初始安排。
const 字根配置: [
	字形: number,
	初始: string | number,
	...空间: (keyof typeof 键集 | `声-${string}` | number)[],
][] = [
	[0x2a41f, "j", "补集合", "声-j"],
	[0x48000, "d", "补集合", "声-d"],
	[0x0036a, ";", "小集合"], // 日
	[0x0036b, 0x0036a], // 曰
	[0x00371, 0x0036a], // 冒字头
	[0x003b2, "y", "全集合", "声-y"],
	[0x00368, 0x003b2, "全集合", "声-q"],
	[0x04656, "j", "补集合", "声-j"],
	[0x00486, 0x04656],
	[0x0032b, "i", "全集合", "声-m"],
	[0x0032d, 0x0032b],
	[0x0037c, "t", "全集合", "声-s"],
	[0x002ae, 0x0037c],
	[0x003d9, "h", "全集合", "声-h"],
	[0x003e0, 0xd89fc],
	[0x0020e, "e", "小集合"],
	[0x00145, "v", "小集合"],
	[0x00137, "d", "全集合", "声-d"],
	[0x00258, "o", "小集合"],
	[0x00259, 0x00258],
	[0x00385, "f", "全集合", "声-s"],
	[0x002da, "n", "全集合", "声-n"],
	[0x0027e, "c", "全集合", "声-s", "声-r"],
	[0x002ca, "g", "全集合", "声-g"],
	[0x1c60c, "g", "补集合", "声-g"],
	[0x00493, "b", "全集合", "声-b"],
	[0x004ae, "b", "全集合", "声-b"],
	[0xa8200, "b", "补集合", "声-b"],
	[0x84705, "b", "补集合", "声-b"],
	[0x003a1, "p", "全集合", "声-p"],
	[0x1d217, "p", "全集合", "声-p"],
	[0x002a8, "m", "全集合", "声-m"],
	[0x0038e, "m", "全集合", "声-m"],
	[0x00451, "m", "全集合", "声-m"],
	[0x00479, "m", "全集合", "声-m"],
	[0x00572, "m", "全集合", "声-m"],
	[0x0054e, "f", "全集合", "声-f"],
	[0x1010f, "f", "补集合", "声-f"],
	[0x00188, "d", "全集合", "声-d"],
	[0x00124, 0x00188],
	[0x01544, "d", "补集合", "声-d"],
	[0x00464, "t", "全集合", "声-t"],
	[0x00386, "n", "全集合", "声-n"],
	[0x01391, "n", "全集合", "声-n"],
	[0x00409, "s", "全集合", "声-s"],
	[0x08324, "y", "补集合", "声-y"],
	[0x004b9, 0x08324],
	[0x00189, "l", "全集合", "声-l"],
	[0x00500, "l", "全集合", "声-l"],
	[0x08217, "l", "全集合", "声-l"],
	[0x00803, "g", "补集合", "声-g"],
	[0x4a413, "g", "补集合", "声-g"],
	[0x002a5, "g", "全集合", "声-g"],
	[0x00347, "g", "全集合", "声-g"],
	[0x00499, "g", "全集合", "声-g"],
	[0x002b5, "a", "小集合"],
	[0x00175, 0x002b5, "小集合"],
	[0x00130, "v", "小集合"],
	[0x00112, "o", "小集合"],
	[0x0017e, "v", "小集合"],
	[0x003e3, "h", "全集合", "声-h"],
	[0x0048d, "h", "全集合", "声-h"],
	[0x56727, "h", "补集合", "声-h"],
	[0x1930b, "h", "补集合", "声-h"],
	[0x00262, "j", "全集合", "声-j"],
	[0x02224, "j", "补集合", "声-j"],
	[0x1243f, "j", "补集合", "声-j"],
	[0x002f0, 0x34204],
	[0x0038f, "q", "全集合", "声-q"],
	[0x09139, "q", "补集合", "声-q"],
	[0x00339, "q", "全集合", "声-q"],
	[0x0028d, 0x00339],
	[0x003e9, "x", "全集合", "声-x"],
	[0x002af, 0x003e9],
	[0x28104, "x", "补集合", "声-x"],
	[0x002ce, "z", "全集合", "声-z"],
	[0x2032d, "z", "补集合", "声-z"],
	[0x1e355, "z", "补集合", "声-z"],
	[0x00721, "z", "补集合", "声-z"], // 隹按左右结构处理，不能进入 B。
	[0x1e353, 0x1e355],
	[0x00647, "z", "全集合", "声-z"],
	[0x0053c, "c", "补集合", "声-c"],
	[0x00340, "c", "补集合", "声-c"],
	[0x4d324, "c", "补集合", "声-c"],
	[0x00215, "c", "全集合", "声-c"],
	[0x20348, "c", "补集合", "声-c"],
	[0x00261, ".", "全集合", "声-s"],
	[0x002c2, "s", "全集合", "声-s", "声-p"],
	[0x1e103, "s", "全集合", "声-s", "声-d"],
	[0x25116, "s", "全集合", "声-s"],
	[0x1e263, "s", "全集合", "声-s"],
	[0x0063c, "s", "补集合", "声-s"],
	[0x86618, "s", "补集合", "声-s"],
	[0x8a204, "s", "补集合", "声-s"],
	[0x1011b, "s", "补集合", "声-s"],
	[0x9a100, "y", "补集合", "声-y"],
	[0x08325, 0x08324],
	[0x75311, "y", "补集合", "声-y"],
	[0x08405, "y", "补集合", "声-y"],
	[0x0341b, "y", "全集合", "声-y"],
	[0x0341c, 0x0341b],
	[0x0061c, "y", "全集合", "声-y"],
	[0x1d100, "y", "全集合", "声-y"], // 友；爱以友为初始正根。
	[0x00307, "w", "全集合", "声-w"],
	[0x016f1, "k", "补集合", "声-y"],
	[0x0442b, 0x04414],
	[0x00342, 0x00340],
	[0x00146, 0x00145],
	[0x0050b, "r", "全集合", "声-0"],
	[0x0070c, "k", "全集合", "声-y"],
	[0x00711, 0x0070c],
	[0x00366, 0x003e9],
	[0x0028f, 0x0017f],
	[0x0017f, 0x0017d, "全集合", "声-j", "声-0", "声-d"],
	[0x0017d, "r", "全集合", "声-0"],
	[0x34204, "r", "补集合", "声-j", "声-s"],
	[0x0034f, "w", "全集合", "声-w"],
	[0x00436, "w", "全集合", "声-w"], // 戊
	[0x3c200, "x", "补集合", "声-x"],
	[0x0033d, "d", "全集合", "声-d"],
	[0x0016e, ";", "小集合"],
	[0x00170, 0x00143, "小集合"],
	[0x00144, "/", "小集合", 0x00105],
	[0x00153, "j", "全集合", "声-j"],
	[0x00224, 0x00215, "全集合", "声-n"],
	[0x00105, ";", "小集合"],
	[0x00197, "/", "小集合"],
	[0x0010a, "c", "全集合", "声-c"],
	[0x00143, "/", "小集合"],
	[0x0f325, "k", "补集合", "声-y"],
	[0x00399, 0x00386],
	[0x00564, "z", "全集合", "声-z"],
	[0x00100, "i", "小集合"],
	[0x00198, ";", "小集合"],
	[0x00308, ";", "小集合"],
	[0x0012f, 0x00130, "小集合"],
	[0x00478, "l", "全集合", "声-l"],
	[0x0010b, 0x00105, "小集合"],
	[0x0016d, "d", "全集合", "声-b", "声-d", "声-l"],
	[0x00151, "e", "小集合"],
	[0x0027f, 0x00200],
	[0x0044e, 0x0037c],
	[0x00405, 0x00307, "全集合", "声-y"],
	[0x00134, "a", "小集合"],
	[0x00152, 0x0014e],
	[0x00156, 0x00112],
	[0x0011f, "a", "小集合"],
	[0x00227, "a", "小集合"],
	[0x00316, "w", "补集合", "声-w"],
	[0x00519, "i", "小集合"],
	[0x5160b, "l", "补集合", "声-l"],
	[0x002bb, "e", "小集合"],
	[0x00200, "u", "小集合"],
	[0x0038a, 0x00385],
	[0x0044f, 0x0044e],
	[0x0019b, 0x00197, "小集合"],
	[0x003d1, "w", "全集合", "声-w"],
	[0x0930e, "r", "补集合", "声-r"],
	[0x0021d, "c", "全集合", "声-c"],
	[0x2a009, "z", "全集合", "声-z", 0x003a9],
	[0x001a1, 0x00197],
	[0x00250, "u", "小集合"],
	[0x00254, 0x00250],
	[0x00178, "o", "小集合"],
	[0x0010e, 0x00109],
	[0x00109, "a", "全集合", "声-x"],
	[0x0026a, "i", "小集合"],
	[0x00291, 0x003d1, "全集合", "声-d"],
	[0x04208, 0x003d1, "全集合", "声-f", "声-w", "声-p"],
	[0x0020f, 0x0020e],
	[0x0020d, 0x00385],
	[0x0221f, "b", "补集合", "声-b"],
	[0x002b7, ",", "全集合", "声-z"],
	[0x002f1, "m", "全集合", "声-m"],
	[0x003e6, 0x00409],
	[0x002a2, 0x8a204],
	[0x00176, 0x08405],
	[0x0014c, 0x00130, "小集合"],
	[0x0022e, "y", "全集合", "声-y"],
	[0x0014d, 0x00151],
	[0x00155, 0x0014e],
	[0x0014e, "u", "小集合"],
	[0x00448, "i", "小集合"],
	[0x002bc, 0x002bb],
	[0x0014f, "v", "小集合"],
	[0x00401, 0x00308, "小集合"],
	[0x0422b, "f", "补集合", "声-f"],
	[0x00201, 0x0020d],
	[0x0048e, 0x0048d],
	[0x0032c, 0x0032b],
	[0x00229, 0x00227],
	[0x1850e, 0x00340],
	[0x00287, 0x00284],
	[0x001a5, 0x0017f, "全集合", "声-c"],
	[0x0013f, 0x0010a, "全集合", "声-f"],
	[0x002b0, 0x00254],
	[0x00290, "x", "全集合", "声-x"],
	[0x00337, "b", "全集合", "声-b"],
	[0x00395, 0x00386],
	[0x0031c, 0x00307],
	[0x002c7, "s", "全集合", "声-s"],
	[0x00127, 0x00124],
	[0x00171, 0x00124],
	[0x002be, 0x002bb],
	[0x1b111, "j", "全集合", "声-j"],
	[0x1910a, 0x04208],
	[0x00355, "z", "全集合", "声-z"],
	[0x003a9, "z", "全集合", "声-z"],
	[0x0536a, "/", "小集合", 0x00143],
	[0x60118, 0x0341b],
	[0x2a210, "g", "补集合", "声-g"],
	[0x0031e, 0x7d104],
	[0x00622, "s", "全集合", "声-s"],
	[0x7331b, "x", "补集合", "声-x"],
	[0x00173, 0x00100],
	[0x00183, 0x0017f],
	[0x00139, "u", "小集合"],
	[0x00279, "u", "小集合"],
	[0x00125, 0x00100],
	[0x00251, 0x00250],
	[0x0027b, 0x00279],
	[0x0026c, 0x0026a],
	[0x0024d, 0x00200],
	[0x00104, 0x00105, "小集合"],
	[0x0013a, 0x00139],
	[0x0015b, 0x0014f, "小集合"],
	[0x51609, 0x5160b],
	[0x00533, "a", "小集合"],
	[0x1940a, "l", "补集合", "声-l"],
	[0x22300, "m", "补集合", "声-m"],
	[0x00646, "b", "全集合", "声-b"],
	[0x4a301, "q", "补集合", "声-q"],
	[0x1f500, "m", "补集合", "声-m"],
	[0x02607, "f", "补集合", "声-f"],
	[0x01933, "h", "补集合", "声-h"],
	[0x7d104, "l", "补集合", "声-l"],
	[0x00252, 0x00250],
	[0x001aa, 0x00100],
	[0x002f4, 0x00200],
	[0x002a9, "/", "小集合", 0x00170],
	[0x21507, "z", "补集合", "声-z"],
	[0x1282f, "x", "补集合", "声-x"],
	[0x004c2, "m", "全集合", "声-m"],
	[0x00246, "b", "全集合", "声-b"],
	[0x00115, 0x00154],
	[0x00154, "b", "全集合", "声-b"],
	[0x00247, 0x0020e],
	[0x20243, "z", "补集合", "声-z"],
	[0x04476, 0x0536a],
	[0x0019c, 0x001a1],
	[0x1e45f, "m", "补集合", "声-m"],
	[0x003dd, "d", "补集合", "声-d"],
	[0x01868, "g", "补集合", "声-l", "声-g"],
	[0x02a1a, "z", "补集合", "声-z"],
	[0x0027d, 0x0013f],
	[0x0027c, 0x00130, "小集合"],
	[0x003a7, 0x00259],
	[0x002ef, 0x002bb],
	[0x00568, "e", "小集合"],
	[0x6610c, "n", "补集合", "声-n"],
	[0x004cd, "m", "全集合", "声-m"],
	[0x00552, 0x00622],
	[0x15306, "s", "补集合", "声-s"],
	[0x0055b, "x", "补集合", "声-x"],
	[0x0055c, 0x0055b],
	[0x1e3ed, "g", "补集合", "声-g"],
	[0x08421, "x", "全集合", "声-x"],
	[0x08422, 0x08421],
	[0x8420a, "s", "补集合", "声-s"],
	[0x0012c, "b", "全集合", "声-b"],
	[0x04414, "z", "补集合", "声-z"],
	[0x8e53e, 0x0053c],
	[0x12338, "s", "补集合", "声-s"],
	[0x41800, "g", "补集合", "声-g"],
	[0x73639, "s", "补集合", "声-s"],
	[0x6d678, "d", "补集合", "声-d"],
	[0x00147, 0x00145],
	[0x00210, "g", "全集合", "声-g"],
	[0xd7001, 0x003a1],
	[0x0041d, "g", "补集合", "声-g"],
	[0x1c400, "z", "声-s", "全集合"],
	[0x01f82, "y", "补集合", "声-y"],
	[0x00140, 0x00100],
	[0x00141, 0x00140],
	[0x00326, "a", "小集合"],
	[0x00325, 0x00326, "小集合"],
	[0x00174, 0x00173],
	[0x00407, 0x00308, "小集合"],
	[0x00507, 0x04656],
	[0x00132, 0x0012f, "小集合"],
	[0x2032e, 0x2032d],
	[0x002c4, 0x002c2],
	[0x00392, 0x0038e],
	[0x00349, 0x00347],
	[0x00393, 0x0038f],
	[0x003e4, 0x003e3],
	[0x0022f, 0x0022e],
	[0x001a3, 0x002b7, "全集合", "声-j"],
	[0x00350, 0x0034f],
	[0x003aa, 0x003a9],
	[0x0049a, 0x00499],
	[0x02429, 0x2a210],
	[0x0029e, 0x00291],
	[0x00264, 0x00261],
	[0x0033e, 0x0033d],
	[0x0038d, 0x0038a],
	[0x00380, 0x02224],
	[0x003d3, 0x003d1],
	[0x00625, 0x00622],
	[0x5160c, 0x5160b],
	[0x46105, "l", "补集合", "声-l", "声-y"],
	[0x1c60d, 0x1c60c],
	[0x0423f, 0x1010f],
	[0x01509, 0x22300],
	[0x86619, 0x86618],
	[0x5160a, 0x51609],
	[0x003f6, "c", "全集合", "声-c"],
	[0x003fa, 0x003f6],
	[0x1a402, "l", "补集合", "声-l"],
	[0x2420f, "d", "补集合", "声-d"],
	[0x00435, "l", "全集合", "声-l"],
	[0x8d105, "h", "补集合", "声-h"],
	[0x02614, 0x8d105],
	[0x001a6, 0x00100],
	[0x00445, 0x00258],
	[0x0047c, 0x00258],
	[0x00551, "e", "小集合"],
	[0x003d2, "i", "小集合"],
	[0x001ac, 0x00258],
	[0x002e8, 0x003d2],
	[0x001ad, 0x001ac],
	[0x003ac, 0x003d2],
	[0x003e1, 0x003d2],
	[0x19104, 0x0011f],
	[0x0035e, 0xd89fc],
	[0x0451f, 0x00551],
	[0x00150, 0x0014f, "小集合"],
	[0x002b2, 0x00200],
	[0x00121, 0x0011f],
	[0xd89fc, "o", "小集合"],
	[0xd89fb, 0x003d2],
	[0x00284, 0x00146, "小集合"],
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
for (const [字形, 初始, ...备选] of 字根配置) {
	const 名称 = 根名(字形);
	映射[名称] = typeof 初始 === "number" ? { element: 根名(初始) } : 初始;
	if (备选.length) {
		空间[名称] = 备选.map((目标) => ({
			value:
				typeof 目标 === "string" && 目标 in 键集
					? [{ variable: 目标 }]
					: { element: typeof 目标 === "number" ? 根名(目标) : 目标 },
			score: 0,
		}));
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

// 拉开人字头与首横，保持首三笔可认同为合上（00284）。
function 人冠融合字形(字形: number, 其余笔画: 矢量图形数据): 基本部件数据 {
	return {
		id: 字形,
		type: "component",
		operator: undefined,
		references: undefined,
		strokes: [
			{
				feature: "撇",
				start: [50, 12],
				curveList: [
					{ command: "c", parameterList: [-10, 19, -20, 39, -45, 58] },
				],
			},
			{
				feature: "捺",
				start: [50, 12],
				curveList: [{ command: "c", parameterList: [10, 19, 20, 39, 45, 58] }],
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
				from: 结构("⿸", 0x35306, 变量(1)),
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
			// 六个融合部件共用人冠；会、佘保留下部原笔画，两横仍为不可拆平行单元。
			人冠融合字形(0x60116, [
				{
					feature: "横",
					start: [31, 47],
					curveList: [{ command: "h", parameterList: [38] }],
				},
				{
					feature: "横",
					start: [34, 65],
					curveList: [{ command: "h", parameterList: [32] }],
				},
				{
					feature: "竖",
					start: [50, 47],
					curveList: [{ command: "v", parameterList: [37] }],
				},
				{
					feature: "横",
					start: [29, 84],
					curveList: [{ command: "h", parameterList: [42] }],
				},
			]), // 全
			人冠融合字形(0x0445b, [
				{
					feature: "横",
					start: [33, 47],
					curveList: [{ command: "h", parameterList: [35] }],
				},
				{
					feature: "横",
					start: [29, 61],
					curveList: [{ command: "h", parameterList: [43] }],
				},
				{
					feature: "撇折",
					start: [50, 61],
					curveList: [
						{ command: "c", parameterList: [-4, 9, -9, 17, -18, 24] },
						{ command: "h", parameterList: [36] },
					],
				},
				{
					feature: "点",
					start: [62, 79],
					curveList: [{ command: "c", parameterList: [3, 3, 6, 6, 7, 9] }],
				},
			]), // 会
			人冠融合字形(0x04543, [
				{
					feature: "横",
					start: [34, 47],
					curveList: [{ command: "h", parameterList: [32] }],
				},
				{
					feature: "横",
					start: [30, 60],
					curveList: [{ command: "h", parameterList: [40] }],
				},
				{
					feature: "竖钩",
					start: [50, 47],
					curveList: [{ command: "v", parameterList: [39] }],
				},
				{
					feature: "撇",
					start: [39, 67],
					curveList: [{ command: "c", parameterList: [-2, 6, -4, 12, -9, 17] }],
				},
				{
					feature: "点",
					start: [62, 67],
					curveList: [{ command: "c", parameterList: [4, 6, 7, 12, 8, 17] }],
				},
			]), // 余
			人冠融合字形(0x71108, [
				{
					feature: "横",
					start: [33, 47],
					curveList: [{ command: "h", parameterList: [35] }],
				},
				{
					feature: "横",
					start: [29, 60],
					curveList: [{ command: "h", parameterList: [43] }],
				},
				{
					feature: "竖钩",
					start: [50, 60],
					curveList: [{ command: "v", parameterList: [28] }],
				},
				{
					feature: "撇",
					start: [39, 68],
					curveList: [
						{ command: "c", parameterList: [-2, 5, -5, 10, -10, 15] },
					],
				},
				{
					feature: "点",
					start: [61, 68],
					curveList: [{ command: "c", parameterList: [4, 6, 8, 11, 10, 15] }],
				},
			]), // 佘
			人冠融合字形(0x04656, [
				{
					feature: "横",
					start: [34, 47],
					curveList: [{ command: "h", parameterList: [32] }],
				},
				{
					feature: "横",
					start: [32, 64],
					curveList: [{ command: "h", parameterList: [36] }],
				},
				{
					feature: "竖",
					start: [50, 47],
					curveList: [{ command: "v", parameterList: [38] }],
				},
				{
					feature: "点",
					start: [36, 71],
					curveList: [{ command: "c", parameterList: [3, 2, 6, 9, 7, 14] }],
				},
				{
					feature: "撇",
					start: [64, 71],
					curveList: [{ command: "c", parameterList: [-1, 3, -3, 8, -7, 14] }],
				},
				{
					feature: "横",
					start: [29, 85],
					curveList: [{ command: "h", parameterList: [43] }],
				},
			]), // 金
			人冠融合字形(0x87106, [
				{
					feature: "横",
					start: [32, 46],
					curveList: [{ command: "h", parameterList: [36] }],
				},
				{
					feature: "横",
					start: [28, 58],
					curveList: [{ command: "h", parameterList: [44] }],
				},
				{
					feature: "竖",
					start: [50, 46],
					curveList: [{ command: "v", parameterList: [28] }],
				},
				{
					feature: "竖",
					start: [34, 74],
					curveList: [{ command: "v", parameterList: [13] }],
				},
				{
					feature: "横折",
					start: [34, 74],
					curveList: [
						{ command: "h", parameterList: [33] },
						{ command: "v", parameterList: [13] },
					],
				},
				{
					feature: "横",
					start: [34, 87],
					curveList: [{ command: "h", parameterList: [33] }],
				},
			]), // 舍
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
			0x01142,
			0x0536a,
			0x04476,
			0x19104,
			0x0451f,
			0xa4106,
			0x05706,
			0x0810c,
			0x0341c,
			0x2a009,
			0x0422b,
			0x04202,
			0x19201,
			0x1852d,
			0x0831f,
			0x08313,
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
			0x0a317,
			0x08114,
			0x2011e,
			0x1e125,
			0x2a210,
			0x5d104,
			0x1c400,
			0x1c405,
			0x1d217,
			0x0163a,
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
			0x1d114,
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
			parameters: { t_max: 2.5, t_min: 0.015, steps: 1000000 },
		},
	},
};

export default 方案;
