use std::path::PathBuf;
use std::fs;
use tree_sitter::{Parser, Language, QueryCursor};
use tree_sitter_java;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Updated Java Queries with Tree-sitter...");
    
    // 创建复杂的Java测试代码
    let java_code = r#"
package com.example.test;

import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;
import static java.util.Collections.emptyList;

/**
 * Complex test class with multiple methods and nested classes
 */
public class ComplexTestClass<T extends Comparable<T>> {
    
    private static final String DEFAULT_NAME = "default";
    private final String name;
    private final List<T> items;
    private Map<String, Object> config;
    
    /**
     * Constructor with parameters
     */
    public ComplexTestClass(String name, List<T> items) {
        this.name = name != null ? name : DEFAULT_NAME;
        this.items = items != null ? items : emptyList();
        this.config = new HashMap<>();
    }
    
    /**
     * Default constructor
     */
    public ComplexTestClass() {
        this(DEFAULT_NAME, null);
    }
    
    /**
     * Get the name of this instance
     */
    public String getName() {
        return this.name;
    }
    
    /**
     * Add item to the list
     */
    public void addItem(T item) {
        if (item != null) {
            this.items.add(item);
        }
    }
    
    /**
     * Get all items as a new list
     */
    public List<T> getItems() {
        return new ArrayList<>(this.items);
    }
    
    /**
     * Process items with stream operations
     */
    public List<T> processItems() {
        return this.items.stream()
            .filter(item -> item != null)
            .sorted()
            .collect(Collectors.toList());
    }
    
    /**
     * Set configuration value
     */
    public void setConfig(String key, Object value) {
        this.config.put(key, value);
    }
    
    /**
     * Get configuration value with default
     */
    @SuppressWarnings("unchecked")
    public <V> V getConfig(String key, V defaultValue) {
        return (V) this.config.getOrDefault(key, defaultValue);
    }
    
    /**
     * Static factory method
     */
    public static <E> ComplexTestClass<E> createDefault() {
        return new ComplexTestClass<>();
    }
    
    /**
     * Static method with generic type
     */
    public static <E> List<E> filterList(List<E> list, java.util.function.Predicate<E> predicate) {
        return list.stream()
            .filter(predicate)
            .collect(Collectors.toList());
    }
    
    /**
     * Method with exception handling
     */
    public void riskyOperation() throws IllegalArgumentException {
        if (this.items.isEmpty()) {
            throw new IllegalArgumentException("Items list cannot be empty");
        }
        // Some risky operation
    }
    
    /**
     * Method with multiple catch blocks
     */
    public void safeOperation() {
        try {
            riskyOperation();
        } catch (IllegalArgumentException e) {
            System.err.println("Invalid argument: " + e.getMessage());
        } catch (Exception e) {
            System.err.println("Unexpected error: " + e.getMessage());
        } finally {
            System.out.println("Operation completed");
        }
    }
    
    /**
     * Nested static class
     */
    public static class Builder<E> {
        private String name = DEFAULT_NAME;
        private List<E> items = new ArrayList<>();
        
        public Builder<E> name(String name) {
            this.name = name;
            return this;
        }
        
        public Builder<E> addItem(E item) {
            this.items.add(item);
            return this;
        }
        
        public ComplexTestClass<E> build() {
            return new ComplexTestClass<>(this.name, this.items);
        }
    }
    
    /**
     * Nested non-static class
     */
    public class InnerClass {
        private final String innerName;
        
        public InnerClass(String innerName) {
            this.innerName = innerName;
        }
        
        public String getInnerName() {
            return this.innerName;
        }
        
        public String getOuterName() {
            return ComplexTestClass.this.name;
        }
    }
    
    /**
     * Interface implementation
     */
    public interface TestInterface {
        void doSomething();
        String getResult();
    }
    
    /**
     * Enum definition
     */
    public enum TestEnum {
        VALUE1("value1"),
        VALUE2("value2"),
        VALUE3("value3");
        
        private final String value;
        
        TestEnum(String value) {
            this.value = value;
        }
        
        public String getValue() {
            return this.value;
        }
    }
}

/**
 * Record class example
 */
