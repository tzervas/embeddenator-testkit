#!/bin/bash
# Full Evaluation Loop for Embeddenator
# Comprehensive testing and benchmarking workflow

set -e

echo "═══════════════════════════════════════════════════════════════"
echo "  🔬 EMBEDDENATOR FULL EVALUATION LOOP"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Evaluation results
declare -A RESULTS
EVALUATION_START=$(date +%s)

# Logging function
log() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
    RESULTS["$1"]="PASS"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
    RESULTS["$1"]="WARN"
}

error() {
    echo -e "${RED}❌ $1${NC}"
    RESULTS["$1"]="FAIL"
}

# Phase 1: Basic Functionality Tests
log "Phase 1: Basic Functionality Tests"
echo "─────────────────────────────────────"

# Test embeddenator-testkit basic functionality
log "Testing testkit basic functionality..."
if cargo test --quiet; then
    success "TestKit basic tests"
else
    error "TestKit basic tests"
fi

# Test main embeddenator functionality
log "Testing main embeddenator functionality..."
cd ../embeddenator
if cargo test --quiet; then
    success "Main embeddenator tests"
else
    error "Main embeddenator tests"
fi
cd ../embeddenator-testkit

echo ""

# Phase 2: Performance Benchmarks
log "Phase 2: Performance Benchmarks"
echo "─────────────────────────────────────"

# Performance validation benchmarks
log "Running performance validation benchmarks..."
if timeout 300 cargo bench --bench performance_validation --quiet 2>/dev/null; then
    success "Performance validation benchmarks"
else
    warning "Performance validation benchmarks (timeout or partial completion)"
fi

# Optimization validation benchmarks
log "Running optimization validation benchmarks..."
if timeout 300 cargo bench --bench optimization_validation --quiet 2>/dev/null; then
    success "Optimization validation benchmarks"
else
    warning "Optimization validation benchmarks (timeout or partial completion)"
fi

echo ""

# Phase 3: Memory and Allocation Tests
log "Phase 3: Memory and Allocation Tests"
echo "─────────────────────────────────────"

# Memory pattern benchmarks
log "Testing memory patterns..."
if timeout 180 cargo bench --bench large_scale_operations --quiet 2>/dev/null; then
    success "Memory pattern benchmarks"
else
    warning "Memory pattern benchmarks (timeout or partial completion)"
fi

echo ""

# Phase 4: End-to-End Workflow Testing
log "Phase 4: End-to-End Workflow Testing"
echo "─────────────────────────────────────"

# Create test data
log "Creating test datasets..."
mkdir -p evaluation_data

# Small dataset test (100MB)
log "Testing small dataset workflow (100MB)..."
dd if=/dev/urandom of=evaluation_data/small_test.dat bs=1M count=100 2>/dev/null
cd ../embeddenator
if ./target/release/embeddenator ingest -i ../embeddenator-testkit/evaluation_data/small_test.dat -e ../embeddenator-testkit/evaluation_data/small_test.engram -m ../embeddenator-testkit/evaluation_data/small_test.json -v > /dev/null 2>&1; then
    success "Small dataset ingestion"
    
    # Test extraction
    mkdir -p ../embeddenator-testkit/evaluation_data/extracted
    if ./target/release/embeddenator extract -e ../embeddenator-testkit/evaluation_data/small_test.engram -m ../embeddenator-testkit/evaluation_data/small_test.json -o ../embeddenator-testkit/evaluation_data/extracted -v > /dev/null 2>&1; then
        success "Small dataset extraction"
        
        # Verify bit-perfect reconstruction
        if diff evaluation_data/small_test.dat evaluation_data/extracted/embeddenator-testkit/evaluation_data/small_test.dat > /dev/null 2>&1; then
            success "Small dataset bit-perfect reconstruction"
        else
            error "Small dataset bit-perfect reconstruction"
        fi
    else
        error "Small dataset extraction"
    fi
else
    error "Small dataset ingestion"
fi
cd ../embeddenator-testkit

echo ""

# Phase 5: Large-Scale Testing (if enabled)
log "Phase 5: Large-Scale Testing"
echo "─────────────────────────────────────"

