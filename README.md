# CodeGraph Core

A powerful Rust-based tool for analyzing code dependencies, building code graphs, and creating word embedding vector indices for semantic code search. CodeGraph Core supports multiple programming languages and provides both CLI and HTTP API interfaces for comprehensive code analysis.

## Features

### 🔍 Code Analysis & Graph Construction
- **Multi-language Support**: Analyze code in Rust, Python, JavaScript/TypeScript, Go, C++, and Java
- **Dependency Graph Generation**: Build comprehensive call graphs and dependency relationships
- **AST-based Parsing**: Uses Tree-sitter for accurate syntax analysis
- **Incremental Analysis**: Support for incremental code analysis and updates

### 🧠 Word Embedding & Vector Indexing
- **Semantic Code Search**: Generate word embeddings for code blocks using external embedding services
- **Vector Database Integration**: Store and query code embeddings using Qdrant vector database
- **Function & Class Vectorization**: Create semantic representations of functions and classes
- **Batch Processing**: Efficiently process entire codebases for vectorization

### 📊 Code Graph Visualization
- **Interactive Web Interface**: Beautiful web-based visualization of function call graphs
- **ECharts Integration**: Rich, interactive graph visualizations with zoom, pan, and filtering
- **Hierarchical Views**: Display code relationships in tree and graph formats
- **Real-time Updates**: Dynamic graph updates and exploration

### 🚀 HTTP API & CLI
- **RESTful API**: Complete HTTP API for integration with other tools
- **Command Line Interface**: Powerful CLI for batch processing and automation
- **Multiple Storage Formats**: Support for JSON and binary serialization
- **CORS Support**: Web-friendly API with cross-origin support

<img width="720" src="assets/demo.gif" alt="graph demo"/><br>

## Installation

### Prerequisites

- **Rust**: Install Rust 1.70+ from [rustup.rs](https://rustup.rs/)
- **Qdrant** (for vector indexing): Install Qdrant vector database
- **Embedding Service** (optional): HTTP service for generating embeddings

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd codegraph-core

# Build the project
cargo build --release

# The binary will be available at target/release/codegraph-cli
```

### Dependencies

The project uses the following key dependencies:
- `petgraph`: Graph data structures and algorithms
- `tree-sitter`: Syntax parsing for multiple languages
- `qdrant-client`: Vector database integration
- `axum`: HTTP server framework
- `tokio`: Async runtime

## Usage

### Command Line Interface

#### 1. Start HTTP Server

```bash
# Start server on default port (8080)
./target/release/codegraph-cli server

# Start server on custom address
./target/release/codegraph-cli server --address 127.0.0.1:3000

# Use different storage mode
./target/release/codegraph-cli server --storage-mode binary
```

#### 2. Vectorize Codebase

```bash
# Vectorize a directory and store in Qdrant
./target/release/codegraph-cli vectorize \
  --path /path/to/your/project \
  --collection my_code_collection \
  --qdrant-url http://localhost:6334
```

### HTTP API

#### Build Code Graph

```bash
curl -X POST http://localhost:8080/build_graph \
  -H "Content-Type: application/json" \
  -d '{
    "project_dir": "/path/to/your/project",
    "force_rebuild": true,
    "exclude_patterns": ["target", ".git", "node_modules"]
  }'
```

#### Query Call Graph

```bash
curl -X POST http://localhost:8080/query_call_graph \
  -H "Content-Type: application/json" \
  -d '{
    "filepath": "/path/to/file.rs",
    "function_name": "main",
    "max_depth": 3
  }'
```

#### Query Code Snippet

```bash
curl -X POST http://localhost:8080/query_code_snippet \
  -H "Content-Type: application/json" \
  -d '{
    "filepath": "/path/to/file.rs",
    "function_name": "process_data",
    "include_context": true,
    "context_lines": 5
  }'
```

### Web Interface

1. Start the HTTP server:
   ```bash
   ./target/release/codegraph-cli server
   ```

2. Open your browser and navigate to `http://localhost:8080`

3. Use the web interface to:
   - Build code graphs for your projects
   - Visualize function call relationships
   - Explore code dependencies interactively
   - Navigate through hierarchical code structures

