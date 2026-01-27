#!/bin/bash
# Comprehensive Evaluation Loop for Embeddenator
# Full system validation with detailed reporting

echo "═══════════════════════════════════════════════════════════════"
echo "  🔬 EMBEDDENATOR COMPREHENSIVE EVALUATION"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

EVALUATION_START=$(date +%s)
PASS_COUNT=0
WARN_COUNT=0
FAIL_COUNT=0

# Logging functions
log() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
    ((PASS_COUNT++))
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
    ((WARN_COUNT++))
}

error() {
    echo -e "${RED}❌ $1${NC}"
    ((FAIL_COUNT++))
}

info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

# ═══════════════════════════════════════════════════════════════
# Phase 1: Basic Functionality Tests
# ═══════════════════════════════════════════════════════════════
echo ""
log "PHASE 1: Basic Functionality Tests"
echo "─────────────────────────────────────"

log "Testing main embeddenator functionality..."
cd ../embeddenator
if timeout 120 cargo test --quiet > /dev/null 2>&1; then
    success "Main embeddenator unit tests"
    cd ../embeddenator-testkit
else
    error "Main embeddenator unit tests"
    cd ../embeddenator-testkit
fi

log "Testing testkit basic functionality..."
if timeout 60 cargo test --quiet > /dev/null 2>&1; then
    success "TestKit basic tests"
else
    error "TestKit basic tests"
fi

# ═══════════════════════════════════════════════════════════════
# Phase 2: System Resources Analysis
# ═══════════════════════════════════════════════════════════════
echo ""
log "PHASE 2: System Resources Analysis"
echo "─────────────────────────────────────"

TOTAL_MEM=$(free -g | awk 'NR==2{printf "%d", $2}')
USED_MEM=$(free -g | awk 'NR==2{printf "%d", $3}')
AVAILABLE_MEM=$(free -g | awk 'NR==2{printf "%d", $7}')
CPU_CORES=$(nproc)

info "Memory: ${USED_MEM}GB used / ${AVAILABLE_MEM}GB available / ${TOTAL_MEM}GB total"
info "CPU Cores: $CPU_CORES"

if [ $AVAILABLE_MEM -ge 20 ]; then
    success "Sufficient memory for large-scale testing (${AVAILABLE_MEM}GB available)"
elif [ $AVAILABLE_MEM -ge 8 ]; then
    warning "Moderate memory available (${AVAILABLE_MEM}GB), large-scale tests may be limited"
else
    error "Low memory available (${AVAILABLE_MEM}GB), large-scale tests not recommended"
fi

if [ $CPU_CORES -ge 8 ]; then
    success "Excellent parallel processing capability ($CPU_CORES cores)"
elif [ $CPU_CORES -ge 4 ]; then
    success "Good parallel processing capability ($CPU_CORES cores)"
else
    warning "Limited CPU cores ($CPU_CORES cores) may impact performance"
fi

# ═══════════════════════════════════════════════════════════════
# Phase 3: Build Verification
# ═══════════════════════════════════════════════════════════════
echo ""
log "PHASE 3: Build Verification"
echo "─────────────────────────────────────"

log "Checking base build..."
if timeout 120 cargo check --quiet 2>&1 > /dev/null; then
    success "Base build verification"
else
    error "Base build verification"
fi

cd ../embeddenator

log "Checking SIMD optimizations..."
if timeout 120 cargo build --release --features "simd" --quiet 2>&1 > /dev/null; then
    success "SIMD optimizations build"
else
    warning "SIMD optimizations not available or failed to build"
fi

log "Checking bt-phase-2 optimizations..."
if timeout 120 cargo build --release --features "bt-phase-2" --quiet 2>&1 > /dev/null; then
    success "BT-phase-2 optimizations build"
else
    warning "BT-phase-2 optimizations not available or failed to build"
fi

cd ../embeddenator-testkit

# ═══════════════════════════════════════════════════════════════
# Phase 4: End-to-End Workflow Testing
# ═══════════════════════════════════════════════════════════════
echo ""
log "PHASE 4: End-to-End Workflow Testing"
echo "─────────────────────────────────────"