public record PersonRecord(
    String name,
    int age,
    List<String> hobbies
) {
    public PersonRecord {
        if (name == null || name.trim().isEmpty()) {
            throw new IllegalArgumentException("Name cannot be null or empty");
        }
        if (age < 0) {
            throw new IllegalArgumentException("Age cannot be negative");
        }
    }
    
    public boolean isAdult() {
        return age >= 18;
    }
}

/**
 * Another class in the same file
 */
class AnotherClass {
    private String data;
    
    public AnotherClass(String data) {
        this.data = data;
    }
    
    public String getData() {
        return this.data;
    }
    
    public void setData(String data) {
        this.data = data;
    }
    
    public void processData() {
        for (int i = 0; i < data.length(); i++) {
            char c = data.charAt(i);
            if (Character.isUpperCase(c)) {
                System.out.println("Found uppercase: " + c);
            }
        }
        
        int count = 0;
        while (count < 10) {
            if (count % 2 == 0) {
                System.out.println("Even: " + count);
            } else {
                System.out.println("Odd: " + count);
            }
            count++;
        }
        
        do {
            System.out.println("Processing...");
            count--;
        } while (count > 0);
    }
}
"#;
    
    // 创建测试文件
    let test_file_path = PathBuf::from("test_java_advanced.java");
    fs::write(&test_file_path, java_code)?;
    println!("✅ Created test file: {:?}", test_file_path);
    
    // 初始化tree-sitter解析器
    let mut parser = Parser::new();
    let language = tree_sitter_java::language();
    parser.set_language(language)?;
    
    // 解析代码
    let tree = parser.parse(java_code, None).ok_or("Failed to parse Java code")?;
    let root_node = tree.root_node();
    
    println!("✅ Parsed Java code successfully");
    println!("✅ Root node: {}", root_node.kind());
    
    // 测试各种查询
    test_queries(language, &root_node)?;
    
    // 清理测试文件
    if let Err(e) = fs::remove_file(&test_file_path) {
        println!("⚠️  Warning: Could not remove test file: {}", e);
    } else {
        println!("✅ Cleaned up test file");
    }
    
    println!("\n🎉 Java queries test completed successfully!");
    Ok(())
}

fn test_queries(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing Java Queries ===");
    
    // 测试包声明查询
    test_package_query(language, root_node)?;
    
    // 测试导入声明查询
    test_import_query(language, root_node)?;
    
    // 测试类定义查询
    test_class_query(language, root_node)?;
    
    // 测试方法定义查询
    test_method_query(language, root_node)?;
    
    // 测试构造函数查询
    test_constructor_query(language, root_node)?;
    
    // 测试变量声明查询
    test_variable_query(language, root_node)?;
    
    // 测试方法调用查询
    test_method_call_query(language, root_node)?;
    
    // 测试字段访问查询
    test_field_access_query(language, root_node)?;
    
    // 测试枚举查询
    test_enum_query(language, root_node)?;
    
    // 测试注解查询
    test_annotation_query(language, root_node)?;
    
    // 测试记录查询
    test_record_query(language, root_node)?;
    
    // 测试泛型查询
    test_generic_query(language, root_node)?;
    
    // 测试异常处理查询
    test_exception_query(language, root_node)?;
    
    // 测试循环语句查询
    test_loop_query(language, root_node)?;
    
    // 测试条件语句查询
    test_conditional_query(language, root_node)?;
    
    Ok(())
}

fn test_package_query(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query::new(
        language,
        r#"
        (package_declaration
          name: (_name) @package.name
        ) @package.decl
        "#,
    )?;
    
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, root_node.walk());
    
    let mut count = 0;
    for m in matches {
        count += 1;
        for capture in m.captures {
            let node = capture.node;
            let text = node.utf8_text(root_node.start_byte()..root_node.end_byte()).unwrap_or("");
            println!("✅ Package: {} at line {}", text, node.start_position().row);
        }
    }
    
    println!("✅ Package declarations found: {}", count);
    Ok(())
}

fn test_import_query(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query::new(
        language,
        r#"
        (import_declaration
          name: (_name) @import.name
        ) @import.decl
        "#,
    )?;
    
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, root_node.walk());
    
    let mut count = 0;
    for m in matches {
        count += 1;
        for capture in m.captures {
            let node = capture.node;
            let text = node.utf8_text(root_node.start_byte()..root_node.end_byte()).unwrap_or("");
            println!("✅ Import: {} at line {}", text, node.start_position().row);
        }
    }
    
    println!("✅ Import declarations found: {}", count);
    Ok(())
}

