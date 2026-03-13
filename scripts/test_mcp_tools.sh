#!/usr/bin/env bash
# Test suite for MEMORY_P v3.0 new MCP tools
# Location: scripts/test_mcp_tools.sh
# Run: bash scripts/test_mcp_tools.sh

set -e

MCP_URL="http://localhost:4040/mcp"
ENDPOINT_URL="${MCP_URL}/tools/call"
ID=1

echo "════════════════════════════════════════════════════════════════════════"
echo "  MEMORY_P v3.0 - MCP Tools Test Suite"
echo "════════════════════════════════════════════════════════════════════════"
echo ""

# Helper function
call_tool() {
    local tool_name=$1
    local args=$2
    local id=$3
    
    echo "🔧 Calling: $tool_name"
    echo "   Arguments: $args"
    
    curl -s -X POST "$MCP_URL" \
        -H "Content-Type: application/json" \
        -d "{
            \"jsonrpc\": \"2.0\",
            \"method\": \"tools/call\",
            \"params\": {
                \"name\": \"$tool_name\",
                \"arguments\": $args
            },
            \"id\": $id
        }" | jq . 2>/dev/null || echo "(jq parsing failed)"
    
    echo ""
}

# ═══════════════════════════════════════════════════════════════════════════
# TEST 1: mojo_dot_product (MOJO FFI)
# ═══════════════════════════════════════════════════════════════════════════

echo "TEST 1: mojo_dot_product - Compute a·b using Mojo SIMD"
call_tool "mojo_dot_product" \
    '{"a": [1.0, 2.0, 3.0, 4.0, 5.0], "b": [2.0, 3.0, 4.0, 5.0, 6.0]}' \
    $((ID++))

# Expected: {result: 70.0}
# Math: (1*2) + (2*3) + (3*4) + (4*5) + (5*6) = 2 + 6 + 12 + 20 + 30 = 70.0

# ═══════════════════════════════════════════════════════════════════════════
# TEST 2: mojo_cosine_similarity (MOJO FFI)
# ═══════════════════════════════════════════════════════════════════════════

echo "TEST 2: mojo_cosine_similarity - Compute cos(a,b) using Mojo"
call_tool "mojo_cosine_similarity" \
    '{"a": [1.0, 0.0, 0.0], "b": [1.0, 0.0, 0.0]}' \
    $((ID++))

# Expected: {result: 1.0} (identical vectors = similarity 1.0)

# ═══════════════════════════════════════════════════════════════════════════
# TEST 3: predict_trajectory (Chaos prediction)
# ═══════════════════════════════════════════════════════════════════════════

echo "TEST 3: predict_trajectory - Forecast next states from history"
call_tool "predict_trajectory" \
    '{"history": ["init", "search", "analyze", "predict"], "steps": 3}' \
    $((ID++))

# Expected: {trajectory: [{move, lyapunov, entropy}, ...], count: 3}

# ═══════════════════════════════════════════════════════════════════════════
# TEST 4: motor_route_query (Engine selection)
# ═══════════════════════════════════════════════════════════════════════════

echo "TEST 4: motor_route_query - Recommend search engine for query"
call_tool "motor_route_query" \
    '{"query": "find similar code patterns", "query_type": "semantic"}' \
    $((ID++))

# Expected: {query, query_type, recommended_motors: [...], fallback: "qdrant"}

# ═══════════════════════════════════════════════════════════════════════════
# TEST 5: ffi_test_all (FFI subsystem health check)
# ═══════════════════════════════════════════════════════════════════════════

echo "TEST 5: ffi_test_all - Test all FFI subsystems"
call_tool "ffi_test_all" '{}' $((ID++))

# Expected: {mojo_dot_product: (result or null), julia_entropy: (float), julia_chaos: (float or null), timestamp: "..."}

# ═══════════════════════════════════════════════════════════════════════════
# TEST 6: memory_persist (Session storage)
# ═══════════════════════════════════════════════════════════════════════════

echo "TEST 6: memory_persist - Save session to persistent storage"
call_tool "memory_persist" \
    '{"session_id": "test_session_001", "data": {"query": "test", "results": 42}}' \
    $((ID++))

# Expected: {session_id: "test_session_001", stored: true, timestamp: "...", note: "..."}

# ═══════════════════════════════════════════════════════════════════════════
# TEST 7: analysis_workspace_detailed (Chaos analysis)
# ═══════════════════════════════════════════════════════════════════════════

echo "TEST 7: analysis_workspace_detailed - Detailed chaos metrics"
call_tool "analysis_workspace_detailed" '{}' $((ID++))

# Expected: {basic: {...}, extensions_detail: {...}, chaos_summary: {is_chaotic: bool}}

# ═══════════════════════════════════════════════════════════════════════════
# TEST 8: ffi_status (Health check)
# ═══════════════════════════════════════════════════════════════════════════

echo "TEST 8: ffi_status - Check FFI availability"
call_tool "ffi_status" '{}' $((ID++))

# Expected: {zig: bool, mojo: bool, julia: bool, pony: bool, jax: bool}

# ═══════════════════════════════════════════════════════════════════════════

echo "════════════════════════════════════════════════════════════════════════"
echo "✅ Test suite complete"
echo ""
echo "Server should be running on port 4040:"
echo "  $ cargo run --bin memory_p_server"
echo ""
echo "Run this test script:"
echo "  $ bash scripts/test_mcp_tools.sh"
echo "════════════════════════════════════════════════════════════════════════"