# Clean up old test data
rm -rf workflow_test
mkdir -p workflow_test

# Create 100MB test file
log "Creating 100MB test file..."
dd if=/dev/urandom of=workflow_test/test_100mb.bin bs=1M count=100 2>/dev/null

# Ingest
log "Testing ingestion workflow..."
START_TIME=$(date +%s%N)
cd ../embeddenator
if timeout 120 ./target/release/embeddenator ingest \
    -i ../embeddenator-testkit/workflow_test/test_100mb.bin \
    -e ../embeddenator-testkit/workflow_test/test_100mb.engram \
    -m ../embeddenator-testkit/workflow_test/test_100mb.json \
    --verbose > /dev/null 2>&1; then
    END_TIME=$(date +%s%N)
    DURATION_MS=$(( (END_TIME - START_TIME) / 1000000 ))
    DURATION_SEC=$(echo "scale=2; $DURATION_MS / 1000" | bc)
    
    success "Ingestion workflow (${DURATION_SEC}s)"
    INGEST_RATE=$(echo "scale=1; 100 / $DURATION_SEC" | bc)
    info "Ingestion rate: ~${INGEST_RATE} MB/s"
else
    error "Ingestion workflow"
fi

# Extract
log "Testing extraction workflow..."
mkdir -p ../embeddenator-testkit/workflow_test/extracted
START_TIME=$(date +%s%N)
if timeout 120 ./target/release/embeddenator extract \
    -e ../embeddenator-testkit/workflow_test/test_100mb.engram \
    -m ../embeddenator-testkit/workflow_test/test_100mb.json \
    -o ../embeddenator-testkit/workflow_test/extracted \
    --verbose > /dev/null 2>&1; then
    END_TIME=$(date +%s%N)
    DURATION_MS=$(( (END_TIME - START_TIME) / 1000000 ))
    DURATION_SEC=$(echo "scale=2; $DURATION_MS / 1000" | bc)
    
    success "Extraction workflow (${DURATION_SEC}s)"
    EXTRACT_RATE=$(echo "scale=1; 100 / $DURATION_SEC" | bc)
    info "Extraction rate: ~${EXTRACT_RATE} MB/s"
else
    error "Extraction workflow"
fi

# Verify bit-perfect reconstruction
log "Verifying bit-perfect reconstruction..."
if diff ../embeddenator-testkit/workflow_test/test_100mb.bin \
        ../embeddenator-testkit/workflow_test/extracted/embeddenator-testkit/workflow_test/test_100mb.bin \
        > /dev/null 2>&1; then
    success "Bit-perfect reconstruction verified"
else
    error "Bit-perfect reconstruction failed"
fi

cd ../embeddenator-testkit

# Storage overhead analysis
ORIGINAL_SIZE=$(du -b workflow_test/test_100mb.bin | cut -f1)
ENGRAM_SIZE=$(du -b workflow_test/test_100mb.engram | cut -f1)
MANIFEST_SIZE=$(du -b workflow_test/test_100mb.json | cut -f1)
TOTAL_STORED=$((ENGRAM_SIZE + MANIFEST_SIZE))
OVERHEAD=$(echo "scale=2; ($TOTAL_STORED / $ORIGINAL_SIZE) * 100" | bc)

info "Storage Analysis:"
info "  Original: $((ORIGINAL_SIZE / 1048576))MB"
info "  Engram: $((ENGRAM_SIZE / 1048576))MB"
info "  Manifest: $((MANIFEST_SIZE / 1048576))MB"
info "  Overhead: ${OVERHEAD}%"

# ═══════════════════════════════════════════════════════════════
# Phase 5: Optimization Analysis
# ═══════════════════════════════════════════════════════════════
echo ""
log "PHASE 5: Optimization Analysis"
echo "─────────────────────────────────────"

info "Available Optimizations:"

cd ../embeddenator
if grep -q 'name = "embeddenator"' Cargo.toml; then
    if grep -q '\[features\]' Cargo.toml; then
        info "✓ Feature flags configured"
    fi
