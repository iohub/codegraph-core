
package com.example.calculator;

import java.util.List;
import java.util.ArrayList;
import java.util.stream.Collectors;

/**
 * 简单的计算器类
 */
public class Calculator {
    private double result;
    private List<String> history;
    
    public Calculator() {
        this.result = 0.0;
        this.history = new ArrayList<>();
    }
    
    /**
     * 加法运算
     */
    public double add(double a, double b) {
        this.result = a + b;
        this.history.add(String.format("add(%.2f, %.2f) = %.2f", a, b, this.result));
        return this.result;
    }
    
    /**
     * 减法运算
     */
    public double subtract(double a, double b) {
        this.result = a - b;
        this.history.add(String.format("subtract(%.2f, %.2f) = %.2f", a, b, this.result));
        return this.result;
    }
    
    /**
     * 乘法运算
     */
    public double multiply(double a, double b) {
        this.result = a * b;
        this.history.add(String.format("multiply(%.2f, %.2f) = %.2f", a, b, this.result));
        return this.result;
    }
    
    /**
     * 除法运算
     */
    public double divide(double a, double b) throws ArithmeticException {
        if (b == 0) {
            throw new ArithmeticException("Division by zero");
        }
        this.result = a / b;
        this.history.add(String.format("divide(%.2f, %.2f) = %.2f", a, b, this.result));
        return this.result;
    }
    
    /**
     * 获取计算历史
     */
    public List<String> getHistory() {
        return new ArrayList<>(this.history);
    }
    
    /**
     * 清空历史记录
     */
    public void clearHistory() {
        this.history.clear();
    }
    
    /**
     * 主方法
     */
    public static void main(String[] args) {
        Calculator calc = new Calculator();
        
        try {
            // 执行一些计算
            double sum = calc.add(10.5, 5.3);
            System.out.println("Sum: " + sum);
            
            double difference = calc.subtract(20.0, 8.5);
            System.out.println("Difference: " + difference);
            
            double product = calc.multiply(4.0, 6.0);
            System.out.println("Product: " + product);
            
            double quotient = calc.divide(15.0, 3.0);
            System.out.println("Quotient: " + quotient);
            
            // 显示计算历史
            List<String> history = calc.getHistory();
            System.out.println("\nCalculation History:");
            history.forEach(System.out::println);
            
        } catch (ArithmeticException e) {
            System.err.println("Error: " + e.getMessage());
        }
    }
}

/**
 * 计算器接口
 */
interface MathOperation {
    double execute(double a, double b);
}

/**
 * 高级计算器类
 */
class AdvancedCalculator extends Calculator implements MathOperation {
    private static final double PI = 3.14159;
    
    @Override
    public double execute(double a, double b) {
        return add(a, b);
    }
    
    /**
     * 计算圆的面积
     */
    public double circleArea(double radius) {
        return PI * radius * radius;
    }
    
    /**
     * 计算幂运算
     */
    public double power(double base, double exponent) {
        return Math.pow(base, exponent);
    }
}

/**
 * 计算器工厂类
 */
class CalculatorFactory {
    public static Calculator createCalculator(String type) {
        switch (type.toLowerCase()) {
            case "basic":
                return new Calculator();
            case "advanced":
                return new AdvancedCalculator();
            default:
                throw new IllegalArgumentException("Unknown calculator type: " + type);
        }
    }
}
