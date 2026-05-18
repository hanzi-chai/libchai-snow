import { readFileSync } from "fs";
import yaml from "js-yaml";

const path = process.argv[2] ?? "output-05-11+14_40_32/0/config.yaml";
const data = yaml.load(readFileSync(path, "utf8")) as any;
const mapping: Record<string, string | { element: string }> = data.form.mapping;

const isAnchor = (key: string) => /^(声|韵|形)-/.test(key);

const cache = new Map<string, string>();
function resolve(key: string): string {
  if (cache.has(key)) return cache.get(key)!;
  const val = mapping[key];
  const result = typeof val === "object" ? resolve(val.element) : val;
  cache.set(key, result);
  return result;
}

// Classify a root element's category by its direct anchor target
function classify(key: string): "声" | "韵" | "形" | "特殊" {
  const val = mapping[key];
  if (typeof val === "object") {
    const target = val.element;
    if (target.startsWith("声-")) return "声";
    if (target.startsWith("韵-")) return "韵";
    if (target.startsWith("形-")) return "形";
  }
  return "特殊";
}

const nonAnchorChildren = new Map<string, string[]>();
for (const [key, val] of Object.entries(mapping)) {
  if (!isAnchor(key) && typeof val === "object") {
    const target = val.element;
    if (!isAnchor(target)) {
      if (!nonAnchorChildren.has(target)) nonAnchorChildren.set(target, []);
      nonAnchorChildren.get(target)!.push(key);
    }
  }
}

const hasNonAnchorParent = new Set<string>();
for (const children of nonAnchorChildren.values()) {
  for (const child of children) hasNonAnchorParent.add(child);
}

function collectGroup(key: string): string[] {
  const children = nonAnchorChildren.get(key) ?? [];
  return [key, ...children.flatMap(collectGroup)];
}

const categories = { 声: {} as Record<string, string>, 韵: {} as Record<string, string>, 形: {} as Record<string, string>, 特殊: {} as Record<string, string> };

for (const key of Object.keys(mapping)) {
  if (!isAnchor(key) && !hasNonAnchorParent.has(key)) {
    const group = collectGroup(key);
    const groupStr = group.join("");
    const letter = resolve(key);
    const cat = classify(key);
    categories[cat][groupStr] = letter;
  }
}

const output = [
  { type: "keymap", sources: categories["声"]},
  { type: "keymap", sources: categories["韵"]},
  { type: "keymap", sources: categories["形"]},
  { type: "keymap", sources: categories["特殊"]}
];
console.log(JSON.stringify(output, null, 2));