## Word Embedding Vector Index

### Setup Qdrant

```bash
# Using Docker
docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant

# Or install locally following Qdrant documentation
```

### Setup Embedding Service

The vectorization feature requires an HTTP embedding service running on `http://localhost:9200/embedding`. The service should:

- Accept POST requests with JSON payload: `{"content": "code_block"}`
- Return embeddings in format: `[{"embedding": [[vector_values]]}]`
- Support 768-dimensional vectors (configurable)

### Vectorization Process

1. **Code Parsing**: Uses Tree-sitter to extract functions and classes
2. **Content Extraction**: Extracts code blocks with context
3. **Embedding Generation**: Sends code to embedding service
4. **Vector Storage**: Stores embeddings in Qdrant with metadata
5. **Batch Processing**: Processes files in batches for efficiency

### Vector Metadata

Each vector includes metadata:
- `file_path`: Source file location
- `symbol_name`: Function or class name
- `symbol_type`: Type of code symbol
- `language`: Programming language
- `line_start`/`line_end`: Source location
- `code_block`: Original code content

## Code Graph Construction

### Supported Languages

| Language | Functions | Classes | Imports | Comments |
|----------|-----------|---------|---------|----------|
| Rust | ✅ | ✅ | ✅ | ✅ |
| Python | ✅ | ✅ | ✅ | ✅ |
| JavaScript/TypeScript | ✅ | ✅ | ✅ | ✅ |
| Go | ✅ | ✅ | ✅ | ✅ |
| C++ | ✅ | ✅ | ✅ | ✅ |
| Java | ✅ | ✅ | ✅ | ✅ |

### Graph Features

- **Call Relationships**: Function-to-function call mappings
- **Inheritance Hierarchies**: Class inheritance and interface implementations
- **Import Dependencies**: Module and package dependencies
- **Cross-file References**: Inter-file function and class usage
- **Namespace Analysis**: Proper namespace and scope handling

## Code Graph Visualization

### Interactive Features

- **Zoom & Pan**: Navigate large graphs easily
- **Node Highlighting**: Highlight related functions on hover
- **Edge Styling**: Different styles for different relationship types
- **Filtering**: Filter by file, function name, or depth
- **Layout Options**: Force-directed and hierarchical layouts

### Visualization Types

1. **Call Graph**: Function call relationships
2. **Hierarchical Tree**: Tree-like dependency structure
3. **File-based View**: Organize by file structure
4. **Class Diagrams**: Object-oriented relationship views

### Customization

- **Max Depth**: Control graph complexity
- **File Filtering**: Focus on specific files or directories
- **Function Selection**: Start from specific functions
- **Visual Themes**: Multiple color schemes and layouts

## Configuration

### Storage Modes

- `json`: Human-readable JSON format
- `binary`: Compact binary format using bincode
- `both`: Store in both formats

### Environment Variables

```bash
# Qdrant configuration
QDRANT_URL=http://localhost:6334

# Embedding service
EMBEDDING_SERVICE_URL=http://localhost:9200/embedding

# Server configuration
SERVER_ADDRESS=127.0.0.1:8080
```

## API Reference

### Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| POST | `/build_graph` | Build code graph |
| POST | `/query_call_graph` | Query call relationships |
| POST | `/query_code_snippet` | Get code snippets |
| POST | `/query_hierarchical_graph` | Get hierarchical view |
| GET | `/draw_call_graph` | Web visualization |
| POST | `/investigate_repo` | Repository analysis |

### Response Format

All API responses follow this format:

```json
{
  "success": true,
  "data": {
    // Response data
  }
}
```

## Development

### Running Tests

```bash
# Run unit tests
cargo test

# Run functional tests
cd tests && ./run_functional_tests.sh

# Test HTTP endpoints
cd scripts && python3 test_http_endpoints.py
```

### Project Structure

```
src/
├── cli/           # Command line interface
├── codegraph/     # Core graph functionality
├── http/          # HTTP server and handlers
├── services/      # Business logic services
└── storage/       # Data persistence layer
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Tree-sitter for excellent parsing capabilities
- Qdrant for vector database functionality
- ECharts for beautiful visualizations
- The Rust community for amazing tools and libraries