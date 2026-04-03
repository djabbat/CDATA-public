#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────────────────
# CDATA v3.0 — Cell-DT Digital Twin Simulator
# Main launcher script
# ──────────────────────────────────────────────────────────────────────────────
set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_DIR"

BOLD='\033[1m'; CYAN='\033[0;36m'; GREEN='\033[0;32m'; RED='\033[0;31m'; NC='\033[0m'

echo -e "${BOLD}${CYAN}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${CYAN}║   CDATA v3.0 — Cell-DT Digital Twin         ║${NC}"
echo -e "${BOLD}${CYAN}║   Centriolar Damage Accumulation Theory      ║${NC}"
echo -e "${BOLD}${CYAN}╚══════════════════════════════════════════════╝${NC}"
echo ""

# ── Menu ──────────────────────────────────────────────────────────────────────
if [[ -z "$1" ]]; then
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  gui         Launch Streamlit GUI (opens in browser)"
    echo "  gui-native  Launch egui Desktop GUI (native window)"
    echo "  sim         Run basic simulation (all 4 tissues, Round 7 fixes)"
    echo "  test        Run full test suite (400+ tests)"
    echo "  build       Build workspace (release)"
    echo "  check       cargo check (fast syntax check)"
    echo "  bench       Run simulation benchmarks"
    echo "  clean       Clean build artifacts"
    echo "  docs        Generate rustdoc documentation"
    echo "  python      Build PyO3 bindings (requires maturin)"
    echo ""
    echo "Examples:"
    echo "  bash run.sh sim"
    echo "  bash run.sh test"
    echo "  bash run.sh build"
    exit 0
fi

CMD="$1"

# ── Build check ───────────────────────────────────────────────────────────────
check_rust() {
    if ! command -v cargo &>/dev/null; then
        echo -e "${RED}Error: cargo not found. Install Rust: https://rustup.rs${NC}"
        exit 1
    fi
}

# ── Commands ──────────────────────────────────────────────────────────────────
case "$CMD" in
    sim)
        check_rust
        echo -e "${BOLD}Running basic simulation...${NC}"
        echo "(Round 7 fixes: B1-B5, M1-M3, L1-L3, C1-C4 applied)"
        echo ""
        cargo run --example basic_simulation --release \
            --manifest-path crates/cell_dt_validation/Cargo.toml 2>&1
        echo ""
        echo -e "${GREEN}✓ Simulation complete${NC}"
        ;;

    test)
        check_rust
        echo -e "${BOLD}Running test suite...${NC}"
        cargo test --workspace 2>&1
        echo ""
        echo -e "${GREEN}✓ Tests complete${NC}"
        ;;

    build)
        check_rust
        echo -e "${BOLD}Building workspace (release)...${NC}"
        cargo build --workspace --release 2>&1
        echo ""
        echo -e "${GREEN}✓ Build complete${NC}"
        ;;

    check)
        check_rust
        echo -e "${BOLD}Checking workspace...${NC}"
        cargo check --workspace 2>&1
        echo -e "${GREEN}✓ Check complete${NC}"
        ;;

    bench)
        check_rust
        echo -e "${BOLD}Running benchmarks...${NC}"
        cargo bench --workspace 2>&1
        ;;

    clean)
        check_rust
        echo -e "${BOLD}Cleaning build artifacts...${NC}"
        cargo clean 2>&1
        echo -e "${GREEN}✓ Clean complete${NC}"
        ;;

    docs)
        check_rust
        echo -e "${BOLD}Generating documentation...${NC}"
        cargo doc --workspace --no-deps --open 2>&1
        ;;

    gui)
        echo -e "${BOLD}Starting CDATA Streamlit GUI...${NC}"
        if ! command -v streamlit &>/dev/null; then
            echo "Installing streamlit..."
            pip install streamlit matplotlib numpy
        fi
        # Kill any stale instance on 8501
        fuser -k 8501/tcp 2>/dev/null || true
        sleep 1
        echo -e "${GREEN}Open in browser: http://localhost:8501${NC}"
        streamlit run gui/cdata_gui.py \
            --server.port 8501 \
            --server.address 0.0.0.0 \
            --server.headless true \
            --browser.gatherUsageStats false
        ;;

    gui-native)
        check_rust
        echo -e "${BOLD}Building and launching egui Desktop GUI...${NC}"
        cargo build -p cell_dt_gui --release 2>&1
        DISPLAY="${DISPLAY:-:0}" ./target/release/cell_dt_gui
        ;;

    python)
        check_rust
        if ! command -v maturin &>/dev/null; then
            echo "Installing maturin..."
            pip install maturin
        fi
        echo -e "${BOLD}Building Python bindings...${NC}"
        cd crates/cell_dt_python
        maturin develop --release 2>&1
        echo -e "${GREEN}✓ Python bindings built${NC}"
        ;;

    *)
        echo -e "${RED}Unknown command: $CMD${NC}"
        echo "Run '$0' without arguments to see usage."
        exit 1
        ;;
esac
