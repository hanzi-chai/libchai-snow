use std::{collections::HashMap, fs::read_to_string};

use chai::data::{元素, 数据};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct 原始双编码 {
    字根: String,
    第一码: String,
    第二码: String,
}

pub struct 双编码 {
    pub 字根: 元素,
    pub 第一码: 元素,
    pub 第二码: 元素,
}

pub fn 构建双编码映射(数据: &数据) -> HashMap<元素, (元素, 元素)> {
    let 内容 = read_to_string("dual.yaml").unwrap();
    let 原始双编码列表: Vec<原始双编码> = serde_yaml::from_str(&内容).unwrap();
    let mut 双编码映射 = HashMap::new();

    for 原始双编码 in 原始双编码列表 {
        let 字根 = 数据.元素转数字[原始双编码.字根.as_str()];
        let 第一码 = 数据.元素转数字[原始双编码.第一码.as_str()];
        let 第二码 = 数据.元素转数字[原始双编码.第二码.as_str()];
        双编码映射.insert(字根, (第一码, 第二码));
    }
    双编码映射
}
