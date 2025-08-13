# Class Hierarchy Analysis (CHA) Implementation Plan

## Overview

This document outlines the plan to implement Class Hierarchy Analysis (CHA) to rewrite the current call graph generation code in the CodeGraph Core project. CHA is a type-based reference analysis that provides more precise call graph construction by considering class inheritance hierarchies and method dispatch.

## Current State Analysis

### Existing Architecture
- **CodeParser**: Main parser that extracts functions and classes from source files
- **PetCodeGraph**: Call graph implementation using petgraph
- **EntityGraph**: Entity relationship graph for classes and functions
- **TreeSitterParser**: AST parsing for multiple programming languages

### Current Limitations
1. **Simple Name Matching**: Current call resolution only uses function name matching
2. **No Type Information**: Lacks consideration of receiver object types
3. **Conservative Analysis**: Creates many false positive edges due to over-approximation
4. **No Inheritance Analysis**: Doesn't leverage class hierarchy information
5. **Limited Method Resolution**: Cannot resolve virtual method calls properly

## Class Hierarchy Analysis (CHA) Benefits

### Advantages
- **More Precise**: Reduces false positive call edges
- **Type-Aware**: Considers declared types of receiver objects
- **Inheritance-Aware**: Leverages class hierarchy for method resolution
- **Sound**: Maintains soundness while improving precision
- **Efficient**: Computationally inexpensive compared to more complex analyses

### Key Concepts
- **cone(A)**: The inheritance subtree rooted at class A
- **Virtual Call Resolution**: Transform virtual calls to direct calls when possible
- **Type-Based Dispatch**: Use declared types to restrict possible method targets

## Implementation Plan

### Phase 1: Enhanced Type System and Class Hierarchy

#### 1.1 Extend Class Information Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    // ... existing fields ...
    pub parent_class: Option<String>,           // Direct parent class
    pub base_classes: Vec<String>,              // All base classes (transitive)
    pub virtual_methods: Vec<Uuid>,            // Virtual/overridable methods
    pub method_signatures: HashMap<String, MethodSignature>, // Method overloading support
    pub inheritance_chain: Vec<String>,        // Complete inheritance path
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodSignature {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub is_virtual: bool,
    pub is_override: bool,
    pub base_method: Option<String>,           // Base class method being overridden
}
```

#### 1.2 Implement Class Hierarchy Builder
```rust
pub struct ClassHierarchyBuilder {
    classes: HashMap<String, ClassInfo>,
    inheritance_graph: DiGraph<String, InheritanceEdge>,
}

impl ClassHierarchyBuilder {
    pub fn build_hierarchy(&mut self, classes: Vec<ClassInfo>) -> Result<(), String>;
    pub fn get_subtypes(&self, class_name: &str) -> Vec<String>;
    pub fn get_supertypes(&self, class_name: &str) -> Vec<String>;
    pub fn find_method_implementations(&self, method_name: &str, receiver_type: &str) -> Vec<Uuid>;
    pub fn resolve_virtual_call(&self, call_site: &CallSite) -> Vec<Uuid>;
}
```

### Phase 2: Enhanced Call Site Analysis

#### 2.1 Call Site Information Structure
```rust
#[derive(Debug, Clone)]
pub struct CallSite {
    pub id: Uuid,
    pub caller_function: Uuid,
    pub callee_name: String,
    pub receiver_type: Option<String>,         // Declared type of receiver object
    pub receiver_expression: Option<String>,   // AST expression for receiver
    pub call_type: CallType,
    pub line_number: usize,
    pub file_path: PathBuf,
    pub parameters: Vec<CallParameter>,
}

#[derive(Debug, Clone)]
pub enum CallType {
    Static,             // Static method call
    Virtual,            // Virtual method call
    Constructor,        // Constructor call
    Function,           // Free function call
    Method,            // Instance method call
}

#[derive(Debug, Clone)]
pub struct CallParameter {
    pub expression: String,
    pub inferred_type: Option<String>,
    pub is_literal: bool,
}
```

#### 2.2 Call Site Extractor
```rust
pub struct CallSiteExtractor {
    ts_parser: TreeSitterParser,
    class_hierarchy: ClassHierarchyBuilder,
}

