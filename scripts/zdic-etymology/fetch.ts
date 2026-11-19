/**
 * 抓取 GB2312 的 6763 个汉字，分别保存三个可能包含字源的区域全文。
 * 用法与输出格式见 scripts/zdic-etymology/AGENTS.md。需要 Node.js 22.18+。
 */
import { mkdir, readFile, rename, writeFile } from "node:fs/promises";
import path from "node:path";
import { setTimeout as 等待 } from "node:timers/promises";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import { load } from "cheerio";

export function 获取字集(): string[] {
	// GB2312 汉字区：16–55 区为一级，56–87 区为二级；55 区末五格为空。
	// 不遍历 GBK 扩展区，避免把 GBK 的新增字误算进 GB2312。
	const 解码器 = new TextDecoder("gb18030", { fatal: true });
	const 字集: string[] = [];
	for (let 区 = 16; 区 <= 87; 区++) {
		for (let 位 = 1; 位 <= (区 === 55 ? 89 : 94); 位++) {
			字集.push(解码器.decode(Uint8Array.of(区 + 0xa0, 位 + 0xa0)));
		}
	}
	if (字集.length !== 6763 || new Set(字集).size !== 6763) {
		throw new Error("GB2312 字集生成失败");
	}
	return 字集;
}

export interface 区域数据 {
	文本: string;
	HTML: string;
	图片: { 说明: string; 地址: string }[];
	链接: { 文本: string; 地址: string }[];
}

export function 解析页面(汉字: string, HTML: string) {
	const 页面 = load(HTML);
	const 网址 = `https://www.zdic.net/hans/${encodeURIComponent(汉字)}`;
	// 不将验证页、错误页或其他条目当作“没有字源”缓存。
	if (
		页面('.char-meta[itemprop="name"]').attr("content") !== 汉字 ||
		!页面(".char-card, #jbjs, #xxjs, #swjz").length
	) {
		throw new Error("未识别到对应汉字的字典正文，可能遇到验证页或页面结构变更");
	}
	const 读取区域 = (选择器: string): 区域数据 | null => {
		const 节点 = 页面(选择器).first();
		if (!节点.length) return null;
		const 副本 = 节点.clone();
		副本.find("script, style, button, .dict-section__footer").remove();
		const 图片 = 副本
			.find("img")
			.toArray()
			.flatMap((节点) => {
				const 图片 = 页面(节点);
				const 地址 = 图片.attr("src") || 图片.attr("data-src");
				return 地址
					? [{ 说明: 图片.attr("alt") || "", 地址: new URL(地址, 网址).href }]
					: [];
			});
		const 链接 = 副本
			.find("a[href]")
			.toArray()
			.map((节点) => ({
				文本: 页面(节点).text().trim(),
				地址: new URL(页面(节点).attr("href") ?? "", 网址).href,
			}));
		// 原片段保留内嵌图片及全部标记；纯文本按块换行，保留罕见字的隐藏文本。
		const 原片段 = 副本.html() || "";
		副本.find("img[alt]").each((_, 节点) => {
			页面(节点).replaceWith(页面(节点).attr("alt") || "");
		});
		副本.find("br").replaceWith("\n");
		副本
			.find("p, div, li, h2, h3, header, section, article")
			.each((_, 节点) => {
				页面(节点).append("\n");
			});
		const 文本 = 副本
			.text()
			.replace(/[^\S\n]+/g, " ")
			.split("\n")
			.map((行) => 行.trim())
			.filter(Boolean)
			.join("\n");
		return { 文本, HTML: 原片段, 图片, 链接 };
	};
	const 分析标签 = 页面(".char-meta .meta-badge").filter(
		(_, 节点) => 页面(节点).text().trim() === "字形分析",
	);
	const 分析值 = 分析标签.next(".meta-value");
	return {
		汉字,
		网址,
		字形分析: 分析值.length ? 分析值.text().trim() : null,
		详细解释: 读取区域("#xxjs .dict-section__body"),
		说文解字: 读取区域("#swjz .dict-section__body"),
	};
}

async function 原子写入(文件: string, 内容: string) {
	const 临时文件 = `${文件}.${process.pid}.tmp`;
	await writeFile(临时文件, 内容);
	await rename(临时文件, 文件);
}

async function 读取缓存(文件: string) {
	try {
		return await readFile(文件, "utf8");
	} catch (错误) {
		if ((错误 as NodeJS.ErrnoException).code === "ENOENT") return null;
		throw 错误;
	}
}

