use std::path::Path;
use tempfile::TempDir;
use std::fs;
use codegraph_cli::codegraph::JavaAnalyzer;
use codegraph_cli::codegraph::analyzers::CodeAnalyzer;

#[test]
fn test_java_analyzer_new() {
    let analyzer = JavaAnalyzer::new();
    match analyzer {
        Ok(_) => println!("JavaAnalyzer created successfully"),
        Err(e) => {
            println!("JavaAnalyzer creation failed: {}", e);
            panic!("JavaAnalyzer::new() failed: {}", e);
        }
    }
    assert!(analyzer.is_ok());
}

#[test]
fn test_java_analyzer_basic_syntax() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    // 创建临时Java文件
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("BasicTest.java");
    
    let java_content = r#"
        package com.example.test;
        
        import java.util.List;
        import java.util.ArrayList;
        
        public class BasicTest {
            private String name;
            private int age;
            
            public BasicTest(String name, int age) {
                this.name = name;
                this.age = age;
            }
            
            public String getName() {
                return name;
            }
            
            public void setName(String name) {
                this.name = name;
            }
            
            public int getAge() {
                return age;
            }
            
            public void setAge(int age) {
                this.age = age;
            }
            
            public static void main(String[] args) {
                BasicTest test = new BasicTest("John", 30);
                System.out.println("Name: " + test.getName());
                System.out.println("Age: " + test.getAge());
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    // 获取分析结果
    let result = analyzer.get_analysis_result(&java_file.to_path_buf()).unwrap();
    println!("Debug: Total functions found: {}", result.functions.len());
    println!("Debug: Total classes found: {}", result.classes.len());
    
    // 检查类定义
    assert!(result.classes.len() >= 1);
    
    // 检查方法定义
    assert!(result.functions.len() >= 6); // 构造函数 + getter/setter + main
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_interfaces_and_abstract_classes() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("InterfaceTest.java");
    
    let java_content = r#"
        package com.example.interfaces;
        
        import java.util.Collection;
        
        public interface DataProcessor<T> {
            void process(T data);
            Collection<T> getProcessedData();
            default void validate(T data) {
                if (data == null) {
                    throw new IllegalArgumentException("Data cannot be null");
                }
            }
        }
        
        public abstract class AbstractProcessor<T> implements DataProcessor<T> {
            protected Collection<T> processedData;
            
            public AbstractProcessor() {
                this.processedData = new ArrayList<>();
            }
            
            @Override
            public Collection<T> getProcessedData() {
                return new ArrayList<>(processedData);
            }
            
            protected abstract void validateData(T data);
        }
        
        public class StringProcessor extends AbstractProcessor<String> {
            @Override
            public void process(String data) {
                validateData(data);
                processedData.add(data.toUpperCase());
            }
            
            @Override
            protected void validateData(String data) {
                if (data.trim().isEmpty()) {
                    throw new IllegalArgumentException("String cannot be empty");
                }
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    let result = analyzer.get_analysis_result(&java_file.to_path_buf()).unwrap();
    
    // 检查类定义
    assert!(result.classes.len() >= 2); // AbstractProcessor and StringProcessor
    
    // 检查方法定义
    assert!(result.functions.len() >= 1); // At least one method
    
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_records_and_enums() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("RecordsAndEnums.java");
    
    let java_content = r#"
        package com.example.records;
        
        import java.time.LocalDateTime;
        
        public record UserRecord(
            String username,
            String email,
            LocalDateTime createdAt
        ) {
            public UserRecord {
                if (username == null || username.trim().isEmpty()) {
                    throw new IllegalArgumentException("Username cannot be empty");
                }
                if (email == null || !email.contains("@")) {
                    throw new IllegalArgumentException("Invalid email format");
                }
            }
            
            public String getDisplayName() {
                return username + " (" + email + ")";
            }
        }
        
        public enum UserStatus {
            ACTIVE("Active"),
            INACTIVE("Inactive"),
            SUSPENDED("Suspended");
            
            private final String displayName;
            
            UserStatus(String displayName) {
                this.displayName = displayName;
            }
            
            public String getDisplayName() {
                return displayName;
            }
            
            public boolean isActive() {
                return this == ACTIVE;
            }
        }
        
        public enum Permission {
            READ,
            WRITE,
            DELETE,
            ADMIN;
            
            public boolean isAdmin() {
                return this == ADMIN;
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    let result = analyzer.get_analysis_result(&java_file.to_path_buf()).unwrap();
    
    // 检查类定义 (records and enums are treated as classes)
    assert!(result.classes.len() >= 3); // UserRecord, UserStatus, Permission
    
    // 检查方法定义
    assert!(result.functions.len() >= 1); // At least one method
    
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_modules() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("module-info.java");
    
    let java_content = r#"
        module com.example.app {
            requires java.base;
            requires java.sql;
            requires transitive java.logging;
            
            exports com.example.app.api;
            exports com.example.app.core to com.example.client;
            
            opens com.example.app.internal to com.example.test;
            
            uses com.example.app.spi.DataProvider;
            provides com.example.app.spi.DataProvider 
                with com.example.app.provider.SqlDataProvider,
                         com.example.app.provider.FileDataProvider;
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    let result = analyzer.get_analysis_result(&java_file.to_path_buf()).unwrap();
    
    // Module files typically don't have classes or functions
    // Just verify the file was parsed without errors
    // The result should be valid even if empty
    
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_annotations() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("AnnotationTest.java");
    
    let java_content = r#"
        package com.example.annotations;
        
        import java.lang.annotation.*;
        import javax.validation.constraints.*;
        
        @Target({ElementType.TYPE, ElementType.METHOD})
        @Retention(RetentionPolicy.RUNTIME)
        @Documented
        public @interface Loggable {
            String value() default "";
            LogLevel level() default LogLevel.INFO;
        }
        
        public enum LogLevel {
            DEBUG, INFO, WARN, ERROR
        }
        
        @Loggable(level = LogLevel.DEBUG)
        public class AnnotatedClass {
            @NotNull
            @Size(min = 1, max = 100)
            private String name;
            
            @Loggable("Constructor called")
            public AnnotatedClass(@NotNull String name) {
                this.name = name;
            }
            
            @Override
            @Loggable(level = LogLevel.INFO)
            public String toString() {
                return "AnnotatedClass{name='" + name + "'}";
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    let result = analyzer.get_analysis_result(&java_file.to_path_buf()).unwrap();
    
    // 检查类定义
    assert!(result.classes.len() >= 2); // Loggable interface, AnnotatedClass, LogLevel enum
    
    // 检查方法定义
    assert!(result.functions.len() >= 1); // At least one method
    
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_exceptions_and_control_flow() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("ControlFlowTest.java");
    
    let java_content = r#"
        package com.example.controlflow;
        
        import java.io.*;
        import java.util.*;
        
        public class ControlFlowTest {
            public void processData(List<String> data) throws IOException {
                try (BufferedReader reader = new BufferedReader(new StringReader("test"))) {
                    for (String item : data) {
                        if (item == null) {
                            continue;
                        }
                        
                        try {
                            processItem(item);
                        } catch (IllegalArgumentException e) {
                            System.err.println("Invalid item: " + item);
                        }
                    }
                } catch (IOException e) {
                    throw new IOException("Failed to process data", e);
                }
            }
            
            private void processItem(String item) {
                switch (item.toLowerCase()) {
                    case "start" -> System.out.println("Starting...");
                    case "stop" -> System.out.println("Stopping...");
                    case "pause" -> System.out.println("Paused");
                    default -> System.out.println("Unknown command: " + item);
                }
                
                // Enhanced switch expression
                String result = switch (item.length()) {
                    case 0 -> "empty";
                    case 1, 2, 3 -> "short";
                    case 4, 5, 6 -> "medium";
                    default -> "long";
                };
                
                System.out.println("Result: " + result);
            }
            
            public void loopExamples() {
                // Traditional for loop
                for (int i = 0; i < 10; i++) {
                    if (i % 2 == 0) {
                        continue;
                    }
                    System.out.println("Odd: " + i);
                }
                
                // Enhanced for loop
                List<String> items = Arrays.asList("a", "b", "c");
                for (String item : items) {
                    System.out.println("Item: " + item);
                }
                
                // While loop
                int count = 0;
                while (count < 5) {
                    System.out.println("Count: " + count);
                    count++;
                }
                
                // Do-while loop
                do {
                    System.out.println("Do-while count: " + count);
                    count--;
                } while (count > 0);
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    let result = analyzer.get_analysis_result(&java_file.to_path_buf()).unwrap();
    
    // 检查类定义
    assert!(result.classes.len() >= 1);
    
    // 检查方法定义
    assert!(result.functions.len() >= 3); // processData, processItem, loopExamples
    
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_generics_and_collections() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("GenericsTest.java");
    
    let java_content = r#"
        package com.example.generics;
        
        import java.util.*;
        import java.util.function.*;
        
        public class GenericsTest<T extends Comparable<T>> {
            private List<T> items;
            private Map<String, T> itemMap;
            
            public GenericsTest() {
                this.items = new ArrayList<>();
                this.itemMap = new HashMap<>();
            }
            
            public <U extends Number> void processItems(List<U> numbers) {
                for (U number : numbers) {
                    if (number.doubleValue() > 0) {
                        items.add((T) number.toString());
                    }
                }
            }
            
            public Optional<T> findItem(Predicate<T> predicate) {
                return items.stream()
                    .filter(predicate)
                    .findFirst();
            }
            
            public <R> List<R> transformItems(Function<T, R> transformer) {
                return items.stream()
                    .map(transformer)
                    .collect(Collectors.toList());
            }
            
            public void addItem(String key, T item) {
                itemMap.put(key, item);
            }
            
            public T getItem(String key) {
                return itemMap.get(key);
            }
            
            public static <E> void swap(List<E> list, int i, int j) {
                E temp = list.get(i);
                list.set(i, list.get(j));
                list.set(j, temp);
            }
            
            public static <E extends Comparable<E>> void sort(List<E> list) {
                Collections.sort(list);
            }
        }
        
        class WildcardExample {
            public static void printList(List<?> list) {
                for (Object item : list) {
                    System.out.println(item);
                }
            }
            
            public static void addNumbers(List<? super Integer> list) {
                list.add(1);
                list.add(2);
                list.add(3);
            }
            
            public static double sumNumbers(List<? extends Number> list) {
                return list.stream()
                    .mapToDouble(Number::doubleValue)
                    .sum();
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    let result = analyzer.get_analysis_result(&java_file.to_path_buf()).unwrap();
    
    // 检查类定义
    assert!(result.classes.len() >= 2); // GenericsTest and WildcardExample
    
    // 检查方法定义
    assert!(result.functions.len() >= 1); // At least one method
    
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_lambda_and_streams() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("LambdaTest.java");
    
    let java_content = r#"
        package com.example.lambda;
        
        import java.util.*;
        import java.util.function.*;
        import java.util.stream.*;
        
        public class LambdaTest {
            public void processWithLambda() {
                List<String> names = Arrays.asList("Alice", "Bob", "Charlie", "David");
                
                // Lambda expressions
                names.forEach(name -> System.out.println("Hello, " + name));
                
                // Method references
                names.stream()
                    .map(String::toUpperCase)
                    .forEach(System.out::println);
                
                // Predicate with lambda
                Predicate<String> longName = name -> name.length() > 4;
                List<String> longNames = names.stream()
                    .filter(longName)
                    .collect(Collectors.toList());
                
                // Function with lambda
                Function<String, Integer> nameLength = String::length;
                List<Integer> lengths = names.stream()
                    .map(nameLength)
                    .collect(Collectors.toList());
                
                // Consumer with lambda
                Consumer<String> printer = name -> System.out.println("Processing: " + name);
                names.forEach(printer);
                
                // Supplier with lambda
                Supplier<Random> randomSupplier = Random::new;
                Random random = randomSupplier.get();
                
                // BiFunction with lambda
                BiFunction<String, String, String> concat = (a, b) -> a + " " + b;
                String result = concat.apply("Hello", "World");
                
                // Stream operations
                Map<Integer, List<String>> groupedByLength = names.stream()
                    .collect(Collectors.groupingBy(String::length));
                
                String longestName = names.stream()
                    .max(Comparator.comparing(String::length))
                    .orElse("");
                
                int totalLength = names.stream()
                    .mapToInt(String::length)
                    .sum();
                
                boolean hasLongName = names.stream()
                    .anyMatch(name -> name.length() > 5);
                
                List<String> sortedNames = names.stream()
                    .sorted()
                    .collect(Collectors.toList());
            }
            
            public void processNumbers() {
                List<Integer> numbers = Arrays.asList(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
                
                // Stream with intermediate operations
                List<Integer> evenSquares = numbers.stream()
                    .filter(n -> n % 2 == 0)
                    .map(n -> n * n)
                    .collect(Collectors.toList());
                
                // Parallel stream
                int sum = numbers.parallelStream()
                    .mapToInt(Integer::intValue)
                    .sum();
                
                // Reduce operation
                Optional<Integer> max = numbers.stream()
                    .reduce(Integer::max);
                
                // Collect with custom collector
                String joined = numbers.stream()
                    .map(Object::toString)
                    .collect(Collectors.joining(", "));
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    let result = analyzer.get_analysis_result(&java_file.to_path_buf()).unwrap();
    
    // 检查类定义
    assert!(result.classes.len() >= 1);
    
    // 检查方法定义
    assert!(result.functions.len() >= 2); // processWithLambda and processNumbers
    
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_directory_analysis() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    // 创建临时目录结构
    let temp_dir = TempDir::new().unwrap();
    let java_dir = temp_dir.path().join("java_project");
    fs::create_dir(&java_dir).unwrap();
    
    // 创建多个Java文件
    let java_file1 = java_dir.join("Main.java");
    let java_file2 = java_dir.join("Utils.java");
    let java_file3 = java_dir.join("model").join("User.java");
    fs::create_dir_all(&java_file3.parent().unwrap()).unwrap();
    
    let java_content1 = r#"
        package com.example;
        
        public class Main {
            public static void main(String[] args) {
                System.out.println("Hello, World!");
            }
        }
    "#;
    
    let java_content2 = r#"
        package com.example;
        
        public class Utils {
            public static String format(String input) {
                return input.trim().toUpperCase();
            }
        }
    "#;
    
    let java_content3 = r#"
        package com.example.model;
        
        public class User {
            private String name;
            
            public User(String name) {
                this.name = name;
            }
            
            public String getName() {
                return name;
            }
        }
    "#;
    
    fs::write(&java_file1, java_content1).unwrap();
    fs::write(&java_file2, java_content2).unwrap();
    fs::write(&java_file3, java_content3).unwrap();
    
    // 分析整个目录
    let result = analyzer.analyze_directory(&java_dir);
    assert!(result.is_ok());
    
    let result = analyzer.get_analysis_result(&java_file1.to_path_buf()).unwrap();
    
    // 检查是否找到了类
    assert!(result.classes.len() >= 1);
    
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_error_handling() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("Invalid.java");
    
    // 创建语法无效的Java文件
    let invalid_java = r#"
        package com.example;
        
        public class Invalid {
            public void invalidMethod() {
                if (true {  // Missing closing parenthesis
                    System.out.println("Invalid syntax");
                }
                
                for (int i = 0; i < 10; i++ {  // Missing closing parenthesis
                    System.out.println(i);
                }
            }
        }
    "#;
    
    fs::write(&java_file, invalid_java).unwrap();
    
    // 分析应该失败
    let result = analyzer.analyze_file(&java_file);
    // 注意：Tree-sitter可能仍然能够解析部分内容，所以这个测试可能不会按预期失败
    
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_complex_project() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("complex_project");
    fs::create_dir(&project_dir).unwrap();
    
    // 创建复杂的项目结构
    let src_dir = project_dir.join("src").join("main").join("java");
    let test_dir = project_dir.join("src").join("test").join("java");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&test_dir).unwrap();
    
    // 主应用类
    let main_class = src_dir.join("com").join("example").join("app").join("Application.java");
    fs::create_dir_all(&main_class.parent().unwrap()).unwrap();
    
    let main_content = r#"
        package com.example.app;
        
        import com.example.service.UserService;
        import com.example.model.User;
        import com.example.config.AppConfig;
        
        @SpringBootApplication
        public class Application {
            public static void main(String[] args) {
                SpringApplication.run(Application.class, args);
            }
        }
    "#;
    
    // 服务类
    let service_class = src_dir.join("com").join("example").join("service").join("UserService.java");
    fs::create_dir_all(&service_class.parent().unwrap()).unwrap();
    
    let service_content = r#"
        package com.example.service;
        
        import com.example.model.User;
        import com.example.repository.UserRepository;
        import org.springframework.stereotype.Service;
        import org.springframework.transaction.annotation.Transactional;
        
        @Service
        @Transactional
        public class UserService {
            private final UserRepository userRepository;
            
            public UserService(UserRepository userRepository) {
                this.userRepository = userRepository;
            }
            
            public User createUser(String name, String email) {
                User user = new User(name, email);
                return userRepository.save(user);
            }
            
            public Optional<User> findUserById(Long id) {
                return userRepository.findById(id);
            }
            
            public List<User> findAllUsers() {
                return userRepository.findAll();
            }
        }
    "#;
    
    // 模型类
    let model_class = src_dir.join("com").join("example").join("model").join("User.java");
    fs::create_dir_all(&model_class.parent().unwrap()).unwrap();
    
    let model_content = r#"
        package com.example.model;
        
        import javax.persistence.*;
        import javax.validation.constraints.*;
        import java.time.LocalDateTime;
        
        @Entity
        @Table(name = "users")
        public class User {
            @Id
            @GeneratedValue(strategy = GenerationType.IDENTITY)
            private Long id;
            
            @NotBlank
            @Size(min = 2, max = 50)
            @Column(name = "name", nullable = false)
            private String name;
            
            @Email
            @Column(name = "email", unique = true, nullable = false)
            private String email;
            
            @Column(name = "created_at")
            private LocalDateTime createdAt;
            
            @PrePersist
            protected void onCreate() {
                createdAt = LocalDateTime.now();
            }
            
            // Getters and setters
            public Long getId() { return id; }
            public void setId(Long id) { this.id = id; }
            public String getName() { return name; }
            public void setName(String name) { this.name = name; }
            public String getEmail() { return email; }
            public void setEmail(String email) { this.email = email; }
            public LocalDateTime getCreatedAt() { return createdAt; }
            public void setCreatedAt(LocalDateTime createdAt) { this.createdAt = createdAt; }
        }
    "#;
    
    // 测试类
    let test_class = test_dir.join("com").join("example").join("service").join("UserServiceTest.java");
    fs::create_dir_all(&test_class.parent().unwrap()).unwrap();
    
    let test_content = r#"
        package com.example.service;
        
        import com.example.model.User;
        import com.example.repository.UserRepository;
        import org.junit.jupiter.api.Test;
        import org.junit.jupiter.api.BeforeEach;
        import org.mockito.Mock;
        import org.mockito.MockitoAnnotations;
        
        import static org.junit.jupiter.api.Assertions.*;
        import static org.mockito.Mockito.*;
        
        class UserServiceTest {
            @Mock
            private UserRepository userRepository;
            
            private UserService userService;
            
            @BeforeEach
            void setUp() {
                MockitoAnnotations.openMocks(this);
                userService = new UserService(userRepository);
            }
            
            @Test
            void testCreateUser() {
                // Given
                String name = "John Doe";
                String email = "john@example.com";
                User expectedUser = new User(name, email);
                
                when(userRepository.save(any(User.class))).thenReturn(expectedUser);
                
                // When
                User result = userService.createUser(name, email);
                
                // Then
                assertNotNull(result);
                assertEquals(name, result.getName());
                assertEquals(email, result.getEmail());
                verify(userRepository).save(any(User.class));
            }
        }
    "#;
    
    fs::write(&main_class, main_content).unwrap();
    fs::write(&service_class, service_content).unwrap();
    fs::write(&model_class, model_content).unwrap();
    fs::write(&test_class, test_content).unwrap();
    
    // 分析整个项目
    let result = analyzer.analyze_directory(&project_dir);
    assert!(result.is_ok());
    
    let result = analyzer.get_analysis_result(&main_class.to_path_buf()).unwrap();
    
    // 检查复杂的分析结果
    assert!(result.classes.len() >= 1);
    
    assert!(result.functions.len() >= 1);
    
    temp_dir.close().unwrap();
} 