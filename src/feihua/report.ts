import { readFileSync } from "node:fs";

export type 报告元素 = {
	label: string;
	name: string;
	key: string;
	colorRole: string;
};
export type 报告字形 = {
	id: string;
	名称: string;
	别名: string[];
	minimal: string[];
	类别: "部件" | "复合体";
	paths: string[];
	roots: { strokes: number[]; name: string; colorRole: string }[];
	作为部件的形码: string;
	完整拆分: { 名称: string; 键: string; 笔画: number[] }[];
	分部: { id: string; 正部: boolean }[];
	行: {
		字: string;
		拼音: string;
		编码: string;
		元素: 报告元素[];
		重码同组: string[];
	}[];
};
export type 报告 = {
	版本: 1;
	来源: string;
	指标: {
		重码: number;
		小集合组数: number;
		补集合例外组数: number;
		分数: number;
	};
	统计: {
		minimal字数: number;
		字声母数: number;
		字形数: number;
		部件数: number;
		重码组数: number;
	};
	字根图: Record<string, string[]>;
	部件顺序: string[];
	字形: 报告字形[];
};

const 报告配色 = {
	initial: "#526077",
	component1: "#527e00",
	component2: "#1559d4",
	component3: "#9c3489",
	positive: "#c13e32",
	remainder1: "#527e00",
	remainder2: "#1559d4",
};

export function 生成报告(数据: 报告): string {
	const 模板 = readFileSync(new URL("./report.html", import.meta.url), "utf8");
	return 模板
		.replace('"{{data}}"', () =>
			JSON.stringify(数据).replaceAll("<", "\\u003c"),
		)
		.replace('"{{palette}}"', () => JSON.stringify(报告配色));
}