impl CallSiteExtractor {
    pub fn extract_call_sites(&self, file_path: &PathBuf) -> Result<Vec<CallSite>, String>;
    pub fn analyze_receiver_type(&self, call_site: &CallSite) -> Option<String>;
    pub fn classify_call_type(&self, call_site: &CallSite) -> CallType;
    pub fn extract_method_parameters(&self, call_site: &CallSite) -> Vec<CallParameter>;
}
```

### Phase 3: CHA Algorithm Implementation

#### 3.1 CHA Core Algorithm
```rust
pub struct ClassHierarchyAnalysis {
    class_hierarchy: ClassHierarchyBuilder,
    call_sites: Vec<CallSite>,
    resolved_calls: HashMap<Uuid, Vec<Uuid>>,
}

impl ClassHierarchyAnalysis {
    /// Main CHA algorithm implementation
    pub fn analyze(&mut self) -> Result<(), String> {
        for call_site in &self.call_sites {
            match call_site.call_type {
                CallType::Static => {
                    self.resolve_static_call(call_site)?;
                },
                CallType::Virtual => {
                    self.resolve_virtual_call(call_site)?;
                },
                CallType::Constructor => {
                    self.resolve_constructor_call(call_site)?;
                },
                CallType::Function => {
                    self.resolve_function_call(call_site)?;
                },
                CallType::Method => {
                    self.resolve_method_call(call_site)?;
                },
            }
        }
        Ok(())
    }

    /// Resolve virtual method calls using CHA
    fn resolve_virtual_call(&self, call_site: &CallSite) -> Result<(), String> {
        let receiver_type = call_site.receiver_type.as_ref()
            .ok_or("No receiver type for virtual call")?;
        
        // Get the inheritance cone (subtree) for the receiver type
        let possible_types = self.class_hierarchy.get_subtypes(receiver_type);
        possible_types.push(receiver_type.clone());
        
        // Find all possible method implementations
        let mut target_methods = Vec::new();
        for class_type in possible_types {
            let methods = self.class_hierarchy.find_method_implementations(
                &call_site.callee_name, 
                &class_type
            );
            target_methods.extend(methods);
        }
        
        // Store resolved calls
        self.resolved_calls.insert(call_site.id, target_methods);
        Ok(())
    }
}
```

#### 3.2 Method Resolution Strategies
```rust
impl ClassHierarchyAnalysis {
    /// Resolve method calls with inheritance consideration
    fn resolve_method_call(&self, call_site: &CallSite) -> Result<(), String> {
        if let Some(receiver_type) = &call_site.receiver_type {
            // Use CHA for method resolution
            self.resolve_virtual_call(call_site)
        } else {
            // Fallback to name-based resolution
            self.resolve_function_call(call_site)
        }
    }

    /// Resolve constructor calls
    fn resolve_constructor_call(&self, call_site: &CallSite) -> Result<(), String> {
        // Constructor calls are typically direct
        let target_class = self.class_hierarchy.find_class_by_name(&call_site.callee_name);
        if let Some(class_id) = target_class {
            self.resolved_calls.insert(call_site.id, vec![class_id]);
        }
        Ok(())
    }
}
```

### Phase 4: Integration with Existing Code

#### 4.1 Enhanced CodeParser
```rust
impl CodeParser {
    /// Build call graph using CHA
    pub fn build_cha_call_graph(&mut self, dir: &Path) -> Result<PetCodeGraph, String> {
        // 1. Parse all files and extract classes/functions
        self.parse_directory(dir)?;
        
        // 2. Build class hierarchy
        let mut hierarchy_builder = ClassHierarchyBuilder::new();
        let classes = self.extract_all_classes();
        hierarchy_builder.build_hierarchy(classes)?;
        
        // 3. Extract call sites
        let mut call_extractor = CallSiteExtractor::new(
            self.ts_parser.clone(),
            hierarchy_builder.clone()
        );
        
        let mut all_call_sites = Vec::new();
        for file_path in self.file_functions.keys() {
            let call_sites = call_extractor.extract_call_sites(file_path)?;
            all_call_sites.extend(call_sites);
        }
        
        // 4. Perform CHA analysis
        let mut cha_analysis = ClassHierarchyAnalysis::new(
            hierarchy_builder,
            all_call_sites
        );
        cha_analysis.analyze()?;
        
        // 5. Build call graph from resolved calls
        let mut call_graph = PetCodeGraph::new();
        self.build_graph_from_cha_results(&cha_analysis, &mut call_graph)?;
        
        Ok(call_graph)
    }
}
```

#### 4.2 Call Graph Construction from CHA Results
```rust
impl CodeParser {
    /// Build call graph from CHA analysis results
    fn build_graph_from_cha_results(
        &self,
        cha_results: &ClassHierarchyAnalysis,
        call_graph: &mut PetCodeGraph,
    ) -> Result<(), String> {
        // Add all functions to the graph
        for (_file_path, functions) in &self.file_functions {
            for function in functions {
                call_graph.add_function(function.clone());
            }
        }
        
        // Add call relations based on CHA results
        for (call_site_id, target_methods) in &cha_results.resolved_calls {
            if let Some(call_site) = cha_results.get_call_site(*call_site_id) {
                for target_method_id in target_methods {
                    let relation = CallRelation {
                        caller_id: call_site.caller_function,
                        callee_id: *target_method_id,
                        caller_name: "".to_string(), // Will be filled by add_call_relation
                        callee_name: call_site.callee_name.clone(),
                        caller_file: call_site.file_path.clone(),
                        callee_file: call_site.file_path.clone(),
                        line_number: call_site.line_number,
                        is_resolved: true,
                    };
                    call_graph.add_call_relation(relation)?;
                }
            }
        }
        
        Ok(())
    }
}
```

### Phase 5: Language-Specific Extensions

#### 5.1 Rust-Specific CHA
```rust
pub struct RustCHAExtender {
    trait_resolver: TraitResolver,
    impl_resolver: ImplResolver,
}