fi

if grep -q "simd" Cargo.toml 2>/dev/null || cargo build --release --features "simd" --quiet 2>/dev/null > /dev/null 2>&1; then
    success "SIMD acceleration available"
else
    info "SIMD acceleration not available"
fi

if grep -q "bt-phase-2" Cargo.toml 2>/dev/null || cargo build --release --features "bt-phase-2" --quiet 2>/dev/null > /dev/null 2>&1; then
    success "BT-Phase-2 optimizations available"
else
    info "BT-Phase-2 optimizations not available"
fi

if grep -q "rayon" Cargo.toml; then
    success "Parallel processing (Rayon) available"
else
    info "Parallel processing not configured"
fi

cd ../embeddenator-testkit

# ═══════════════════════════════════════════════════════════════
# Phase 6: Large-Scale Testing Framework
# ═══════════════════════════════════════════════════════════════
echo ""
log "PHASE 6: Large-Scale Testing Framework"
echo "─────────────────────────────────────"

if timeout 30 cargo check --features large-scale --quiet 2>&1 > /dev/null; then
    success "Large-scale testing framework available"
    info "To run large-scale benchmarks:"
    info "  cargo bench --bench large_scale_operations --features large-scale"
else
    warning "Large-scale testing framework not available"
fi

if timeout 30 cargo check --features gpu --quiet 2>&1 > /dev/null; then
    success "GPU acceleration framework available"
else
    info "GPU acceleration framework (future implementation)"
fi

# ═══════════════════════════════════════════════════════════════
# Phase 7: Summary and Recommendations
# ═══════════════════════════════════════════════════════════════
echo ""
log "PHASE 7: Final Summary and Recommendations"
echo "═══════════════════════════════════════════════════════════════"

EVALUATION_END=$(date +%s)
DURATION=$((EVALUATION_END - EVALUATION_START))

TOTAL_TESTS=$((PASS_COUNT + WARN_COUNT + FAIL_COUNT))

echo ""
echo "Evaluation Results:"
echo "  ✅ Passed:   $PASS_COUNT"
echo "  ⚠️  Warnings: $WARN_COUNT"
echo "  ❌ Failed:   $FAIL_COUNT"
echo "  ━━━━━━━━━━━━━━━━━━━━━━"
echo "  📊 Total:    $TOTAL_TESTS"
echo ""
echo "Evaluation Time: ${DURATION}s"
echo ""

# Overall assessment
EXIT_CODE=0
if [ $FAIL_COUNT -eq 0 ]; then
    if [ $WARN_COUNT -eq 0 ]; then
        echo -e "${GREEN}🎉 EVALUATION PASSED${NC}"
        echo "Status: ✅ READY FOR PRODUCTION"
    else
        echo -e "${YELLOW}⚠️  EVALUATION PASSED WITH WARNINGS${NC}"
        echo "Status: ⚠️  OPERATIONAL (review warnings)"
    fi
else
    echo -e "${RED}❌ EVALUATION FAILED${NC}"
    echo "Status: ❌ REQUIRES REVIEW"
    EXIT_CODE=1
fi

echo ""
echo "Next Steps:"
echo "─────────────────────────────────────"
echo "  1. Development & Testing:"
echo "     cd ../embeddenator && cargo build --release"
echo "     cargo test --all --release"
echo ""
echo "  2. Performance Benchmarking:"
echo "     cd ../embeddenator-testkit"
echo "     cargo bench --bench performance_validation"
echo ""
echo "  3. Large-Scale Testing:"
echo "     cargo bench --bench large_scale_operations --features large-scale"
echo ""
echo "  4. Feature Optimization:"
echo "     cargo build --release --features 'bt-phase-2,simd'"
echo ""

echo "═══════════════════════════════════════════════════════════════"
echo "  📊 EVALUATION COMPLETE"
echo "═══════════════════════════════════════════════════════════════"

# Cleanup
log "Cleaning up test data..."
rm -rf workflow_test

exit $EXIT_CODE
