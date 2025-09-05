import { parse } from "csv-parse/sync";
import { readFileSync, writeFileSync } from "fs";

const 码表: Record<string, string>[] = parse(
  readFileSync("/Users/tansongchen/Public/libchai-snow/output-08-09+12_34_55/8/编码.txt", "utf8"),
  {
    columns: ["字", "全码", "全码顺序", "简码", "简码顺序"],
    delimiter: "\t",
  }
);
const 拆分表: Record<string, string>[] = parse(
  readFileSync("debug/拆分结果.txt", "utf8"),
  {
    columns: ["字", "拆分序列"],
    delimiter: "\t",
  }
);

const 大竹码表: [string, string][] = [];
for (const 码 of 码表) {
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
  "debug/dazhu.txt",
  大竹码表.map((x) => `${x[0]}\t${x[1]}`).join("\n"),
  "utf8"
);

writeFileSync(
  "debug/box.txt",
  码表.map((x) => `${x.字}\t${x.简码.replace("_", " ")}`).join("\n"),
  "utf8"
);
