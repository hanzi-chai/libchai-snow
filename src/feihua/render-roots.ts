import { spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, extname, relative, resolve, sep } from "node:path";
import {
	包围内部范围表,
	字符,
	生成笔画路径,
	获取字库,
	读取配置,
} from "hanzi-chai";

const 输出目录 = resolve("output/feihua-charts");

const 帮助 = `渲染冰雪飞花字根图（SVG、PNG、A4 横版 PDF）
用法：bun src/feihua/render-roots.ts --config <优化结果/config.yaml> [--out <输出前缀>]
  --out       默认 output/feihua-charts/root-chart；也可传 .svg、.png 或 .pdf 路径
  --inkscape  Inkscape 可执行文件，默认 inkscape（PDF 文字转曲，便于打印）`;
const 参数 = new Map<string, string>();
for (let 位 = 2; 位 < process.argv.length; 位++) {
	const 名 = process.argv[位];
	if (名 === "--help" || 名 === "-h") {
		console.log(帮助);
		process.exit(0);
	}
	const 值 = process.argv[位 + 1];
	if (
		!["--config", "--out", "--inkscape"].includes(名) ||
		!值 ||
		值.startsWith("--")
	)
		throw Error(`无效参数：${名}\n${帮助}`);
	参数.set(名, 值);
	位++;
}
const 配置参数 = 参数.get("--config");
if (!配置参数) throw Error(帮助);
const 配置路径 = resolve(配置参数);
let 前缀 = resolve(参数.get("--out") ?? `${输出目录}/root-chart`);
if ([".svg", ".png", ".pdf"].includes(extname(前缀))) 前缀 = 前缀.slice(0, -4);
const 输出相对路径 = relative(输出目录, 前缀);
if (
	!输出相对路径 ||
	输出相对路径 === ".." ||
	输出相对路径.startsWith(`..${sep}`)
)
	throw Error("--out 必须位于 output/feihua-charts/ 内，并包含文件名前缀");
if (["svg", "png", "pdf"].some((格式) => `${前缀}.${格式}` === 配置路径))
	throw Error("输出不能覆盖输入配置");

const 方案 = 读取配置(配置路径);
const 映射 = 方案.form.mapping;
const 字库 = 获取字库(方案);
const 键序 = ["AOEIUV;/", "BPMFDTNL", "GKHWJQXY", "ZCSR,."];
const 全键 = 键序.join("").toLowerCase();
const 笔画 = ["⼀", "⼁", "⼃", "⼂", "乙"];
const 是根名 = (名称: string) => /^字形-[0-9a-f]{5}$|^[1-5]$/.test(名称);
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

