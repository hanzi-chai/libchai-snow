import { readFileSync, writeFileSync } from "fs";

const content = readFileSync("data/tw2g.txt", "utf8");
const result: string[] = [];
for (const line of content.trim().split("\n")) {
  if (line.startsWith("#") || line.trim() === "") continue;
  const [台标, 陆标] = line.split(/\s+/);
  result.push(`${陆标}\t${台标}`);
}

writeFileSync("data/t2tw.txt", result.join("\n"));