// 简单的Rust测试文件
use std::collections::HashMap;

/// 简单的计算器结构体
pub struct Calculator {
    value: i32,
}

impl Calculator {
    /// 创建新的计算器实例
    pub fn new() -> Self {
        Calculator { value: 0 }
    }
    
    /// 添加数字
    pub fn add(&mut self, x: i32) -> i32 {
        self.value += x;
        self.value
    }
    
    /// 获取当前值
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

/// 数学运算trait
pub trait MathOperations {
    fn multiply(&self, x: i32) -> i32;
    fn divide(&self, x: i32) -> Result<i32, String>;
}

impl MathOperations for Calculator {
    fn multiply(&self, x: i32) -> i32 {
        self.value * x
    }
    
    fn divide(&self, x: i32) -> Result<i32, String> {
        if x == 0 {
            Err("Division by zero".to_string())
        } else {
            Ok(self.value / x)
        }
    }
}

/// 主函数
fn main() {
    let mut calc = Calculator::new();
    let result = calc.add(5);
    println!("Result: {}", result);
    
    let multiplied = calc.multiply(2);
    println!("Multiplied: {}", multiplied);
    
    match calc.divide(2) {
        Ok(result) => println!("Divided: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}

/// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculator_new() {
        let calc = Calculator::new();
        assert_eq!(calc.get_value(), 0);
    }
    
    #[test]
    fn test_calculator_add() {
        let mut calc = Calculator::new();
        assert_eq!(calc.add(5), 5);
        assert_eq!(calc.add(3), 8);
    }
    
    #[test]
    fn test_math_operations() {
        let mut calc = Calculator::new();
        calc.add(10);
        assert_eq!(calc.multiply(2), 20);
        assert_eq!(calc.divide(2), Ok(5));
    }
} 