# Test hierarchical graph API
test-hierarchical-api:
	@echo "🧪 Testing Hierarchical Graph API..."
	@cd scripts && python3 test_hierarchical_api.py

# Demo scheme1 output format
demo-scheme1:
	@echo "🎯 Demonstrating Scheme1 Output Format..."
	@cd scripts && python3 demo_hierarchical_output.py

# Build and test hierarchical API
build-test-hierarchical: build test-hierarchical-api
	@echo "✅ Hierarchical API build and test completed" 