fn test_class_query(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query::new(
        language,
        r#"
        (class_declaration
          name: (identifier) @class.name
          body: (class_body) @class.body
        ) @class.def
        "#,
    )?;
    
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, root_node.walk());
    
    let mut count = 0;
    for m in matches {
        count += 1;
        for capture in m.captures {
            if capture.name == "class.name" {
                let node = capture.node;
                let text = node.utf8_text(root_node.start_byte()..root_node.end_byte()).unwrap_or("");
                println!("✅ Class: {} at line {}", text, node.start_position().row);
            }
        }
    }
    
    println!("✅ Class declarations found: {}", count);
    Ok(())
}

fn test_method_query(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query::new(
        language,
        r#"
        (method_declaration
          name: (identifier) @method.name
          parameters: (formal_parameters) @method.params
        ) @method.def
        "#,
    )?;
    
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, root_node.walk());
    
    let mut count = 0;
    for m in matches {
        count += 1;
        for capture in m.captures {
            if capture.name == "method.name" {
                let node = capture.node;
                let text = node.utf8_text(root_node.start_byte()..root_node.end_byte()).unwrap_or("");
                println!("✅ Method: {} at line {}", text, node.start_position().row);
            }
        }
    }
    
    println!("✅ Method declarations found: {}", count);
    Ok(())
}

fn test_constructor_query(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query::new(
        language,
        r#"
        (constructor_declaration
          name: (identifier) @constructor.name
          parameters: (formal_parameters) @constructor.params
        ) @constructor.def
        "#,
    )?;
    
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, root_node.walk());
    
    let mut count = 0;
    for m in matches {
        count += 1;
        for capture in m.captures {
            if capture.name == "constructor.name" {
                let node = capture.node;
                let text = node.utf8_text(root_node.start_byte()..root_node.end_byte()).unwrap_or("");
                println!("✅ Constructor: {} at line {}", text, node.start_position().row);
            }
        }
    }
    
    println!("✅ Constructor declarations found: {}", count);
    Ok(())
}

fn test_variable_query(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query::new(
        language,
        r#"
        (local_variable_declaration
          type: (_unannotated_type) @variable.type
          declarator: (variable_declarator
            name: (_variable_declarator_id) @variable.name
          )
        ) @variable.decl
        "#,
    )?;
    
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, root_node.walk());
    
    let mut count = 0;
    for m in matches {
        count += 1;
        for capture in m.captures {
            if capture.name == "variable.name" {
                let node = capture.node;
                let text = node.utf8_text(root_node.start_byte()..root_node.end_byte()).unwrap_or("");
                println!("✅ Variable: {} at line {}", text, node.start_position().row);
            }
        }
    }
    
    println!("✅ Variable declarations found: {}", count);
    Ok(())
}

fn test_method_call_query(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query::new(
        language,
        r#"
        (method_invocation
          name: (identifier) @method.called
          arguments: (argument_list) @method.args
        ) @method.call
        "#,
    )?;
    
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, root_node.walk());
    
    let mut count = 0;
    for m in matches {
        count += 1;
        for capture in m.captures {
            if capture.name == "method.called" {
                let node = capture.node;
                let text = node.utf8_text(root_node.start_byte()..root_node.end_byte()).unwrap_or("");
                println!("✅ Method call: {} at line {}", text, node.start_position().row);
            }
        }
    }
    
    println!("✅ Method calls found: {}", count);
    Ok(())
}

fn test_field_access_query(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query::new(
        language,
        r#"
        (field_access
          object: (primary_expression) @field.object
          field: (identifier) @field.name
        ) @field.access
        "#,
    )?;
    
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, root_node.walk());
    
    let mut count = 0;
    for m in matches {
        count += 1;
        for capture in m.captures {
            if capture.name == "field.name" {
                let node = capture.node;
                let text = node.utf8_text(root_node.start_byte()..root_node.end_byte()).unwrap_or("");
                println!("✅ Field access: {} at line {}", text, node.start_position().row);
            }
        }
    }
    
    println!("✅ Field accesses found: {}", count);
    Ok(())
}