async function 主程序() {
	const { values: 参数 } = parseArgs({
		options: {
			out: { type: "string", default: "output/zdic-etymology" },
			chars: { type: "string" },
			limit: { type: "string" },
			delay: { type: "string", default: "2000" },
			retries: { type: "string", default: "3" },
			refresh: { type: "boolean", default: false },
			offline: { type: "boolean", default: false },
			help: { type: "boolean", default: false },
		},
	});
	if (参数.help) {
		console.log(
			"node scripts/zdic-etymology/fetch.ts [--out 目录] [--chars 颖明] [--limit 数量] [--delay 2000] [--retries 3] [--refresh] [--offline]\n默认抓取 GB2312 全部 6763 字。缓存和断点自动保存，详见 scripts/zdic-etymology/AGENTS.md。",
		);
		return;
	}
	const 数值参数 = (值: string, 名称: string, 下限: number) => {
		const 数值 = Number(值);
		if (!Number.isSafeInteger(数值) || 数值 < 下限)
			throw new Error(`${名称} 必须为不小于 ${下限} 的整数`);
		return 数值;
	};
	const 间隔 = 数值参数(参数.delay ?? "2000", "--delay", 1000);
	const 重试次数 = 数值参数(参数.retries ?? "3", "--retries", 0);
	const 全字集 = 获取字集();
	let 字集 = 参数.chars === undefined ? 全字集 : [...new Set([...参数.chars])];
	if (!字集.length || 字集.some((字) => !全字集.includes(字)))
		throw new Error("--chars 必须仅包含 GB2312 汉字");
	if (参数.limit !== undefined)
		字集 = 字集.slice(0, 数值参数(参数.limit, "--limit", 1));
	if (参数.refresh && 参数.offline)
		throw new Error("--refresh 和 --offline 不能同时使用");
	const 目录 = path.resolve(参数.out ?? "output/zdic-etymology");
	await mkdir(path.join(目录, "html"), { recursive: true });
	await mkdir(path.join(目录, "characters"), { recursive: true });
	let 停止 = false;
	process.once("SIGINT", () => {
		停止 = true;
		console.error("正在保存进度，再次 Ctrl+C 可强制退出。");
	});
	let 下次请求时间 = 0;
	const 下载 = async (网址: string): Promise<string> => {
		for (let 尝试 = 0; ; 尝试++) {
			await 等待(Math.max(0, 下次请求时间 - Date.now()));
			if (停止) throw new Error("用户中断");
			下次请求时间 = Date.now() + 间隔;
			try {
				const 响应 = await fetch(网址, {
					headers: {
						"User-Agent": "GB2312-Etymology-Collector/1.0",
						Accept: "text/html",
						"Accept-Language": "zh-CN,zh;q=0.9",
					},
					signal: AbortSignal.timeout(30_000),
				});
				if (!响应.ok) {
					await 响应.body?.cancel();
					if (响应.status === 403) {
						停止 = true;
						throw new Error("HTTP 403：站点拒绝访问，停止抓取");
					}
					if (响应.status === 429 || 响应.status === 503) {
						const 值 = 响应.headers.get("retry-after");
						const 时长 = 值
							? /^\d+$/.test(值)
								? Number(值) * 1000
								: Date.parse(值) - Date.now()
							: 60_000;
						if (Number.isFinite(时长))
							下次请求时间 = Math.max(下次请求时间, Date.now() + 时长);
					}
					throw new Error(`HTTP ${响应.status}`);
				}
				return await 响应.text();
			} catch (错误) {
				if (停止 || 尝试 >= 重试次数) throw 错误;
				console.error(`请求失败，准备第 ${尝试 + 1} 次重试：${错误}`);
				下次请求时间 = Math.max(
					下次请求时间,
					Date.now() + 间隔 * 2 ** (尝试 + 1),
				);
			}
		}
	};
	const 汇总: ReturnType<typeof 解析页面>[] = [];
	const 失败: { 汉字: string; 错误: string }[] = [];
	let 连续失败 = 0;
	for (const [序号, 汉字] of 字集.entries()) {
		if (停止) break;
		const 缓存路径 = path.join(目录, "html", `${汉字}.html`);
		try {
			const 缓存 = 参数.refresh ? null : await 读取缓存(缓存路径);
			if (参数.offline && 缓存 === null) throw new Error("缺少 HTML 缓存");
			const HTML =
				缓存 ??
				(await 下载(`https://www.zdic.net/hans/${encodeURIComponent(汉字)}`));
			const 结果 = 解析页面(汉字, HTML);
			if (缓存 === null) await 原子写入(缓存路径, HTML);
			await 原子写入(
				path.join(目录, "characters", `${汉字}.json`),
				`${JSON.stringify(结果, null, 2)}\n`,
			);
			汇总.push(结果);
			连续失败 = 0;
			console.log(
				`[${序号 + 1}/${字集.length}] ${汉字} ${缓存 === null ? "已下载" : "使用缓存"}`,
			);
		} catch (错误) {
			失败.push({ 汉字, 错误: String(错误) });
			console.error(`${汉字}：${错误}`);
			if (++连续失败 >= 5 && !参数.offline) {
				停止 = true;
				console.error("连续失败五字，停止抓取；重新运行可续爬。");
			}
		}
	}
	// 汇总只包含本次请求的字集；全量重跑可用缓存重建全部汇总。
	await 原子写入(
		path.join(目录, "etymology.json"),
		`${JSON.stringify(汇总, null, 2)}\n`,
	);
	const 状态 = {
		请求字数: 字集.length,
		成功字数: 汇总.length,
		失败,
		未处理: 字集.slice(汇总.length + 失败.length),
		缺失区域: 汇总
			.map((结果) => ({
				汉字: 结果.汉字,
				区域: (["字形分析", "详细解释", "说文解字"] as const).filter(
					(键) => 结果[键] === null,
				),
			}))
			.filter((项) => 项.区域.length),
		更新时间: new Date().toISOString(),
	};
	await 原子写入(
		path.join(目录, "status.json"),
		`${JSON.stringify(状态, null, 2)}\n`,
	);
	console.log(`已保存 ${汇总.length}/${字集.length} 字：${目录}`);
	if (失败.length || 状态.未处理.length) process.exitCode = 1;
}

if (
	process.argv[1] &&
	import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href
) {
	主程序().catch((错误) => {
		console.error(错误);
		process.exitCode = 1;
	});
}
