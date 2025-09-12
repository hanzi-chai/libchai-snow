pub mod common;
pub mod qingyun;
pub mod snow2;
pub mod snow4;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

/// 每个代码片段的统计信息
#[derive(Debug, Default)]
struct Stat {
    count: usize,
    total: Duration,
}

impl Stat {
    fn add(&mut self, dur: Duration) {
        self.count += 1;
        self.total += dur;
    }

    fn average(&self) -> Duration {
        if self.count == 0 {
            Duration::new(0, 0)
        } else {
            self.total / (self.count as u32)
        }
    }
}

/// 全局计时器管理器
pub struct Timer {
    data: Mutex<HashMap<String, Stat>>,
}

impl Timer {
    fn new() -> Self {
        Timer {
            data: Mutex::new(HashMap::new()),
        }
    }

    fn add(&self, name: &str, dur: Duration) {
        let mut data = self.data.lock().unwrap();
        data.entry(name.to_string()).or_default().add(dur);
    }

    /// 打印汇总表
    pub fn report(&self) {
        let data = self.data.lock().unwrap();
        println!(
            "{} {} {} {}",
            Self::pad_str("名称", 15),
            Self::pad_str("计数", 10),
            Self::pad_str("总时间 (s)", 15),
            Self::pad_str("平均时间 (μs)", 15),
        );
        println!("{}", "-".repeat(65));
        for (name, stat) in data.iter() {
            println!(
                "{} {:>10} {:>15.3} {:>15.3}",
                Self::pad_str(name, 15),
                stat.count,
                stat.total.as_secs_f64(),
                stat.average().as_secs_f64() * 1e6,
            );
        }
    }

    pub fn pad_str(s: &str, width: usize) -> String {
        let w = UnicodeWidthStr::width(s);
        if w >= width {
            s.to_string()
        } else {
            format!("{}{}", s, " ".repeat(width - w))
        }
    }
}

/// 全局对象
pub static TIMER: Lazy<Timer> = Lazy::new(Timer::new);

/// 宏：计时代码块
#[macro_export]
macro_rules! time_block {
    ($name:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = { $block };
        let elapsed = start.elapsed();
        $crate::TIMER.add($name, elapsed);
        result
    }};
}
