//! 简单的Rust代码示例
//! 
//! 这个文件展示了基本的Rust语法和结构，用于演示代码分析器的功能。

/// 简单的问候函数
pub fn hello_world() {
    println!("Hello, world!");
}

/// 加法运算函数
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// 乘法运算函数（使用加法实现）
pub fn multiply(a: i32, b: i32) -> i32 {
    add(a, b) * 2
}

/// 主函数 - 演示各种函数调用
pub fn main() {
    hello_world();
    let result = multiply(5, 3);
    println!("Result: {}", result);
    
    // 演示错误处理
    let numbers = vec![1, 2, 3, 4, 5];
    let sum: i32 = numbers.iter().sum();
    println!("Sum of numbers: {}", sum);
} 