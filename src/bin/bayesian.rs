use chai::config::SolverConfig;
use chai::optimizers::simulated_annealing::降温时间表;
use chai::optimizers::{优化方法, 优化问题};
use chai::{命令行, 命令行参数, 错误};
use clap::Parser;
use snow::snow2::snow2objective::冰雪二拼目标函数;
use snow::snow2::snow2operators::冰雪二拼操作;
use snow::snow2::冰雪二拼编码器;
use std::fs::File;
use std::io::Write;
use std::thread::{sleep, spawn};

fn main() -> Result<(), 错误> {
    let 参数 = 命令行参数::parse();
    let 组数 = 参数.threads.unwrap_or(1);
    sleep(std::time::Duration::from_secs(组数 as u64));
    let 命令行 = 命令行::新建(参数, None);
    let mut 数据 = 命令行.准备数据();
    数据.词列表.sort_by_key(|词| 词.词长);
    数据.词列表.iter_mut().for_each(|词| {
        if 词.词长 == 1 {
            词.词长 = 0;
        }
    });
    let _config = 数据.配置.clone();
    let 组数 = 命令行.参数.threads.unwrap_or(1);
    let SolverConfig::SimulatedAnnealing(mut 退火) =
        _config.optimization.unwrap().metaheuristic.unwrap();
    let t_max_list = vec![10.0, 1.0, 0.1];
    let t_min_list = vec![1e-5, 1e-6, 1e-7];
    let all_list: Vec<_> = t_max_list
        .iter()
        .flat_map(|t_max| t_min_list.iter().map(|t_min| (*t_max, *t_min)))
        .collect();
    if 组数 >= all_list.len() {
        return Ok(());
    }
    let (t_max, t_min) = all_list[组数];
    退火.parameters = Some(降温时间表 {
        t_max,
        t_min,
        steps: 1_000_000,
    });
    let mut 线程池 = vec![];
    let 总线程数 = 9;
    for 线程序号 in 0..总线程数 {
        let 编码器 = 冰雪二拼编码器::新建(&数据)?;
        let 目标函数 = 冰雪二拼目标函数::新建(&数据)?;
        let 操作 = 冰雪二拼操作::新建(&数据);
        let mut 问题 = 优化问题::新建(数据.clone(), 编码器, 目标函数, 操作);
        let 优化方法 = 退火.clone();
        let 子命令行 = 命令行.生成子命令行(线程序号);
        let 线程 = spawn(move || 优化方法.优化(&mut 问题, &子命令行));
        线程池.push(线程);
    }
    let mut 优化结果列表 = vec![];
    for (线程序号, 线程) in 线程池.into_iter().enumerate() {
        优化结果列表.push((线程序号, 线程.join().unwrap()));
    }
    优化结果列表.sort_by(|a, b| a.1.分数.partial_cmp(&b.1.分数).unwrap());
    let mut 总结文件 = File::create(命令行.输出目录.join("总结.txt"))?;
    write!(总结文件, "t_max: {}, t_min: {}", t_max, t_min)?;
    write!(总结文件, "最优解: {:?}", 优化结果列表[0].1.分数)?;
    Ok(())
}
