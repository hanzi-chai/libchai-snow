import { 默认分类器, type 笔画名称 } from "hanzi-chai";
import { 基本拆分配色, type 报告, type 报告字形, 比较笔画顺序 } from "./report.ts";

const 页宽 = 794;
const 页高 = 1122;
const 边距 = 28;
const 间距 = 5;
const 列数 = 6;
const 栏宽 = (页宽 - 边距 * 2 - 间距 * (列数 - 1)) / 列数;
const 部件大小 = 46;
const 字根大小 = 18;
const 根格宽 = 21;
const 根格高 = 21;
const 每行根数 = 3;
const 内容顶 = 26;
const 内容底 = 页高 - 28;
const 区标题高 = 26;
const 笔画顺位 = new Map(Object.keys(默认分类器).map((名, i) => [名, i]));
const 起笔名称 = ["横", "竖", "撇", "点", "折"];

const 转义 = (值: string) =>
	值.replaceAll("&", "&amp;")
		.replaceAll("<", "&lt;")
		.replaceAll(">", "&gt;")
		.replaceAll('"', "&quot;");

function 图形(横: number, 纵: number, 大小: number, 笔画: string[], 色: string[]) {
	return `<g transform="translate(${横} ${纵}) scale(${大小 / 100})" fill="none" stroke-width="4" stroke-linecap="round" stroke-linejoin="round">
${笔画.map((径, i) => `<path d="${转义(径)}" stroke="${色[i]}"/>`).join("")}</g>`;
}

function 部件图解(项: 报告字形, 高: number, 字根图: 报告["字根图"]) {
	const 笔画色 = 项.paths.map(() => "#a4aab3");
	const 根区高 = Math.ceil(项.完整拆分.length / 每行根数) * 根格高;
	const 根区顶 = (高 - 根区高) / 2;
	const 根格 = 项.完整拆分.map((根, i) => {
		const 色 = 基本拆分配色[i % 基本拆分配色.length];
		for (const 笔 of 根.笔画) 笔画色[笔] = 色;
		const 径 = 字根图[根.名称];
		if (!径) throw Error(`基本拆分 PDF 缺少字根图形：${项.id} ${根.名称}`);
		const 横 = 部件大小 + 6 + (i % 每行根数) * 根格宽;
		const 纵 = 根区顶 + Math.floor(i / 每行根数) * 根格高 + (根格高 - 字根大小) / 2;
		return 图形(横, 纵, 字根大小, 径, 径.map(() => 色));
	}).join("");
	return `${图形(0, (高 - 部件大小) / 2, 部件大小, 项.paths, 笔画色)}${根格}`;
}

export function 生成基本拆分分页图(
	数据: 报告,
	部件排序键: readonly { id: string; 笔画名称: readonly 笔画名称[] }[],
): string {
	const 字形 = new Map(数据.字形.map((项) => [项.id, 项]));
	type 部件项 = (typeof 部件排序键)[number] & { 字形: 报告字形 };
	const 部件区 = 起笔名称.map((): 部件项[] => []);
	for (const 键 of 部件排序键) {
		const 项 = 字形.get(键.id);
		if (!项) throw Error(`基本拆分 PDF 缺少部件：${键.id}`);
		if (项.完整拆分.length <= 1) continue;
		const 起笔 = 默认分类器[键.笔画名称[0]];
		if (!起笔) throw Error(`基本拆分 PDF 缺少首笔类别：${键.id}`);
		部件区[起笔 - 1].push({ ...键, 字形: 项 });
	}
	const 页面: string[][] = [[]];
	let 纵 = 内容顶;
	for (const [区号, 区] of 部件区.entries()) {
		区.sort((甲, 乙) => 比较笔画顺序(甲, 乙, 笔画顺位));
		for (let 起点 = 0; 起点 < 区.length; 起点 += 列数) {
			const 行 = 区.slice(起点, 起点 + 列数).map((项) => 项.字形);
			const 最大根数 = Math.max(...行.map((项) => 项.完整拆分.length));
			const 高 = Math.max(部件大小, Math.ceil(最大根数 / 每行根数) * 根格高) + 5;
			const 首行 = 起点 === 0;
			const 换页 = 纵 + 高 + (首行 ? 区标题高 : 0) > 内容底;
			if (换页) {
				页面.push([]);
				纵 = 内容顶;
			}
			const 本页 = 页面[页面.length - 1];
			if (首行) {
				本页.push(`<text x="${边距}" y="${纵 + 15}" font-size="14" font-weight="600" fill="#26394a">${起笔名称[区号]}起笔</text>`);
				纵 += 区标题高;
			}
			for (const [列, 项] of 行.entries()) {
				const 横 = 边距 + 列 * (栏宽 + 间距);
				本页.push(
					`<g id="split-${转义(项.id)}" transform="translate(${横} ${纵})">${部件图解(项, 高, 数据.字根图)}</g>`,
				);
			}
			纵 += 高 + 间距;
		}
	}
	const 页序 = (i: number) => i * (页高 + 12);
	const 页标记 = 页面.map((_, i) =>
		`<inkscape:page id="page-${i + 1}" x="0" y="${页序(i)}" width="${页宽}" height="${页高}"/>`,
	).join("\n");
	const 内容 = 页面.map((片段, i) => `<g transform="translate(0 ${页序(i)})">${片段.join("\n")}</g>`).join("\n");
	return `<svg xmlns="http://www.w3.org/2000/svg" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd" width="210mm" height="297mm" viewBox="0 0 ${页宽} ${页高}">
<sodipodi:namedview id="view">${页标记}</sodipodi:namedview>
<g font-family="PingFang SC, Noto Sans CJK SC, Source Han Sans SC, sans-serif">${内容}</g>
</svg>\n`;
}
