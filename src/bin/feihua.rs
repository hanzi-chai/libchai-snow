use clap::{Args, Parser, Subcommand};
use snow::feihua::{上下文, 退火参数, 错误, 预算};
use std::{path::PathBuf, process::Command, time::Instant};

#[derive(Parser)]
#[command(about = "冰雪飞花：增量编码与多线程模拟退火")]
struct 参数 {
    /// 使用已有自包含输入；省略时先全量执行 prepare.ts
    #[arg(long = "tree-data", global = true)]
    输入: Option<PathBuf>,
    #[command(subcommand)]
    命令: 命令,
}
#[derive(Subcommand)]
enum 命令 {
    #[command(name = "encode", about = "计算指标并导出码表与决策")]
    编码 {
        #[arg(long = "solution")]
        决策: Option<PathBuf>,
        #[arg(long = "out", default_value = "output/feihua-encoding")]
        输出: PathBuf,
        #[command(flatten)]
        预算: 预算参数,
    },
    #[command(name = "optimize", about = "独立多线程退火，保存各线程及全局最优结果")]
    优化(优化参数),
    #[command(name = "bench", about = "运行真实退火循环测量步耗时，默认不写产物")]
    基准(优化参数),
}
#[derive(Args)]
struct 预算参数 {
    #[arg(long = "b-budget", default_value_t = 50)]
    小集合: usize,
    #[arg(long = "complement-budget", default_value_t = 30)]
    补集合: usize,
    #[arg(long = "budget-penalty", default_value_t = 1.0)]
    惩罚: f64,
}
impl 预算参数 {
    fn 转换(&self) -> 预算 {
        预算 {
            小集合: self.小集合,
            补集合: self.补集合,
            惩罚: self.惩罚,
        }
    }
}
#[derive(Args)]
struct 优化参数 {
    #[arg(short = 't', long = "threads", default_value_t = 1)]
    线程: usize,
    /// 每个线程的步数；默认采用配置中的 steps
    #[arg(long = "steps")]
    步数: Option<usize>,
    #[arg(long = "t-max")]
    最高温: Option<f64>,
    #[arg(long = "t-min")]
    最低温: Option<f64>,
    #[arg(long = "seed", default_value_t = 42)]
    种子: u64,
    #[arg(long = "report-every", default_value_t = 10_000)]
    报告间隔: usize,
    /// 每隔若干尝试步全量核对增量缓存；0 表示仅在结束时核对
    #[arg(long = "verify-every", default_value_t = 0)]
    校验间隔: usize,
    /// 从同一 input.json 保存的 solution.json 或 checkpoint.json 开始新的退火段
    #[arg(long = "solution")]
    决策: Option<PathBuf>,
    #[arg(long = "out")]
    输出: Option<PathBuf>,
    #[command(flatten)]
    预算: 预算参数,
}
fn main() -> Result<(), 错误> {
    let 参数 = 参数::parse();
    let 输入 = if let Some(路径) = 参数.输入 {
        路径
    } else {
        let 退出 = Command::new("bun").arg("src/feihua/prepare.ts").status()?;
        if !退出.success() {
            return Err(std::io::Error::other("飞花预处理失败，未读取旧产物").into());
        }
        PathBuf::from("output/feihua/input.json")
    };
    let 开始 = Instant::now();
    let 上下文 = 上下文::读取(&输入)?;
    println!("载入输入：{:.3} 秒", 开始.elapsed().as_secs_f64());
    let 是基准 = matches!(&参数.命令, 命令::基准(_));
    match 参数.命令 {
        命令::编码 {
            决策, 输出, 预算
        } => {
            let 决策 = match 决策 {
                Some(路径) => 上下文.读取决策(&路径)?,
                None => 上下文.初始决策(),
            };
            let 指标 = 上下文.编码(决策, &输出, 预算.转换())?;
            println!(
                "重码 {}，B 组数 {}，补集合例外组数 {}，分数 {:.3}；输出 {}",
                指标.重码,
                指标.小集合组数,
                指标.补集合例外组数,
                指标.分数,
                输出.display()
            );
        }
        命令::优化(选项) | 命令::基准(选项) => {
            let 默认 = 上下文.默认参数();
            let 运行 = 退火参数 {
                线程: 选项.线程,
                步数: 选项.步数.unwrap_or(默认.0),
                最高温: 选项.最高温.unwrap_or(默认.1),
                最低温: 选项.最低温.unwrap_or(默认.2),
                种子: 选项.种子,
                报告间隔: 选项.报告间隔,
                校验间隔: 选项.校验间隔,
                预算: 选项.预算.转换(),
            };
            let 决策 = match 选项.决策 {
                Some(路径) => 上下文.读取决策(&路径)?,
                None => 上下文.初始决策(),
            };
            let 输出 = 选项.输出.or_else(|| {
                (!是基准).then(|| {
                    PathBuf::from(format!(
                        "output/feihua-optimization/{}",
                        chrono::Local::now().format("%Y%m%d-%H%M%S-%3f")
                    ))
                })
            });
            let 开始 = Instant::now();
            let 结果 = 上下文.并行退火(&运行, &决策, 输出.as_deref())?;
            for 条 in &结果 {
                println!(
                    "线程 {}：重码 {}，B {}，补例外 {}，分数 {:.3}；{:.2} μs/尝试，{:.2} μs/有效评估；有效 {}/{}，接受 {}，加根 {}",
                    条.线程,
                    条.最优指标.重码,
                    条.最优指标.小集合组数,
                    条.最优指标.补集合例外组数,
                    条.最优指标.分数,
                    条.每步微秒,
                    条.每有效步微秒,
                    条.有效步数,
                    条.步数,
                    条.接受步数,
                    条.添加字根
                );
            }
            println!(
                "并行总耗时 {:.3} 秒，目标 100 μs/有效评估",
                开始.elapsed().as_secs_f64()
            );
            if let Some(目录) = 输出 {
                println!("结果目录：{}", 目录.display());
            }
        }
    }
    Ok(())
}
