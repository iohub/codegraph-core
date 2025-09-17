use std::path::Path;
use std::fs;
use std::collections::HashMap;
use qdrant_client::Qdrant;
use qdrant_client::config::QdrantConfig;
use qdrant_client::qdrant::{CreateCollection, VectorParams, Distance, PointStruct, VectorsConfig, Value, UpsertPointsBuilder};
use uuid::Uuid;
use tracing::{info, error, debug};
use serde_json::json;
use reqwest;

use crate::codegraph::treesitter::TreeSitterParser;
use crate::codegraph::parser::CodeParser;

pub struct VectorizeService {
    qdrant_client: Qdrant,
    collection_name: String,
    embedding_client: reqwest::Client,
    embedding_url: String,
}

impl VectorizeService {
    pub async fn new(qdrant_url: &str, collection_name: String) -> Result<Self, Box<dyn std::error::Error>> {
        let config = QdrantConfig::from_url(qdrant_url);
        let qdrant_client = Qdrant::new(config)?;
        let embedding_client = reqwest::Client::new();
        let embedding_url = "http://localhost:9200/embedding".to_string();
        
        Ok(Self {
            qdrant_client,
            collection_name,
            embedding_client,
            embedding_url,
        })
    }

    /// 创建或获取集合
    pub async fn ensure_collection(&self) -> Result<(), Box<dyn std::error::Error>> {
        let collections = self.qdrant_client.list_collections().await?;
        let collection_exists = collections
            .collections
            .iter()
            .any(|c| c.name == self.collection_name);

        if !collection_exists {
            info!("Creating collection: {}", self.collection_name);
            
            let create_collection = CreateCollection {
                collection_name: self.collection_name.clone(),
                vectors_config: Some(VectorsConfig {
                    config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                        VectorParams {
                            size: 768,
                            distance: Distance::Cosine.into(),
                            ..Default::default()
                        }
                    ))
                }), // 768维向量，使用余弦相似度
                ..Default::default()
            };
            
