use chai::data::{元素, 数据};
use std::collections::HashMap;
pub mod snow2encoder;
pub use snow2encoder::冰雪双拼编码器;
pub mod snow2objective;
pub use snow2objective::冰雪双拼目标函数;
pub mod snow2operators;
pub use snow2operators::冰雪双拼操作;
pub mod snow4operators;
pub use snow4operators::冰雪四拼操作;
pub mod dual;
pub mod tree;

pub struct 冰雪双拼元素分类 {
    pub 声母列表: Vec<元素>,
    pub 韵母列表: Vec<元素>,
    pub 韵部列表: Vec<[元素; 4]>, // 《中华通韵》中的韵部
    pub 字根列表: Vec<元素>,
}

impl 冰雪双拼元素分类 {
    pub fn 新建(数据: &数据) -> Self {
        let mut 声母列表 = vec![];
        let mut 韵母列表 = vec![];
        let mut 韵部映射 = HashMap::new();
        let mut 字根列表 = vec![];
        for 元素 in (数据.进制 as usize)..数据.初始映射.len() {
            let 元素名 = &数据.数字转元素[&元素];
            if 元素名.starts_with("冰声") {
                let 声母名 = 元素名[7..].to_string();
                if ["z", "c", "s", "零开", "零合", "零齐", "零撮"].contains(&声母名.as_str())
                {
                    声母列表.push(元素);
                }
            } else if 元素名.starts_with("冰韵") {
                韵母列表.push(元素);
                let 字符列表: Vec<char> = 元素名.chars().collect();
                let 声调 = 字符列表[字符列表.len() - 1].to_digit(10).unwrap() - 1;
                let 无声调韵母: String = 字符列表[..(字符列表.len() - 1)].iter().collect();
                韵部映射.entry(无声调韵母).or_insert([元素::default(); 4])[声调 as usize] = 元素;
            } else {
                字根列表.push(元素);
            }
        }
        let 韵部列表: Vec<_> = 韵部映射.values().cloned().collect();
        Self {
            声母列表,
            韵母列表,
            韵部列表,
            字根列表,
        }
    }
}
