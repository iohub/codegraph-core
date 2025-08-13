use uuid::Uuid;
use std::path::PathBuf;

/// 简化的类信息结构
#[derive(Debug, Clone)]
pub struct SimpleClassInfo {
    pub id: Uuid,
    pub name: String,
    pub file_path: PathBuf,
    pub line_start: usize,
    pub line_end: usize,
    pub namespace: String,
    pub language: String,
    pub parent_class: Option<String>,
    pub methods: Vec<Uuid>,
}

/// 简化的调用点信息
#[derive(Debug, Clone)]
pub struct SimpleCallSite {
    pub id: Uuid,
    pub caller_function: Uuid,
    pub callee_name: String,
    pub receiver_type: Option<String>,
    pub line_number: usize,
    pub file_path: PathBuf,
}

/// 简化的方法解析结果
#[derive(Debug, Clone)]
pub struct SimpleMethodResolution {
    pub call_site_id: Uuid,
    pub target_methods: Vec<Uuid>,
    pub is_resolved: bool,
}

impl SimpleClassInfo {
    pub fn new(
        id: Uuid,
        name: String,
        file_path: PathBuf,
        line_start: usize,
        line_end: usize,
        namespace: String,
        language: String,
    ) -> Self {
        Self {
            id,
            name,
            file_path,
            line_start,
            line_end,
            namespace,
            language,
            parent_class: None,
            methods: Vec::new(),
        }
    }

    pub fn add_method(&mut self, method_id: Uuid) {
        if !self.methods.contains(&method_id) {
            self.methods.push(method_id);
        }
    }

    pub fn set_parent_class(&mut self, parent: String) {
        self.parent_class = Some(parent);
    }
}

impl SimpleCallSite {
    pub fn new(
        id: Uuid,
        caller_function: Uuid,
        callee_name: String,
        line_number: usize,
        file_path: PathBuf,
    ) -> Self {
        Self {
            id,
            caller_function,
            callee_name,
            receiver_type: None,
            line_number,
            file_path,
        }
    }

    pub fn with_receiver_type(mut self, receiver_type: String) -> Self {
        self.receiver_type = Some(receiver_type);
        self
    }
} 