# 汉字编码算法・冰雪拼音特化

## 准备文件

- `assets/key_distribution.txt`, `assets/pair_equivalence.txt`：用指分布和当量文件，含义和 chai 中相同
- `snow2.yaml`, `snow2.txt`：配置文件和元素序列文件，含义和 chai 中相同
- `tree.yaml`, `dual.yaml`：储存了字根树信息和双编码信息

# 运行

```bash
cargo run --release -- snow2.yaml -e snow2.txt optimize
```

本程序还支持多线程运行，使用 `-t` 指定线程数量，例如

```bash
cargo run --release -- snow2.yaml -e snow2.txt -t 10 optimize
```

多线程时计算进度的输出会重定向到 `output-xxx/<线程编号>/log.txt`。