            self.qdrant_client.create_collection(create_collection).await?;
            info!("Collection {} created successfully", self.collection_name);
        } else {
            info!("Collection {} already exists", self.collection_name);
        }

        Ok(())
    }

    /// 获取代码块的嵌入向量（HTTP请求实现）
    async fn get_embedding(&self, code_block: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        if code_block.is_empty() {
            return Err("Code block is empty".into());
        }
        // if code_block len > 2048 get first 1800 chars
        let code_block = if code_block.len() > 2048 {
            &code_block[..1800]
        } else {
            code_block
        };
        let request_body = json!({
            "content": code_block
        });
        debug!("Sending embedding request for code block (length: {})", code_block.len());
        
        let response = self.embedding_client
            .post(&self.embedding_url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Embedding service returned error: {}", response.status()).into());
        }

        let response_json: serde_json::Value = response.json().await?;
        // info!("Embedding service response: {:?}", response_json);
        // 解析返回的嵌入向量
        if let Some(first_item) = response_json.get(0) {
            if let Some(embedding_array) = first_item.get("embedding") {
                // embedding是一个二维数组 [[...]]，我们需要获取第一个（也是唯一一个）子数组
                if let Some(embedding_outer_array) = embedding_array.as_array() {
                    if let Some(embedding_inner_array) = embedding_outer_array.get(0) {
                        if let Some(embedding_values) = embedding_inner_array.as_array() {
                            let vector: Vec<f32> = embedding_values
                                .iter()
                                .filter_map(|v| v.as_f64().map(|f| f as f32))
                                .collect();
                            info!("Embedding vector created with size: {}", vector.len());
                            Ok(vector)
                        } else {
                            error!("Inner embedding field is not an array");
                            Err("Inner embedding field is not an array".into())
                        }
                    } else {
                        error!("No inner embedding array found");
                        Err("No inner embedding array found".into())
                    }
                } else {
                    error!("Embedding field is not an array");
                    Err("Embedding field is not an array".into())
                }
            } else {
                error!("No embedding field in response");
                Err("No embedding field in response".into())
            }
        } else {
            error!("No data in response");
            Err("No data in response".into())
        }
    }

    /// 向量化目录中的代码文件
    pub async fn vectorize_directory(&self, dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting vectorization of directory: {}", dir_path);
        
        let mut parser = CodeParser::new();
        let mut ts_parser = TreeSitterParser::new();
        
        let path = Path::new(dir_path);
        let files = parser.scan_directory(path);
        
        info!("Found {} files to vectorize", files.len());
        let mut total_vectors = 0;
        
        for file_path in files {
            debug!("Processing file: {}", file_path.display());
            match self.process_file(&file_path, &mut ts_parser).await {
                Ok(vectors) => {
                    total_vectors += vectors;
                    debug!("File {} processed successfully with {} vectors", file_path.display(), vectors);
                }
                Err(e) => {
                    error!("Failed to process file {}: {}", file_path.display(), e);
                }
            }
        }
        
        info!("Vectorization completed. Total vectors created: {}", total_vectors);
        Ok(())
    }

    /// 处理单个文件
    async fn process_file(&self, file_path: &Path, ts_parser: &mut TreeSitterParser) -> Result<usize, Box<dyn std::error::Error>> {
        // 读取文件内容
        let _content = fs::read_to_string(file_path)?;
        
        // 使用TreeSitter解析器获取代码块
        let symbols = ts_parser.parse_file(&file_path.to_path_buf())?;
        
        let mut vectors_created = 0;
        let mut points = Vec::new();
        
        for symbol in symbols {
            let symbol_guard = symbol.read();
            let symbol_ref = symbol_guard.as_ref();
            
            // 只处理函数和类定义
            match symbol_ref.symbol_type() {
                crate::codegraph::treesitter::structs::SymbolType::StructDeclaration |
                crate::codegraph::treesitter::structs::SymbolType::FunctionDeclaration => {
                    
                    // 获取代码块内容
                    let symbol_info = symbol_ref.symbol_info_struct();
                    let code_block = symbol_info.get_content_from_file_blocked()
                        .unwrap_or_else(|e| {
                            eprintln!("Warning: Failed to get content for {}: {}", symbol_ref.name(), e);
                            symbol_ref.name().to_string()
                        });
                    
                    // 生成嵌入向量
                    let embedding = match self.get_embedding(&code_block).await {
                        Ok(vec) => vec,
                        Err(e) => {
                            error!("Failed to get embedding for symbol {}: {}", symbol_ref.name(), e);
                            continue;
                        }
                    };
                    
                    // 创建点数据
                    let point_id = Uuid::new_v4().to_string();
                    // 创建payload
                    let mut payload = HashMap::new();
                    payload.insert("file_path", Value::from(file_path.to_string_lossy().to_string()));
                    payload.insert("symbol_name", Value::from(symbol_ref.name().to_string()));
                    payload.insert("symbol_type", Value::from(format!("{:?}", symbol_ref.symbol_type())));
                    payload.insert("language", Value::from(format!("{:?}", symbol_ref.language())));
                    payload.insert("line_start", Value::from((symbol_ref.full_range().start_point.row + 1) as i64));
                    payload.insert("line_end", Value::from((symbol_ref.full_range().end_point.row + 1) as i64));
                    payload.insert("code_block", Value::from(code_block));
                    
                    let point = PointStruct::new(
                        point_id,
                        embedding,
                        payload
                    );
                    debug!("Point: {:?}", point);
                    points.push(point);
                    vectors_created += 1;
                    
                    // 批量上传，每100个向量上传一次
                    if points.len() >= 100 {
                        self.upload_points(&points).await?;
                        points.clear();
                    }
                }
                _ => {}
            }
        }
        
        // 上传剩余的向量
        if !points.is_empty() {
            self.upload_points(&points).await?;
        }
        
        Ok(vectors_created)
    }

    /// 上传向量到Qdrant
    async fn upload_points(&self, points: &[PointStruct]) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Uploading {} vectors to Qdrant", points.len());
        
        let upsert_points = UpsertPointsBuilder::new(&self.collection_name, points.to_vec()).wait(true);
        let operation_info = self.qdrant_client
            .upsert_points(upsert_points)
            .await?;
        
        debug!("Upload completed: {:?}", operation_info);
        Ok(())
    }
}

/// 运行向量化命令
pub async fn run_vectorize(path: String, collection: String, qdrant_url: String) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting vectorize command");
    info!("Path: {}", path);
    info!("Collection: {}", collection);
    info!("Qdrant URL: {}", qdrant_url);
    
    // 创建向量化服务
    let service = VectorizeService::new(&qdrant_url, collection).await?;
    
    // 向量化目录
    service.vectorize_directory(&path).await?;
    
    info!("Vectorize command completed successfully");
    Ok(())
}