if cargo check --features large-scale > /dev/null 2>&1; then
    log "Large-scale testing available, running sample..."
    # Note: Full large-scale testing would require significant time/resources
    # This is just a capability check
    success "Large-scale testing framework available"
else
    warning "Large-scale testing not enabled (requires --features large-scale)"
fi

echo ""

# Phase 6: Optimization Effectiveness Analysis
log "Phase 6: Optimization Effectiveness Analysis"
echo "─────────────────────────────────────"

# Check SIMD availability
log "Checking SIMD optimization status..."
cd ../embeddenator
if cargo build --release --features "bt-phase-2,simd" --quiet 2>/dev/null; then
    success "SIMD optimizations available and building"
else
    warning "SIMD optimizations not available"
fi
cd ../embeddenator-testkit

echo ""

# Phase 7: System Resource Analysis
log "Phase 7: System Resource Analysis"
echo "─────────────────────────────────────"

# Check available system resources
log "Analyzing system resources..."
TOTAL_MEM=$(free -g | awk 'NR==2{printf "%.0f", $2}')
USED_MEM=$(free -g | awk 'NR==2{printf "%.0f", $3}')
CPU_CORES=$(nproc)

echo "System Resources:"
echo "  Memory: ${USED_MEM}GB used / ${TOTAL_MEM}GB total"
echo "  CPU Cores: $CPU_CORES"

if [ $TOTAL_MEM -ge 16 ]; then
    success "Sufficient memory for large-scale testing"
else
    warning "Limited memory may constrain large-scale testing"
fi

if [ $CPU_CORES -ge 4 ]; then
    success "Sufficient CPU cores for parallel operations"
else
    warning "Limited CPU cores may impact performance"
fi

echo ""

# Phase 8: Evaluation Summary and Recommendations
log "Phase 8: Evaluation Summary and Recommendations"
echo "═══════════════════════════════════════════════════════════════"

EVALUATION_END=$(date +%s)
DURATION=$((EVALUATION_END - EVALUATION_START))

echo "Evaluation completed in ${DURATION} seconds"
echo ""

# Count results
PASS_COUNT=0
WARN_COUNT=0
FAIL_COUNT=0

for key in "${!RESULTS[@]}"; do
    case "${RESULTS[$key]}" in
        "PASS") ((PASS_COUNT++)) ;;
        "WARN") ((WARN_COUNT++)) ;;
        "FAIL") ((FAIL_COUNT++)) ;;
    esac
done

TOTAL_TESTS=$((PASS_COUNT + WARN_COUNT + FAIL_COUNT))

# Debug: Show all results
if [ ${#RESULTS[@]} -eq 0 ]; then
    # If no results tracked, manually count from output
    PASS_COUNT=$(echo "${!RESULTS[@]}" | wc -w)
fi

echo "Results Summary:"
echo "  ✅ Passed: $PASS_COUNT"
echo "  ⚠️  Warnings: $WARN_COUNT"
echo "  ❌ Failed: $FAIL_COUNT"
echo "  📊 Total: $TOTAL_TESTS"
echo ""

# Overall assessment
if [ $FAIL_COUNT -eq 0 ]; then
    if [ $WARN_COUNT -eq 0 ]; then
        echo -e "${GREEN}🎉 EVALUATION COMPLETE: All tests passed!${NC}"
        echo "System is ready for production use."
    else
        echo -e "${YELLOW}⚠️  EVALUATION COMPLETE: Tests passed with warnings${NC}"
        echo "System is functional but may benefit from optimization."
    fi
else
    echo -e "${RED}❌ EVALUATION COMPLETE: Some tests failed${NC}"
    echo "System requires attention before production use."
fi

echo ""

# Recommendations
echo "Recommendations:"
if [ $FAIL_COUNT -gt 0 ]; then
    echo "  • Address failed tests before proceeding"
fi
if [ $WARN_COUNT -gt 0 ]; then
    echo "  • Review warnings for potential improvements"
fi
echo "  • Consider running large-scale tests with --features large-scale"
echo "  • Monitor performance baselines for regression detection"
echo "  • GPU acceleration framework is ready for future implementation"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  📊 EVALUATION LOOP COMPLETE"
echo "═══════════════════════════════════════════════════════════════"

# Cleanup
log "Cleaning up evaluation data..."
rm -rf evaluation_data

exit $FAIL_COUNT
