/**
 * TypeScript测试文件
 * 包含各种TypeScript语法结构用于测试分析器
 */

// 导入语句
import { Component, OnInit, Input, Output, EventEmitter } from '@angular/core';
import * as React from 'react';
import { useState, useEffect } from 'react';

// 类型定义
type UserRole = 'admin' | 'user' | 'guest';
type UserStatus = 'active' | 'inactive' | 'pending';

interface User {
    id: number;
    name: string;
    email: string;
    role: UserRole;
    status: UserStatus;
    createdAt: Date;
}

interface ApiResponse<T> {
    data: T;
    message: string;
    success: boolean;
}

// 联合类型和交叉类型
type AdminUser = User & { permissions: string[] };
type UserOrAdmin = User | AdminUser;

// 枚举定义
enum HttpStatus {
    OK = 200,
    CREATED = 201,
    BAD_REQUEST = 400,
    UNAUTHORIZED = 401,
    NOT_FOUND = 404,
    INTERNAL_SERVER_ERROR = 500
}

// 命名空间
namespace Utils {
    export function formatDate(date: Date): string {
        return date.toISOString();
    }
    
    export function generateId(): string {
        return Math.random().toString(36).substr(2, 9);
    }
}

// 装饰器
function log(target: any, propertyKey: string, descriptor: PropertyDescriptor) {
    const originalMethod = descriptor.value;
    descriptor.value = function(...args: any[]) {
        console.log(`Calling ${propertyKey} with args:`, args);
        const result = originalMethod.apply(this, args);
        console.log(`Result:`, result);
        return result;
    };
    return descriptor;
}

// 泛型函数
function identity<T>(arg: T): T {
    return arg;
}

function createArray<T>(length: number, value: T): T[] {
    return Array(length).fill(value);
}

// 函数声明
function greet(name: string): string {
    return `Hello, ${name}!`;
}

// 函数表达式
const multiply = (a: number, b: number): number => a * b;

// 箭头函数
const divide = (a: number, b: number): number => {
    if (b === 0) {
        throw new Error('Division by zero');
    }
    return a / b;
};

// 类定义
class UserService {
    private users: User[] = [];
    
    constructor(private apiUrl: string) {}
    
    @log
    async getUsers(): Promise<User[]> {
        const response = await fetch(`${this.apiUrl}/users`);
        const data: ApiResponse<User[]> = await response.json();
        return data.data;
    }
    
    async createUser(userData: Omit<User, 'id' | 'createdAt'>): Promise<User> {
        const newUser: User = {
            ...userData,
            id: parseInt(Utils.generateId()),
            createdAt: new Date()
        };
        
        this.users.push(newUser);
        return newUser;
    }
    
    static getInstance(): UserService {
        return new UserService('/api');
    }
}

// 继承类
class AdminService extends UserService {
    constructor(apiUrl: string, private adminToken: string) {
        super(apiUrl);
    }
    
    async getAdminUsers(): Promise<AdminUser[]> {
        const users = await this.getUsers();
        return users.filter(user => user.role === 'admin') as AdminUser[];
    }
}

// 实现接口的类
class UserRepository implements UserService {
    private users: User[] = [];
    
    constructor(private apiUrl: string) {}
    
    async getUsers(): Promise<User[]> {
        return this.users;
    }
    
    async createUser(userData: Omit<User, 'id' | 'createdAt'>): Promise<User> {
        const newUser: User = {
            ...userData,
            id: this.users.length + 1,
            createdAt: new Date()
        };
        
        this.users.push(newUser);
        return newUser;
    }
}

// 抽象类
abstract class BaseService {
    protected abstract getBaseUrl(): string;
    
    protected async request<T>(endpoint: string): Promise<T> {
        const url = `${this.getBaseUrl()}${endpoint}`;
        const response = await fetch(url);
        return response.json();
    }
}

// 具体实现
class ApiService extends BaseService {
    protected getBaseUrl(): string {
        return 'https://api.example.com';
    }
    
    async getData<T>(endpoint: string): Promise<T> {
        return this.request<T>(endpoint);
    }
}

// 模块导出
export { UserService, AdminService, UserRepository, ApiService };
export type { User, AdminUser, UserOrAdmin, ApiResponse };
export { Utils, HttpStatus };

// 默认导出
export default UserService;

// 异步函数
async function main(): Promise<void> {
    try {
        const userService = new UserService('/api/users');
        const users = await userService.getUsers();
        
        console.log('Users:', users);
        
        const newUser = await userService.createUser({
            name: 'John Doe',
            email: 'john@example.com',
            role: 'user',
            status: 'active'
        });
        
        console.log('New user created:', newUser);
        
        // 使用泛型函数
        const numbers = createArray(5, 0);
        const strings = createArray(3, 'hello');
        
        console.log('Numbers:', numbers);
        console.log('Strings:', strings);
        
        // 使用工具函数
        const formattedDate = Utils.formatDate(new Date());
        console.log('Formatted date:', formattedDate);
        
    } catch (error) {
        console.error('Error:', error);
    }
}

// 条件执行
if (require.main === module) {
    main();
} 