// 以真实归并链的终点分组；同键的独立字根不会被合为一组。
const 终点缓存 = new Map<string, { 名称: string; 键: string }>();
function 取终点(
	名称: string,
	链 = new Set<string>(),
): { 名称: string; 键: string } {
	const 缓存 = 终点缓存.get(名称);
	if (缓存) return 缓存;
	if (链.has(名称)) throw Error(`归并成环：${[...链, 名称].join(" → ")}`);
	链.add(名称);
	const 安排 = 映射[名称];
	if (安排 == null) throw Error(`归并目标未启用或不存在：${名称}`);
	let 结果: { 名称: string; 键: string };
	if (typeof 安排 === "string") {
		if (安排.length !== 1 || !全键.includes(安排))
			throw Error(`字根必须安排在一个有效键上：${名称} = ${安排}`);
		结果 = { 名称, 键: 安排 };
	} else if (!Array.isArray(安排) && typeof 安排.element === "string") {
		const 目标 = 取终点(安排.element, 链);
		// 共享声母锚点不等于字根归并；与优化目标函数的组数口径一致。
		结果 = { 名称: 是根名(安排.element) ? 目标.名称 : 名称, 键: 目标.键 };
	} else throw Error(`不支持的字根安排：${名称}（需要单键或 element 归并）`);
	终点缓存.set(名称, 结果);
	return 结果;
}
const 键笔画 = new Map<string, string[]>();
for (const [位, 符号] of 笔画.entries()) {
	const 键 = 取终点(String(位 + 1)).键;
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
// 包围变形若当前与原形沿真实归并链同组，就只展示原形。
// 空间是否另有可选键位不影响当前键面的去重；独立安排的根仍保留。
function 是包围变形归并根(名称: string): boolean {
	if (!/^字形-[0-9a-f]{5}$/.test(名称)) return false;
	const 编号 = parseInt(名称.slice(3), 16);
	const 原形 = 包围内部范围表[String(编号)]?.sourceGlyphId;
	if (原形 === undefined || 原形 === 编号) return false;
	const 原形名 = `字形-${原形.toString(16).padStart(5, "0")}`;
	return (
		映射[名称] != null &&
		映射[原形名] != null &&
		取终点(名称).名称 === 取终点(原形名).名称
	);
}
const 屏蔽根 = new Set(Object.keys(映射).filter(是包围变形归并根));
const 分组 = new Map<string, 根组>();
for (const [名称, 安排] of Object.entries(映射)) {
	if (安排 == null || !是根名(名称) || 屏蔽根.has(名称)) continue;
	const 终点 = 取终点(名称);
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
for (const 组 of 分组.values()) {
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
const 行高 = 键序.map((行, 序) =>
	Math.max(
		序 === 3 ? 240 : 156,
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
// 标题与字根共用字库笔画，避免标题字形随系统字体变化。
片段.push('<g data-heading="冰雪飞花" aria-label="冰雪飞花">');
for (const [序, 字] of [..."冰雪飞花"].entries()) {
	const 如字符 = 字符.从码位创建(字.codePointAt(0) ?? 0);
	if (!如字符.ok) throw 如字符.error;
	const 形 = 字库.查询字符的字形(如字符.value)?.[0];
	if (!形) throw Error(`字库缺少标题字：${字}`);
	片段.push(
		`<svg data-heading-char="${字}" x="${边距 + 序 * 48}" y="30" width="48" height="48" viewBox="0 0 100 100"><g fill="none" stroke="#17364a" stroke-width="5.4" stroke-linecap="round" stroke-linejoin="round">${生成笔画路径(
			形.图形盒子.获取笔画列表(),
		)
			.map((径) => `<path d="${逃逸(径)}"/>`)
			.join("")}</g></svg>`,
	);
}
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
文本(
	宽 - 边距,
	75,
	"字根图",
	20,
	"#617380",
	'text-anchor="end" letter-spacing="4"',
);
文本(
	宽 - 边距,
	111,
	"zh / ch / sh → Z / C / S　　零声母 → R",
	16,
	"#617380",
	'text-anchor="end"',
);
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
		if (笔画标记)
			文本(
				横 + 格宽 - 15,
				纵 + 32,
				笔画标记.join(""),
				27,
				色.字,
				'text-anchor="end"',
			);
		for (const 组 of 取布局(键).组列表) {
			片段.push(
				`<g data-group="${逃逸(组.终点)}" data-irrational="${组.无理}"><title>${逃逸(组.字根.join("、"))}</title>`,
			);
			for (const [序, 名称] of 组.字根.entries()) {
				const 形 = 字库.获取字形(parseInt(名称.slice(3), 16));
				if (!形) throw Error(`字库缺少字根：${名称}`);
				const 格号 = 组.起格 + 序;
				const 字横 = 横 + 12 + (格号 % 每行根数) * 格距 + (格距 - 字号) / 2;
				const 字纵 =
					纵 + 49 + Math.floor(格号 / 每行根数) * 格距 + (格距 - 字号) / 2;
				片段.push(
					`<svg data-root="${名称}" x="${字横}" y="${字纵}" width="${字号}" height="${字号}" viewBox="0 0 100 100"><g fill="none" stroke="${组.无理 ? 配色.无理 : 色.字}" stroke-width="5.4" stroke-linecap="round" stroke-linejoin="round">${生成笔画路径(
						形.图形盒子.获取笔画列表(),
					)
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
			{
				标题: "部件字",
				行: [
					"声母 ＋ 前三个基本根",
					"",
					"按笔顺取根，不足取全。",
					"整字为基本根时，",
					"可选 AB／ABB／ABBB。",
				],
			},
			{
				标题: "复合体字",
				行: [
					"声母 ＋ 正根 ＋ 余根",
					"",
					"正部取一根；",
					"余部最多取两个基本根。",
					"依方案规则分部，",
					"指定意符优先。",
				],
			},
		];
		for (const [序, 规则格] of 规则.entries()) {
			const 横 = 边距 + (6 + 序) * (格宽 + 间距);
			矩形(横, 纵, 格宽, 高, 配色.灰.底, 配色.灰.边);
			文本(横 + 15, 纵 + 33, 规则格.标题, 23, 配色.灰.字, 'font-weight="700"');
			for (const [位, 行] of 规则格.行.entries())
				文本(横 + 15, 纵 + 68 + 位 * 24, 行, 14.5);
		}
	}
	纵 += 高 + 间距;
}
const 根数 = [...分组.values()].reduce((和, 组) => 和 + 组.字根.length, 0);
function 统计字根(基本: boolean) {
	const 组 = [...分组.values()].filter(
		(组) => 键序[0].toLowerCase().includes(组.键) === 基本,
	);
	return `${组.length} 组 ${组.reduce((和, 项) => 和 + 项.字根.length, 0)} 个`;
}
const 例外组数 = [...分组.values()].filter((组) => 组.无理).length;
const 统计文本 = `基本字根 ${统计字根(true)}・扩展字根 ${统计字根(false)}（非声母归并 ${例外组数} 组）`;
文本(边距, 纵 + 20, 统计文本, 15, "#617380");
const SVG = `<svg xmlns="http://www.w3.org/2000/svg" width="297mm" height="210mm" viewBox="0 0 ${宽} ${页高}" role="img" aria-labelledby="title desc">
<title id="title">冰雪飞花 beta 字根图</title><desc id="desc">键序 AOEIUV;/、BPMFDTNL、GKHWJQXY、ZCSR,.。归并字根同组排列，右下两格为取码规则。</desc>
<rect width="${宽}" height="${页高}" fill="#ffffff"/>
<g font-family="PingFang SC, Noto Sans CJK SC, Source Han Sans SC, sans-serif" transform="translate(${平移横} ${平移纵}) scale(${缩放})">${片段.join("\n")}</g></svg>\n`;
mkdirSync(dirname(前缀), { recursive: true });
writeFileSync(`${前缀}.svg`, SVG);
for (const 格式 of ["png", "pdf"]) {
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
}
console.log(
	`已屏蔽 ${屏蔽根.size} 个包围变形归并根；${统计文本}（含五笔画，共 ${根数} 个、${分组.size} 组）：\n${前缀}.svg\n${前缀}.png\n${前缀}.pdf`,
);
