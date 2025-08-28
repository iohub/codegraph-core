#!/usr/bin/env python3
"""
测试层次化函数调用关系图API的脚本
"""

import requests
import json
import sys

def test_hierarchical_graph_api():
    """测试层次化图API"""
    base_url = "http://localhost:3000"
    
    # 测试1: 获取默认的层次化结构
    print("=== 测试1: 获取默认的层次化结构 ===")
    request_data = {
        "project_id": None,  # 使用第一个可用项目
        "root_function": None,  # 不指定根函数
        "max_depth": 3,
        "include_file_info": True
    }
    
    try:
        response = requests.post(
            f"{base_url}/query_hierarchical_graph",
            json=request_data,
            headers={"Content-Type": "application/json"}
        )
        
        if response.status_code == 200:
            result = response.json()
            print(f"✅ 成功获取层次化图")
            print(f"项目ID: {result['data']['project_id']}")
            print(f"总函数数: {result['data']['total_functions']}")
            print(f"总关系数: {result['data']['total_relations']}")
            print(f"最大深度: {result['data']['max_depth']}")
            
            # 打印树结构
            print_tree_structure(result['data']['tree_structure'], 0)
        else:
            print(f"❌ 请求失败: {response.status_code}")
            print(f"错误信息: {response.text}")
            
    except requests.exceptions.RequestException as e:
        print(f"❌ 请求异常: {e}")
    
    print("\n" + "="*50 + "\n")
    
    # 测试2: 从特定函数开始的层次化结构
    print("=== 测试2: 从特定函数开始的层次化结构 ===")
    request_data = {
        "project_id": None,
        "root_function": "main",  # 指定根函数为main
        "max_depth": 4,
        "include_file_info": True
    }
    
    try:
        response = requests.post(
            f"{base_url}/query_hierarchical_graph",
            json=request_data,
            headers={"Content-Type": "application/json"}
        )
        
        if response.status_code == 200:
            result = response.json()
            print(f"✅ 成功获取从main函数开始的层次化图")
            print(f"根函数: {result['data']['root_function']}")
            print(f"总函数数: {result['data']['total_functions']}")
            print(f"总关系数: {result['data']['total_relations']}")
            
            # 打印树结构
            print_tree_structure(result['data']['tree_structure'], 0)
        else:
            print(f"❌ 请求失败: {response.status_code}")
            print(f"错误信息: {response.text}")
            
    except requests.exceptions.RequestException as e:
        print(f"❌ 请求异常: {e}")

def print_tree_structure(node, depth):
    """递归打印树结构"""
    indent = "  " * depth
    
    # 打印节点信息
    if node.get('function_id'):
        # 函数节点
        print(f"{indent}├── {node['name']} [function]")
        if node.get('file_path'):
            print(f"{indent}│   📁 {node['file_path']}")
        if node.get('line_start') and node.get('line_end'):
            print(f"{indent}│   📍 行 {node['line_start']}-{node['line_end']}")
    else:
        # 目录或项目节点
        print(f"{indent}├── {node['name']}")
    
    # 递归打印子节点
    for i, child in enumerate(node.get('children', [])):
        if i == len(node.get('children', [])) - 1:
            # 最后一个子节点
            print_tree_structure(child, depth + 1)
        else:
            print_tree_structure(child, depth + 1)

def print_usage():
    """打印使用说明"""
    print("使用方法:")
    print("  python3 test_hierarchical_api.py")
    print("")
    print("确保CodeGraph HTTP服务器正在运行在 http://localhost:3000")
    print("")

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] in ["-h", "--help"]:
        print_usage()
        sys.exit(0)
    
    print("🚀 开始测试层次化函数调用关系图API")
    print("="*50)
    
    test_hierarchical_graph_api()
    
    print("\n✅ 测试完成!") 