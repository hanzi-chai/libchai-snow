import { parse } from "csv-parse/sync";
import { readFileSync, writeFileSync } from "fs";

const 码表: Record<string, string>[] = parse(
  readFileSync("output-09-19+22_07_19/2/编码.txt", "utf8"),
  {
    columns: ["字", "全码", "全码顺序", "简码", "简码顺序"],
    delimiter: "\t",
  }
);
writeFileSync(
  "data/box.txt",
  码表.map((x) => `${x.字}\t${x.简码.replace("_", " ")}`).join("\n")
);

const 宇浩码表: [string, string][] = [];

for (const 码 of 码表) {
  宇浩码表.push([码.字, 码.全码]);
  if (码.简码 == "") continue;
  宇浩码表.push([码.字, 码.简码]);
}

writeFileSync(
  "data/ceping.txt",
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
