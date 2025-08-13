# CodeGraph CHA Mode Usage Guide

## Overview

CodeGraph now supports Class Hierarchy Analysis (CHA) mode for building more precise call graphs. CHA mode provides better accuracy by considering class inheritance hierarchies and method dispatch, reducing false positive call edges.

## Available Analysis Modes

### 1. Standard Mode (Default)
- Uses the traditional CodeGraphAnalyzer
- Simple name-based function resolution
- Fast but may include false positives

### 2. Simple CHA Mode
- Uses simplified Class Hierarchy Analysis
- Avoids complex type dependencies
- Good balance of accuracy and performance
- **Recommended for most use cases**

### 3. Full CHA Mode
- Uses complete Class Hierarchy Analysis
- Advanced type-aware analysis
- Best accuracy but requires more resources
- **Use for critical analysis tasks**

## HTTP API Usage

### Build Graph Endpoint

**POST** `/build_graph`

#### Request Body
```json
{
  "project_dir": "/path/to/your/project",
  "analysis_mode": "simple_cha",
  "force_rebuild": true,
  "exclude_patterns": ["target", "node_modules"]
}
```

#### Analysis Mode Values
- `"standard"` - Standard analysis mode
- `"simple_cha"` - Simple CHA mode
- `"cha"` - Full CHA mode

#### Response
```json
{
  "success": true,
  "data": {
    "project_id": "abc123...",
    "total_files": 150,
    "total_functions": 1200,
    "build_time_ms": 2500,
    "cache_hit_rate": 0.0,
    "analysis_mode": "simple_cha"
  }
}
```

## Command Line Usage

### Start HTTP Server
```bash
cargo run -- server
```

### Test CHA Mode
```bash
# Make the test script executable
chmod +x test_cha_mode.py

# Run the test
python3 test_cha_mode.py
```

## Example Usage

### Python Example
```python
import requests

# Build graph with Simple CHA mode
response = requests.post("http://127.0.0.1:8080/build_graph", json={
    "project_dir": "/path/to/project",
    "analysis_mode": "simple_cha",
    "force_rebuild": True
})

if response.status_code == 200:
    data = response.json()
    print(f"Built graph with {data['data']['total_functions']} functions")
    print(f"Analysis mode: {data['data']['analysis_mode']}")
```

### cURL Example
```bash
curl -X POST http://127.0.0.1:8080/build_graph \
  -H "Content-Type: application/json" \
  -d '{
    "project_dir": "/path/to/project",
    "analysis_mode": "cha",
    "force_rebuild": true
  }'
```

## Performance Characteristics

| Mode | Accuracy | Speed | Memory Usage | Use Case |
|------|----------|-------|--------------|----------|
| Standard | Medium | Fast | Low | Quick analysis, development |
| Simple CHA | High | Medium | Medium | Production analysis, most projects |
| Full CHA | Very High | Slow | High | Critical analysis, research |

## When to Use Each Mode

### Use Standard Mode When:
- You need quick results
- Working on small projects
- Doing development/testing
- Memory is limited

### Use Simple CHA Mode When:
- You need accurate call graphs
- Working on production code
- Analyzing object-oriented code
- **This is the recommended default**

### Use Full CHA Mode When:
- You need maximum accuracy
- Analyzing complex inheritance hierarchies
- Doing research or critical analysis
- Have sufficient computational resources

## Troubleshooting

### Common Issues

1. **Server not responding**
   ```bash
   # Check if server is running
   curl http://127.0.0.1:8080/health
   ```

2. **CHA mode fails**
   - Check that the project directory exists
   - Ensure the project contains supported file types
   - Check server logs for detailed error messages

3. **Memory issues with Full CHA**
   - Use Simple CHA mode instead
   - Reduce project size
   - Increase system memory

### Error Messages

- `"Failed to build CHA call graph"` - CHA analysis failed
- `"Failed to build Simple CHA call graph"` - Simple CHA analysis failed
- `"HTTP 500"` - Internal server error, check logs

## Advanced Configuration

### Environment Variables
```bash
# Increase timeout for large projects
export CODEGRAPH_TIMEOUT=600

# Set memory limits
export CODEGRAPH_MAX_MEMORY=4GB
```

### Custom Analysis
You can extend the CHA implementation by:
1. Adding new language-specific analyzers
2. Implementing custom method resolution strategies
3. Adding new call site extraction methods

## Support

For issues or questions:
1. Check the server logs
2. Review the CHA_README.md file
3. Check the planning.md for implementation details
4. Run the test script to verify functionality

## Migration from Standard Mode

If you're currently using standard mode:

1. **Test with Simple CHA first** - It's the safest upgrade path
2. **Compare results** - Verify that the new mode produces expected results
3. **Update your scripts** - Change `analysis_mode` to `"simple_cha"`
4. **Monitor performance** - CHA modes may take longer but provide better accuracy

## Future Enhancements

Planned improvements for CHA mode:
- Language-specific optimizations
- Incremental analysis support
- Better caching strategies
- Performance profiling tools 