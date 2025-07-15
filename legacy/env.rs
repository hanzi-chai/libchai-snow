// 首先在 Cargo.toml 加上
// num_cpus = "1.16" （或者最新版）

fn main() {
    let cores = num_cpus::get();
    println!("系统可以并行运行的线程数: {}", cores);
}