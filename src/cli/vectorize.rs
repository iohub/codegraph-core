use std::path::Path;
use std::fs;
use std::collections::HashMap;
use qdrant_client::Qdrant;
use qdrant_client::config::QdrantConfig;
use qdrant_client::qdrant::{CreateCollection, VectorParams, Distance, PointStruct, VectorsConfig, Value, UpsertPointsBuilder};
use uuid::Uuid;
use tracing::{info, error, debug};

use crate::codegraph::treesitter::TreeSitterParser;
use crate::codegraph::parser::CodeParser;

pub struct VectorizeService {
    qdrant_client: Qdrant,
    collection_name: String,
}

impl VectorizeService {
    pub async fn new(qdrant_url: &str, collection_name: String) -> Result<Self, Box<dyn std::error::Error>> {
        let config = QdrantConfig::from_url(qdrant_url);
        let qdrant_client = Qdrant::new(config)?;
        
        Ok(Self {
            qdrant_client,
            collection_name,
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
                            size: 384,
                            distance: Distance::Cosine.into(),
                            ..Default::default()
                        }
                    ))
                }), // 384维向量，使用余弦相似度
                ..Default::default()
            };
            
            self.qdrant_client.create_collection(create_collection).await?;
            info!("Collection {} created successfully", self.collection_name);
        } else {
            info!("Collection {} already exists", self.collection_name);
        }

        Ok(())
    }

    /// 获取代码块的嵌入向量（mock实现）
    fn get_embedding(&self, code_block: &str) -> Vec<f32> {
        // Mock embedding - 使用简单的哈希方法生成384维向量
        let mut vector = vec![0.0f32; 384];
        let code_bytes = code_block.as_bytes();
        
        for (i, &byte) in code_bytes.iter().enumerate() {
            let idx = i % 384;
            vector[idx] += (byte as f32) / 255.0;
        }
        
        // 归一化向量
        let norm = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            vector.iter_mut().for_each(|x| *x /= norm);
        }
        
        vector
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
                    let embedding = self.get_embedding(&code_block);
                    
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
    
    // 确保集合存在
    service.ensure_collection().await?;
    
    // 向量化目录
    service.vectorize_directory(&path).await?;
    
    info!("Vectorize command completed successfully");
    Ok(())
}