impl RustCHAExtender {
    /// Handle Rust trait method resolution
    pub fn resolve_trait_method(&self, call_site: &CallSite) -> Result<Vec<Uuid>, String>;
    
    /// Handle Rust impl blocks
    pub fn resolve_impl_method(&self, call_site: &CallSite) -> Result<Vec<Uuid>, String>;
}
```

#### 5.2 Java-Specific CHA
```rust
pub struct JavaCHAExtender {
    interface_resolver: InterfaceResolver,
    package_resolver: PackageResolver,
}

impl JavaCHAExtender {
    /// Handle Java interface method resolution
    pub fn resolve_interface_method(&self, call_site: &CallSite) -> Result<Vec<Uuid>, String>;
    
    /// Handle Java package resolution
    pub fn resolve_package_method(&self, call_site: &CallSite) -> Result<Vec<Uuid>, String>;
}
```

#### 5.3 C++-Specific CHA
```rust
pub struct CppCHAExtender {
    template_resolver: TemplateResolver,
    namespace_resolver: NamespaceResolver,
}

impl CppCHAExtender {
    /// Handle C++ template method resolution
    pub fn resolve_template_method(&self, call_site: &CallSite) -> Result<Vec<Uuid>, String>;
    
    /// Handle C++ namespace resolution
    pub fn resolve_namespace_method(&self, call_site: &CallSite) -> Result<Vec<Uuid>, String>;
}
```


## Testing Strategy

### Unit Tests
- Class hierarchy construction
- Method resolution algorithms
- Call site extraction
- CHA analysis correctness

### Integration Tests
- End-to-end call graph generation
- Multi-language support
- Large codebase performance

### Benchmark Tests
- Comparison with current implementation
- Memory usage analysis
- Processing time measurements

## Expected Outcomes

### Precision Improvements
- **Reduced False Positives**: 30-50% reduction in incorrect call edges
- **Better Method Resolution**: Improved virtual method call resolution
- **Type-Aware Analysis**: More accurate call graph based on actual types

### Performance Characteristics
- **Linear Complexity**: O(n) where n is the number of classes
- **Memory Efficient**: Minimal additional memory overhead
- **Fast Analysis**: Sub-second analysis for medium-sized codebases

### Maintainability
- **Modular Design**: Clear separation of concerns
- **Extensible Architecture**: Easy to add new language support
- **Comprehensive Testing**: High test coverage for reliability

## Risk Mitigation

### Technical Risks
- **Complexity**: Break down implementation into manageable phases
- **Performance**: Profile and optimize critical paths
- **Language Support**: Start with well-supported languages first

### Integration Risks
- **Breaking Changes**: Maintain backward compatibility
- **Testing Coverage**: Ensure comprehensive testing before deployment
- **Documentation**: Provide clear migration guides

## Conclusion

The implementation of Class Hierarchy Analysis will significantly improve the precision and accuracy of call graph generation while maintaining the efficiency and scalability of the current system. The phased approach ensures manageable development and thorough testing at each stage.

This enhancement will make the CodeGraph Core project more valuable for static analysis, dependency management, and code understanding tasks across multiple programming languages.
