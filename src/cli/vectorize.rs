use std::path::Path;
use std::fs;
use std::sync::Arc;
use std::env;
use lancedb::{connect, Connection};
use arrow::array::{
    FixedSizeListBuilder, Float32Builder, Int64Builder, RecordBatch, StringBuilder,
};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatchIterator;
use uuid::Uuid;
use tracing::{info, error, debug};
use reqwest::Client;
use serde::Deserialize;

use crate::codegraph::treesitter::TreeSitterParser;
use crate::codegraph::parser::CodeParser;

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

struct CodePoint {
    id: String,
    vector: Vec<f32>,
    file_path: String,
    symbol_name: String,
    symbol_type: String,
    language: String,
    line_start: i64,
    line_end: i64,
    code_block: String,
}

pub struct VectorizeService {
    connection: Connection,
    table_name: String,
    client: Client,
    api_token: String,
}

impl VectorizeService {
    pub async fn new(db_path: &str, table_name: String) -> Result<Self, Box<dyn std::error::Error>> {
        // LanceDB connection (embedded)
        let connection = connect(db_path).execute().await?;
        
        // Initialize HTTP client and get API token
        let api_token = env::var("SILICONFLOW_API_KEY")
            .map_err(|_| "SILICONFLOW_API_KEY environment variable not set. Please set it to use remote embedding service.")?;
        let client = Client::new();
        
        Ok(Self {
            connection,
            table_name,
            client,
            api_token,
        })
    }

    /// Create or get the collection (table)
    pub async fn ensure_collection(&self) -> Result<(), Box<dyn std::error::Error>> {
        let table_names = self.connection.table_names().execute().await?;
        if !table_names.contains(&self.table_name) {
            info!("Creating table: {}", self.table_name);
            
            // Qwen/Qwen3-Embedding-4B has 2560 dimensions
            let vector_size = 2560; 
            
            let schema = Arc::new(Schema::new(vec![
                Field::new("id", DataType::Utf8, false),
                Field::new("vector", DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    vector_size
                ), false),
                Field::new("file_path", DataType::Utf8, false),
                Field::new("symbol_name", DataType::Utf8, false),
                Field::new("symbol_type", DataType::Utf8, false),
                Field::new("language", DataType::Utf8, false),
                Field::new("line_start", DataType::Int64, false),
                Field::new("line_end", DataType::Int64, false),
                Field::new("code_block", DataType::Utf8, false),
            ]));
            
            self.connection.create_empty_table(&self.table_name, schema).execute().await?;
            info!("Table {} created successfully", self.table_name);
        } else {
            info!("Table {} already exists", self.table_name);
        }

