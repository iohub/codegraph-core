#!/usr/bin/env python3
"""
Test Python file for code analysis
"""

import os
import sys
from typing import List, Dict, Optional
from dataclasses import dataclass

@dataclass
class TestData:
    """Test data class"""
    name: str
    value: int
    items: List[str]

def simple_function():
    """Simple function without parameters"""
    print("Hello, World!")
    return True

def function_with_params(name: str, age: int = 18) -> str:
    """Function with parameters and return type"""
    result = f"Name: {name}, Age: {age}"
    print(result)
    return result

def function_with_decorators():
    """Function with decorators"""
    @dataclass
    class LocalClass:
        x: int
        y: int
    
    return LocalClass(1, 2)

class TestClass:
    """Test class for analysis"""
    
    def __init__(self, name: str):
        self.name = name
        self.data = []
    
    def add_item(self, item: str) -> None:
        """Add item to the list"""
        self.data.append(item)
        print(f"Added {item} to {self.name}")
    
    def get_items(self) -> List[str]:
        """Get all items"""
        return self.data.copy()
    
    @classmethod
    def create_default(cls) -> 'TestClass':
        """Class method to create default instance"""
        return cls("default")
    
    @staticmethod
    def validate_name(name: str) -> bool:
        """Static method to validate name"""
        return len(name) > 0

class InheritedClass(TestClass):
    """Class that inherits from TestClass"""
    
    def __init__(self, name: str, extra: str):
        super().__init__(name)
        self.extra = extra
    
    def get_extra(self) -> str:
        return self.extra

def main():
    """Main function"""
    # Create instances
    obj1 = TestClass("test1")
    obj2 = InheritedClass("test2", "extra_value")
    
    # Call methods
    obj1.add_item("item1")
    obj1.add_item("item2")
    
    # Function calls
    simple_function()
    result = function_with_params("Alice", 25)
    
    # List comprehensions
    numbers = [1, 2, 3, 4, 5]
    squares = [x**2 for x in numbers if x % 2 == 0]
    
    # Exception handling
    try:
        value = int("abc")
    except ValueError as e:
        print(f"Error: {e}")
    except Exception as e:
        print(f"Unexpected error: {e}")
    finally:
        print("Cleanup completed")
    
    # Dictionary operations
    config = {
        "debug": True,
        "port": 8080,
        "host": "localhost"
    }
    
    for key, value in config.items():
        print(f"{key}: {value}")
    
    return 0

if __name__ == "__main__":
    sys.exit(main()) 