fn test_enum_query(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query::new(
        language,
        r#"
        (enum_declaration
          name: (identifier) @enum.name
          body: (enum_body) @enum.body
        ) @enum.def
        "#,
    )?;
    
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, root_node.walk());
    
    let mut count = 0;
    for m in matches {
        count += 1;
        for capture in m.captures {
            if capture.name == "enum.name" {
                let node = capture.node;
                let text = node.utf8_text(root_node.start_byte()..root_node.end_byte()).unwrap_or("");
                println!("✅ Enum: {} at line {}", text, node.start_position().row);
            }
        }
    }
    
    println!("✅ Enum declarations found: {}", count);
    Ok(())
}

fn test_annotation_query(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query::new(
        language,
        r#"
        (marker_annotation
          name: (_name) @annotation.name
        ) @annotation.decl
        "#,
    )?;
    
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, root_node.walk());
    
    let mut count = 0;
    for m in matches {
        count += 1;
        for capture in m.captures {
            if capture.name == "annotation.name" {
                let node = capture.node;
                let text = node.utf8_text(root_node.start_byte()..root_node.end_byte()).unwrap_or("");
                println!("✅ Annotation: {} at line {}", text, node.start_position().row);
            }
        }
    }
    
    println!("✅ Annotations found: {}", count);
    Ok(())
}

fn test_record_query(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query::new(
        language,
        r#"
        (record_declaration
          name: (identifier) @record.name
          parameters: (formal_parameters) @record.params
        ) @record.def
        "#,
    )?;
    
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, root_node.walk());
    
    let mut count = 0;
    for m in matches {
        count += 1;
        for capture in m.captures {
            if capture.name == "record.name" {
                let node = capture.node;
                let text = node.utf8_text(root_node.start_byte()..root_node.end_byte()).unwrap_or("");
                println!("✅ Record: {} at line {}", text, node.start_position().row);
            }
        }
    }
    
    println!("✅ Record declarations found: {}", count);
    Ok(())
}

fn test_generic_query(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query::new(
        language,
        r#"
        (generic_type
          name: (type_identifier) @generic.name
          arguments: (type_arguments) @generic.args
        ) @generic.def
        "#,
    )?;
    
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, root_node.walk());
    
    let mut count = 0;
    for m in matches {
        count += 1;
        for capture in m.captures {
            if capture.name == "generic.name" {
                let node = capture.node;
                let text = node.utf8_text(root_node.start_byte()..root_node.end_byte()).unwrap_or("");
                println!("✅ Generic type: {} at line {}", text, node.start_position().row);
            }
        }
    }
    
    println!("✅ Generic types found: {}", count);
    Ok(())
}

fn test_exception_query(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query::new(
        language,
        r#"
        (try_statement
          body: (block) @try.body
          catch_clause: (catch_clause) @try.catch
        ) @try.stmt
        "#,
    )?;
    
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, root_node.walk());
    
    let mut count = 0;
    for m in matches {
        count += 1;
        for capture in m.captures {
            if capture.name == "try.stmt" {
                let node = capture.node;
                println!("✅ Try statement at line {}", node.start_position().row);
            }
        }
    }
    
    println!("✅ Try statements found: {}", count);
    Ok(())
}

fn test_loop_query(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query::new(
        language,
        r#"
        (for_statement
          init: (_) @for.init
          condition: (expression) @for.condition
          update: (_) @for.update
          body: (statement) @for.body
        ) @for.stmt
        "#,
    )?;
    
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, root_node.walk());
    
    let mut count = 0;
    for m in matches {
        count += 1;
        for capture in m.captures {
            if capture.name == "for.stmt" {
                let node = capture.node;
                println!("✅ For statement at line {}", node.start_position().row);
            }
        }
    }
    
    println!("✅ For statements found: {}", count);
    Ok(())
}

fn test_conditional_query(language: Language, root_node: &tree_sitter::Node) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query::new(
        language,
        r#"
        (if_statement
          condition: (parenthesized_expression) @if.condition
          consequence: (statement) @if.consequence
          alternative: (statement) @if.alternative
        ) @if.stmt
        "#,
    )?;
    
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, root_node, root_node.walk());
    
    let mut count = 0;
    for m in matches {
        count += 1;
        for capture in m.captures {
            if capture.name == "if.stmt" {
                let node = capture.node;
                println!("✅ If statement at line {}", node.start_position().row);
            }
        }
    }
    
    println!("✅ If statements found: {}", count);
    Ok(())
} 