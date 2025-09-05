import { readFileSync, writeFileSync } from "fs";

const content = readFileSync("flamegraph.svg", "utf-8");
const replacedContent = content.replace(/\$u([0-9a-fA-F]+)\$/g, (_, hex) => {
  const num = parseInt(hex, 16);
  return String.fromCodePoint(num);
});
writeFileSync("flamegraph-new.svg", replacedContent, "utf-8");