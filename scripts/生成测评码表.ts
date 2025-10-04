import { parse } from "csv-parse/sync";
import { readFileSync, writeFileSync } from "fs";
import { argv } from "process";

const 码表: Record<string, string>[] = parse(readFileSync(argv[2], "utf8"), {
  columns: ["字", "全码", "全码顺序", "简码", "简码顺序"],
  delimiter: "\t",
});

const 翻转码表: Record<string, string[]> = {};
for (const 码 of 码表) {
  const 简码 = (码.简码 as string).replace("_", "");
  if (简码 == "") continue;
  if (!翻转码表[简码]) 翻转码表[简码] = [];
  翻转码表[简码].push(码.字);
}
writeFileSync(
  "data/snow_qingyun.fixed.txt",
  Object.entries(翻转码表)
    .sort((a, b) => {
      if (a[0].length != b[0].length) return a[0].length - b[0].length;
      return a[0].localeCompare(b[0]);
    })
    .map((x) => `${x[0]}\t${x[1].join(" ")}`)
    .join("\n"),
  "utf8"
);

writeFileSync(
  "data/box.txt",
  码表.map((x) => `${x.字}\t${x.简码.replace("_", " ")}`).join("\n")
);

const 宇浩码表: [string, string][] = [];

for (const 码 of 码表) {
  宇浩码表.push([码.字, 码.全码.replace("_", "")]);
  if (码.简码 == "") continue;
  宇浩码表.push([码.字, 码.简码.replace("_", "")]);
}

writeFileSync(
  "data/冰雪清韵测试.txt",
  宇浩码表.map((x) => `${x[0]}\t${x[1]}`).join("\n")
);

const 拆分表: Record<string, string>[] = parse(
  readFileSync("data/拆分结果.txt", "utf8"),
  {
    columns: ["字", "拆分序列"],
    delimiter: "\t",
  }
);

const 大竹码表: [string, string][] = [];
for (const 码 of 码表) {
  if (码.简码 == "") continue;
  大竹码表.push([码.简码, 码.字]);
}
for (const 码 of 码表) {
  if (码.简码 == 码.全码) continue;
  大竹码表.push([码.全码, 码.字]);
}
for (const 拆分 of 拆分表) {
  大竹码表.push([`拆分［${拆分.拆分序列}］`, 拆分.字]);
}

writeFileSync(
  "data/dazhu.txt",
  大竹码表.map((x) => `${x[0]}\t${x[1]}`).join("\n"),
  "utf8"
);
