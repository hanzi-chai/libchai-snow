use rustc_hash::FxHashMap;
use serde_yaml::from_str;
use std::fs::read_to_string;

/// 定义通用的转换 trait

pub trait 转换 {
    fn hash(&self) -> usize;
    fn 编码空间大小() -> usize;
}

pub fn get_pua_mapper() -> FxHashMap<char, char> {
    from_str(&read_to_string("pua_mapper.yaml").unwrap()).unwrap()
}
