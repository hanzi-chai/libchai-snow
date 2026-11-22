# 汉字编码算法・冰雪拼音特化

飞花每轮优化结束后，自动把当前最优解的小鱼数据、拆分取码报告和字根图写到 `output/feihua-current/`。报告入口为 `report.html`，「看字」支持筛选重码，「部件基本拆分」用紧凑网格展示完整拆分，默认隐藏单根的平凡拆分。

已有快照可以单独执行后处理（需要 Bun、发布版飞花编码器和 Inkscape）：

```bash
bun src/feihua/postprocess.ts --tree-data output/feihua-current/input.json --solution output/feihua-current/solution.json
```
