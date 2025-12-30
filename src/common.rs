/// 定义通用的转换 trait

pub trait 转换 {
    fn hash(&self) -> usize;
    fn 编码空间大小() -> usize;
}
