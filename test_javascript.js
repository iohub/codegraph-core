/**
 * JavaScript测试文件
 * 包含各种JavaScript语法结构用于测试分析器
 */

// 导入语句 (ES6模块)
import { Component, useState, useEffect } from 'react';
import * as React from 'react';
import { createStore } from 'redux';

// 函数声明
function greet(name) {
    return `Hello, ${name}!`;
}

// 函数表达式
const multiply = function(a, b) {
    return a * b;
};

// 箭头函数
const divide = (a, b) => {
    if (b === 0) {
        throw new Error('Division by zero');
    }
    return a / b;
};

// 异步函数
async function fetchData(url) {
    try {
        const response = await fetch(url);
        const data = await response.json();
        return data;
    } catch (error) {
        console.error('Error fetching data:', error);
        throw error;
    }
}

// 生成器函数
function* fibonacci() {
    let a = 0, b = 1;
    while (true) {
        yield a;
        [a, b] = [b, a + b];
    }
}

// 类定义
class UserService {
    constructor(apiUrl) {
        this.apiUrl = apiUrl;
        this.users = [];
    }
    
    async getUsers() {
        const response = await fetch(`${this.apiUrl}/users`);
        const data = await response.json();
        return data;
    }
    
    async createUser(userData) {
        const newUser = {
            ...userData,
            id: this.users.length + 1,
            createdAt: new Date()
        };
        
        this.users.push(newUser);
        return newUser;
    }
    
    static getInstance() {
        return new UserService('/api');
    }
}

// 继承类
class AdminService extends UserService {
    constructor(apiUrl, adminToken) {
        super(apiUrl);
        this.adminToken = adminToken;
    }
    
    async getAdminUsers() {
        const users = await this.getUsers();
        return users.filter(user => user.role === 'admin');
    }
}

// 对象字面量
const utils = {
    formatDate(date) {
        return date.toISOString();
    },
    
    generateId() {
        return Math.random().toString(36).substr(2, 9);
    },
    
    async delay(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
};

// 对象方法简写
const calculator = {
    add(a, b) {
        return a + b;
    },
    
    subtract(a, b) {
        return a - b;
    },
    
    multiply(a, b) {
        return a * b;
    },
    
    divide(a, b) {
        if (b === 0) {
            throw new Error('Division by zero');
        }
        return a / b;
    }
};

// 立即执行函数表达式 (IIFE)
const counter = (function() {
    let count = 0;
    
    return {
        increment() {
            count++;
        },
        decrement() {
            count--;
        },
        getCount() {
            return count;
        }
    };
})();

// 高阶函数
function withLogging(fn) {
    return function(...args) {
        console.log(`Calling function with args:`, args);
        const result = fn.apply(this, args);
        console.log(`Function returned:`, result);
        return result;
    };
}

// 装饰器函数
function log(target, propertyKey, descriptor) {
    const originalMethod = descriptor.value;
    descriptor.value = function(...args) {
        console.log(`Calling ${propertyKey} with args:`, args);
        const result = originalMethod.apply(this, args);
        console.log(`Result:`, result);
        return result;
    };
    return descriptor;
}

// 使用装饰器的类
class LoggedService {
    @log
    async fetchData(url) {
        const response = await fetch(url);
        return response.json();
    }
}

// 模块导出
export { UserService, AdminService, utils, calculator };
export default UserService;

// 默认导出
export default class ApiClient {
    constructor(baseUrl) {
        this.baseUrl = baseUrl;
    }
    
    async request(endpoint, options = {}) {
        const url = `${this.baseUrl}${endpoint}`;
        const response = await fetch(url, options);
        return response.json();
    }
}

// 条件执行
if (typeof module !== 'undefined' && module.exports) {
    // Node.js环境
    module.exports = {
        UserService,
        AdminService,
        utils,
        calculator
    };
}

// 异步主函数
async function main() {
    try {
        const userService = new UserService('/api/users');
        const users = await userService.getUsers();
        
        console.log('Users:', users);
        
        const newUser = await userService.createUser({
            name: 'John Doe',
            email: 'john@example.com',
            role: 'user'
        });
        
        console.log('New user created:', newUser);
        
        // 使用工具函数
        const formattedDate = utils.formatDate(new Date());
        console.log('Formatted date:', formattedDate);
        
        // 使用计算器
        const result = calculator.add(5, 3);
        console.log('5 + 3 =', result);
        
        // 使用高阶函数
        const loggedGreet = withLogging(greet);
        loggedGreet('World');
        
        // 使用生成器
        const fib = fibonacci();
        console.log('First 10 Fibonacci numbers:');
        for (let i = 0; i < 10; i++) {
            console.log(fib.next().value);
        }
        
    } catch (error) {
        console.error('Error:', error);
    }
}

// 条件执行
if (typeof window !== 'undefined') {
    // 浏览器环境
    window.main = main;
} else if (typeof global !== 'undefined') {
    // Node.js环境
    global.main = main;
} 