        Ok(())
    }

    /// Get embedding for code block using remote model
    async fn get_embedding(&self, code_block: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        if code_block.is_empty() {
            return Err("Code block is empty".into());
        }

        // Truncate if too long (approx 32k tokens, safe limit 30k chars for now)
        // Note: 32K context window is quite large, but we should still have a safety limit
        // Assuming ~4 chars per token for English, 32k tokens is ~128k chars.
        // For mixed content, being conservative with 64k chars is safe.
        let code_block = if code_block.len() > 64000 {
            &code_block[..64000]
        } else {
            code_block
        };
        
        let response = self.client.post("https://api.siliconflow.cn/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "Qwen/Qwen3-Embedding-4B", // Correcting this line
                "input": code_block,
                "encoding_format": "float"
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            return Err(format!("API request failed with status {}: {}", status, text).into());
        }

        let embedding_response: EmbeddingResponse = response.json().await?;
        
        if let Some(data) = embedding_response.data.into_iter().next() {
            Ok(data.embedding)
        } else {
            Err("No embedding data returned from API".into())
        }
    }

    /// Vectorize directory
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

    /// Process single file
    async fn process_file(&self, file_path: &Path, ts_parser: &mut TreeSitterParser) -> Result<usize, Box<dyn std::error::Error>> {
        // Read file content
        let _content = fs::read_to_string(file_path)?;
        
        // Parse with TreeSitter
        let symbols = ts_parser.parse_file(&file_path.to_path_buf())?;
        
        let mut vectors_created = 0;
        let mut points = Vec::new();
        
        for symbol in symbols {
            let symbol_guard = symbol.read();
            let symbol_ref = symbol_guard.as_ref();
            
            // Only process function and class definitions
            match symbol_ref.symbol_type() {
                crate::codegraph::treesitter::structs::SymbolType::StructDeclaration |
                crate::codegraph::treesitter::structs::SymbolType::FunctionDeclaration => {
                    
                    // Get code block content
                    let symbol_info = symbol_ref.symbol_info_struct();
                    let code_block = symbol_info.get_content_from_file_blocked()
                        .unwrap_or_else(|e| {
                            eprintln!("Warning: Failed to get content for {}: {}", symbol_ref.name(), e);
                            symbol_ref.name().to_string()
                        });
                    
                    // Generate embedding
                    let embedding = match self.get_embedding(&code_block).await {
                        Ok(vec) => vec,
                        Err(e) => {
                            error!("Failed to get embedding for symbol {}: {}", symbol_ref.name(), e);
                            continue;
                        }
                    };
                    
                    // Create point
                    let point = CodePoint {
                        id: Uuid::new_v4().to_string(),
                        vector: embedding,
                        file_path: file_path.to_string_lossy().to_string(),
                        symbol_name: symbol_ref.name().to_string(),
                        symbol_type: format!("{:?}", symbol_ref.symbol_type()),
                        language: format!("{:?}", symbol_ref.language()),
                        line_start: (symbol_ref.full_range().start_point.row + 1) as i64,
                        line_end: (symbol_ref.full_range().end_point.row + 1) as i64,
                        code_block,
                    };
                    
                    debug!("Point created for symbol: {}", point.symbol_name);
                    points.push(point);
                    vectors_created += 1;
                    
                    // Batch upload every 100 vectors
                    if points.len() >= 100 {
                        self.upload_points(&points).await?;
                        points.clear();
                    }
                }
                _ => {}
            }
        }
        
        // Upload remaining vectors
        if !points.is_empty() {
            self.upload_points(&points).await?;
        }
        
        Ok(vectors_created)
    }

    /// Upload vectors to LanceDB
    async fn upload_points(&self, points: &[CodePoint]) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Uploading {} vectors to LanceDB", points.len());
        
        if points.is_empty() {
            return Ok(());
        }

        // Qwen/Qwen3-Embedding-4B has 2560 dimensions
        let vector_size = 2560;

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("vector", DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                vector_size
            ), false),
            Field::new("file_path", DataType::Utf8, false),
            Field::new("symbol_name", DataType::Utf8, false),
            Field::new("symbol_type", DataType::Utf8, false),
            Field::new("language", DataType::Utf8, false),
            Field::new("line_start", DataType::Int64, false),
            Field::new("line_end", DataType::Int64, false),
            Field::new("code_block", DataType::Utf8, false),
        ]));

        // Build arrays
        let mut id_builder = StringBuilder::new();
        let mut vector_builder = FixedSizeListBuilder::new(Float32Builder::new(), vector_size);
        let mut file_path_builder = StringBuilder::new();
        let mut symbol_name_builder = StringBuilder::new();
        let mut symbol_type_builder = StringBuilder::new();
        let mut language_builder = StringBuilder::new();
        let mut line_start_builder = Int64Builder::new();
        let mut line_end_builder = Int64Builder::new();
        let mut code_block_builder = StringBuilder::new();
        
        for p in points {
            id_builder.append_value(&p.id);
            
            // Ensure vector size matches
            if p.vector.len() != vector_size as usize {
                error!("Vector size mismatch: expected {}, got {}", vector_size, p.vector.len());
                continue;
            }

            vector_builder.values().append_slice(&p.vector);
            vector_builder.append(true);
            
            file_path_builder.append_value(&p.file_path);
            symbol_name_builder.append_value(&p.symbol_name);
            symbol_type_builder.append_value(&p.symbol_type);
            language_builder.append_value(&p.language);
            line_start_builder.append_value(p.line_start);
            line_end_builder.append_value(p.line_end);
            code_block_builder.append_value(&p.code_block);
        }
        
        let batch = RecordBatch::try_new(schema.clone(), vec![
            Arc::new(id_builder.finish()),
            Arc::new(vector_builder.finish()),
            Arc::new(file_path_builder.finish()),
            Arc::new(symbol_name_builder.finish()),
            Arc::new(symbol_type_builder.finish()),
            Arc::new(language_builder.finish()),
            Arc::new(line_start_builder.finish()),
            Arc::new(line_end_builder.finish()),
            Arc::new(code_block_builder.finish()),
        ])?;
        
        let table = self.connection.open_table(&self.table_name).execute().await?;
        
        let batches = vec![Ok(batch)];
        let batch_iter = RecordBatchIterator::new(batches, schema.clone());
        table.add(batch_iter).execute().await?;
        
        debug!("Upload completed");
        Ok(())
    }
}

/// Run vectorize command
pub async fn run_vectorize(path: String, collection: String, db_path: String) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting vectorize command");
    info!("Path: {}", path);
    info!("Collection: {}", collection);
    info!("LanceDB Path: {}", db_path);
    
    // Create vectorize service
    let service = VectorizeService::new(&db_path, collection).await?;
    
    // Ensure collection exists
    service.ensure_collection().await?;
    
    // Vectorize directory
    service.vectorize_directory(&path).await?;
    
    info!("Vectorize command completed successfully");
    Ok(())
}
