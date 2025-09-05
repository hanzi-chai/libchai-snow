import { readFileSync, writeFileSync } from "node:fs";

// https://language.moe.gov.tw/001/Upload/files/SITE_CONTENT/M0001/PIN/biau1.htm?open
const content = readFileSync("debug/ftzp_raw.txt", "utf8");

const count = (str: string, pat: string) => {
  let count = 0;
  let pos = str.indexOf(pat);
  while (pos !== -1) {
    count++;
    pos = str.indexOf(pat, pos + pat.length);
  }
  return count;
};

const result: [string, string][] = [];

for (const line of content.trim().split("\n")) {
  if (count(line, "│") < 4 || count(line, ".") < 1) continue;
  const fields = line.split(/[│║]/).map((x) => x.trim());
  result.push([fields[2], fields[5]]);
}

writeFileSync(
  "debug/ftzp.txt",
  result.map((x) => `${x[0]}\t${x[1]}`).join("\n")
);
