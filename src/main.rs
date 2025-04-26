use chai::config::SolverConfig;
use chai::encoders::编码器;
use chai::objectives::目标函数;
use chai::optimizers::{优化方法, 优化问题};
use chai::{命令, 命令行, 命令行参数, 错误};
use clap::Parser;
use snow::snow2objective::冰雪双拼目标函数;
use snow::snow2operators::冰雪双拼操作;
use snow::冰雪双拼编码器;
use std::fs::File;
use std::io::Write;
use std::thread::spawn;

fn main() -> Result<(), 错误> {
    let 参数 = 命令行参数::parse();
    let 命令行 = 命令行::新建(参数, None);
    let mut 数据 = 命令行.准备数据();
    数据.词列表.sort_by_key(|词| 词.词长);
    数据.词列表.iter_mut().for_each(|词| {
        if 词.词长 == 1 {
            词.词长 = 0;
        }
    });
    let _config = 数据.配置.clone();
    match 命令行.参数.command {
        命令::Encode => {
            let mut 编码器 = 冰雪双拼编码器::新建(&数据)?;
            let mut 目标函数 = 冰雪双拼目标函数::新建(&数据)?;
            let mut 编码结果 = 编码器.编码(&数据.初始映射, &None).clone();
            let 码表 = 数据.生成码表(&编码结果);
            let (指标, _) = 目标函数.计算(&mut 编码结果, &数据.初始映射);
            命令行.输出编码结果(码表);
            命令行.输出评测指标(指标);
        }
        命令::Optimize => {
            let 线程数 = 命令行.参数.threads.unwrap_or(1);
            let SolverConfig::SimulatedAnnealing(退火) =
                _config.optimization.unwrap().metaheuristic.unwrap();
            let mut 线程池 = vec![];
            for 线程序号 in 0..线程数 {
                let 编码器 = 冰雪双拼编码器::新建(&数据)?;
                let 目标函数 = 冰雪双拼目标函数::新建(&数据)?;
                let 操作 = 冰雪双拼操作::新建(&数据);
                let mut 问题 = 优化问题::新建(数据.clone(), 编码器, 目标函数, 操作);
                let 优化方法 = 退火.clone();
                let 数据 = 数据.clone();
                let 子命令行 = 命令行.生成子命令行(线程序号);
                let 线程 = spawn(move || {
                    let 优化结果 = 优化方法.优化(&mut 问题, &子命令行);
                    let 编码结果 = 问题.编码器.编码(&优化结果.映射, &None);
                    let 码表 = 数据.生成码表(&编码结果);
                    子命令行.输出编码结果(码表);
                    return 优化结果;
                });
                线程池.push(线程);
            }
            let mut 优化结果列表 = vec![];
            for (线程序号, 线程) in 线程池.into_iter().enumerate() {
                优化结果列表.push((线程序号, 线程.join().unwrap()));
            }
            优化结果列表.sort_by(|a, b| a.1.分数.partial_cmp(&b.1.分数).unwrap());
            let mut 总结文件 = File::create(命令行.输出目录.join("总结.txt"))?;
            for (线程序号, 优化结果) in 优化结果列表 {
                print!(
                    "线程 {} 分数：{:.4}；{}",
                    线程序号, 优化结果.分数, 优化结果.指标
                );
                write!(
                    总结文件,
                    "线程 {} 分数：{:.4}；{}",
                    线程序号, 优化结果.分数, 优化结果.指标
                )?;
            }
        }
    }
    Ok(())
}
