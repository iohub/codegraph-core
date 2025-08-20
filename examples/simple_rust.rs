pub fn hello_world() {
    println!("Hello, world!");
}

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    add(a, b) * 2
}

pub fn main() {
    hello_world();
    let result = multiply(5, 3);
    println!("Result: {}", result);
} 