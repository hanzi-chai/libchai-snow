use chai::config::SolverConfig;
use chai::interfaces::command_line::{
    从命令行参数创建, 命令, 命令行, 默认命令行参数
};
use chai::objectives::目标函数;
use chai::错误;
use clap::Parser;
use snow::qingyun::context::冰雪清韵上下文;
use snow::qingyun::encoder::冰雪清韵编码器;
use snow::qingyun::objective::冰雪清韵目标函数;
use snow::qingyun::operators::冰雪清韵操作;
use std::fs::File;
use std::io::Write;
use std::thread::spawn;

fn main() -> Result<(), 错误> {
    let 参数 = 默认命令行参数::parse();
    let 输入 = 从命令行参数创建(&参数);
    let 命令行 = 命令行::新建(参数, None);
    let 上下文 = 冰雪清韵上下文::新建(输入)?;
    let _config = 上下文.配置.clone();
    match 命令行.参数.command {
        命令::Encode => {
            let 编码器 = 冰雪清韵编码器::新建(&上下文, true)?;
            let mut 目标函数 = 冰雪清韵目标函数::新建(&上下文, 编码器);
            let (指标, 分数) = 目标函数.计算(&上下文.初始决策, &None);
            println!("分数：{分数:.4}");
            let 码表 = 上下文.生成码表(&目标函数.编码结果);
            命令行.输出编码结果(码表);
            命令行.输出评测指标(指标);
        }
        命令::Optimize => {
            let 线程数 = 命令行.参数.threads.unwrap_or(1);
            let SolverConfig::SimulatedAnnealing(退火) =
                _config.optimization.unwrap().metaheuristic.unwrap();
            let mut 线程池 = vec![];
            for 线程序号 in 0..线程数 {
                let 编码器 = 冰雪清韵编码器::新建(&上下文, false)?;
                let mut 目标函数 = 冰雪清韵目标函数::新建(&上下文, 编码器);
                let mut 操作 = 冰雪清韵操作::新建(&上下文);
                let 优化方法 = 退火.clone();
                let 上下文 = 上下文.clone();
                let 子命令行 = 命令行.生成子命令行(线程序号);
                let 线程 = spawn(move || {
                    let 优化结果 = 优化方法.优化(
                        &上下文.初始决策,
                        &mut 目标函数,
                        &mut 操作,
                        &上下文,
                        &子命令行,
                    );
                    目标函数.计算(&优化结果.映射, &None);
                    let 码表 = 上下文.生成码表(&目标函数.编码结果);
                    let 分析路径 = 子命令行.输出目录.join("分析.md");
                    上下文
                        .分析码表(&目标函数.编码结果, &分析路径)
                        .expect("分析码表失败");
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
