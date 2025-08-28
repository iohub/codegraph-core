#!/usr/bin/env python3
"""
演示如何将层次化图API输出转换为方案一的格式
"""

import requests
import json

def get_hierarchical_graph(server_url="http://localhost:3000", root_function=None, max_depth=3):
    """获取层次化图数据"""
    request_data = {
        "root_function": root_function,
        "max_depth": max_depth,
        "include_file_info": True
    }
    
    try:
        response = requests.post(
            f"{server_url}/query_hierarchical_graph",
            json=request_data,
            headers={"Content-Type": "application/json"}
        )
        
        if response.status_code == 200:
            return response.json()['data']
        else:
            print(f"❌ API请求失败: {response.status_code}")
            return None
            
    except requests.exceptions.RequestException as e:
        print(f"❌ 请求异常: {e}")
        return None

def convert_to_scheme1_format(tree_structure, root_function_name=None):
    """将API输出转换为方案一的格式"""
    if root_function_name:
        # 根函数模式
        return convert_function_tree_to_scheme1(tree_structure, 0)
    else:
        # 项目结构模式
        return convert_project_tree_to_scheme1(tree_structure, 0)

def convert_function_tree_to_scheme1(node, depth):
    """将函数树转换为方案一格式"""
    indent = "│   " * depth
    
    if depth == 0:
        # 根节点
        result = f"{node['name']} [entry_point]\n"
    else:
        result = f"{indent}├── {node['name']}()\n"
    
    # 处理子节点
    for i, child in enumerate(node.get('children', [])):
        if i == len(node.get('children', [])) - 1:
            # 最后一个子节点
            result += convert_function_tree_to_scheme1(child, depth + 1)
        else:
            result += convert_function_tree_to_scheme1(child, depth + 1)
    
    return result

def convert_project_tree_to_scheme1(node, depth):
    """将项目树转换为方案一格式"""
    indent = "│   " * depth
    
    if depth == 0:
        # 根节点
        result = f"Function Call Hierarchy:\n"
    elif node.get('function_id'):
        # 函数节点
        result = f"{indent}├── {node['name']}()\n"
    else:
        # 文件/目录节点
        result = f"{indent}├── {node['name']}\n"
    
    # 处理子节点
    for i, child in enumerate(node.get('children', [])):
        if i == len(node.get('children', [])) - 1:
            # 最后一个子节点
            result += convert_project_tree_to_scheme1(child, depth + 1)
        else:
            result += convert_project_tree_to_scheme1(child, depth + 1)
    
    return result

def demo_scheme1_output():
    """演示方案一的输出格式"""
    print("🚀 演示方案一的函数调用关系图表达方式")
    print("="*60)
    
    # 1. 演示从特定函数开始的层次化结构
    print("\n📋 示例1: 从main函数开始的层次化结构")
    print("-" * 40)
    
    data = get_hierarchical_graph(root_function="main", max_depth=4)
    if data:
        scheme1_output = convert_to_scheme1_format(data['tree_structure'], "main")
        print(scheme1_output)
    
    # 2. 演示整个项目的层次化结构
    print("\n📋 示例2: 整个项目的层次化结构")
    print("-" * 40)
    
    data = get_hierarchical_graph(max_depth=3)
    if data:
        scheme1_output = convert_to_scheme1_format(data['tree_structure'])
        print(scheme1_output)
    
    # 3. 展示如何自定义输出
    print("\n📋 示例3: 自定义深度和根函数")
    print("-" * 40)
    
    data = get_hierarchical_graph(root_function="initialize_system", max_depth=2)
    if data:
        scheme1_output = convert_to_scheme1_format(data['tree_structure'], "initialize_system")
        print(scheme1_output)

def print_api_info():
    """打印API信息"""
    print("🔧 API端点信息:")
    print("  POST /query_hierarchical_graph")
    print("  支持参数:")
    print("    - root_function: 根函数名称")
    print("    - max_depth: 最大递归深度")
    print("    - include_file_info: 是否包含文件信息")
    print("")

if __name__ == "__main__":
    print_api_info()
    demo_scheme1_output()
    
    print("\n✅ 演示完成!")
    print("\n💡 提示:")
    print("  1. 确保CodeGraph HTTP服务器正在运行")
    print("  2. 可以根据需要调整max_depth参数")
    print("  3. 方案一的格式特别适合大模型理解") 