use chai::{元素, 元素映射};
use chai::contexts::default::默认上下文;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::read_to_string};

#[derive(Serialize, Deserialize)]
pub struct 原始字根树 {
    pub 字根: String,
    pub 孩子们: Vec<原始字根树>,
}

pub struct 字根树 {
    pub 字根: 元素,
    pub 孩子们: Vec<字根树>,
}

impl 字根树 {
    fn 查找元素(&self, 目标: &元素) -> Option<&字根树> {
        if self.字根 == *目标 {
            return Some(self);
        }
        for 孩子 in &self.孩子们 {
            let 结果 = 孩子.查找元素(目标);
            if 结果.is_some() {
                return 结果;
            }
        }
        None
    }

    // 获取所有后代
    pub fn 获取所有被代表字根(
        &self, 目标: &元素, 映射: &元素映射
    ) -> Vec<元素> {
        let 节点 = self.查找元素(目标).unwrap();
        let mut 结果 = Vec::new();
        字根树::递归获取后代(&mut 结果, 节点, 映射);
        结果
    }

    // 递归获取某个节点的所有后代
    fn 递归获取后代(结果: &mut Vec<元素>, 节点: &字根树, 映射: &元素映射) {
        结果.push(节点.字根);
        for 孩子 in &节点.孩子们 {
            if 映射[孩子.字根] == 映射[节点.字根] {
                字根树::递归获取后代(结果, 孩子, 映射);
            }
        }
    }
}

pub struct 字根树控制器 {
    pub 字根树: 字根树,
    pub 父映射: HashMap<元素, 元素>,
}

impl 字根树控制器 {
    pub fn 新建(数据: &默认上下文) -> Self {
        let 原始字根树内容 = read_to_string("tree.yaml").unwrap();
        let 原始字根树: 原始字根树 = serde_yaml::from_str(&原始字根树内容).unwrap();
        let mut 父映射 = HashMap::new();
        let 字根树 = Self::构建字根树和父映射(原始字根树, 数据, &mut 父映射);
        Self {
            字根树, 父映射
        }
    }

    pub fn 构建字根树和父映射(
        原始: 原始字根树,
        数据: &默认上下文,
        父映射: &mut HashMap<元素, 元素>,
    ) -> 字根树 {
        let 字根 = if 原始.字根 == "" {
            0
        } else {
            数据.棱镜.元素转数字[&原始.字根]
        };
        if 原始.孩子们.is_empty() {
            return 字根树 {
                字根,
                孩子们: vec![],
            };
        } else {
            let 孩子们: Vec<_> = 原始
                .孩子们
                .into_iter()
                .map(|孩子| Self::构建字根树和父映射(孩子, 数据, 父映射))
                .collect();
            for 孩子 in &孩子们 {
                父映射.insert(孩子.字根, 字根);
            }
            return 字根树 { 字根, 孩子们 };
        }
    }

    pub fn 查询字根是否被选取(&self, 映射: &元素映射, 字根: &元素) -> bool {
        let 父字根 = self.父映射.get(字根);
        if let Some(父字根) = 父字根 {
            let 父按键 = 映射[*父字根];
            let 当前按键 = 映射[*字根];
            return 父按键 != 当前按键;
        } else {
            return true;
        }
    }
}
