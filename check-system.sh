#!/bin/bash
# Verification script to check what's working and what's needed

echo "=========================================="
echo "DonutSMP Map Flipper - System Check"
echo "=========================================="
echo ""

# Check Node.js
echo "Checking Node.js..."
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    echo "✓ Node.js installed: $NODE_VERSION"
    
    # Check if version is >= 16
    MAJOR_VERSION=$(node --version | cut -d'v' -f2 | cut -d'.' -f1)
    if [ "$MAJOR_VERSION" -ge 16 ]; then
        echo "✓ Node.js version is sufficient (>= v16)"
    else
        echo "✗ Node.js version too old (need >= v16)"
        echo "  Install newer version: https://nodejs.org/"
    fi
else
    echo "✗ Node.js not installed"
    echo "  Install from: https://nodejs.org/"
fi
echo ""

# Check npm
echo "Checking npm..."
if command -v npm &> /dev/null; then
    NPM_VERSION=$(npm --version)
    echo "✓ npm installed: $NPM_VERSION"
else
    echo "✗ npm not installed"
fi
echo ""

# Check JavaScript dependencies
echo "Checking JavaScript dependencies..."
if [ -d "node_modules" ]; then
    if [ -d "node_modules/mineflayer" ]; then
        echo "✓ Mineflayer installed"
    else
        echo "✗ Mineflayer not installed"
        echo "  Run: npm install"
    fi
    
    if [ -d "node_modules/prismarine-auth" ]; then
        echo "✓ Prismarine-auth installed"
    else
        echo "✗ Prismarine-auth not installed"
        echo "  Run: npm install"
    fi
else
    echo "✗ No node_modules directory"
    echo "  Run: npm install"
fi
echo ""

# Check Rust
echo "Checking Rust..."
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo "✓ Rust installed: $RUST_VERSION"
    
    # Check if nightly
    if rustc --version | grep -q "nightly"; then
        echo "✓ Rust nightly toolchain active"
    else
        echo "⚠ Rust stable detected (nightly needed for Azalea)"
        echo "  Run: rustup default nightly"
    fi
else
    echo "✗ Rust not installed"
    echo "  Install from: https://rustup.rs/"
fi
echo ""

# Check Cargo
echo "Checking Cargo..."
if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version)
    echo "✓ Cargo installed: $CARGO_VERSION"
else
    echo "✗ Cargo not installed"
fi
echo ""

# Check required files
echo "Checking required files..."
FILES=("bot.js" "package.json" "config.json" "Cargo.toml" "src/main.rs")
for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "✓ $file exists"
    else
        echo "✗ $file missing"
    fi
done
echo ""

# Check config.json
echo "Checking config.json..."
if [ -f "config.json" ]; then
    if grep -q "your-email@example.com" config.json; then
        echo "⚠ config.json contains default email"
        echo "  Edit config.json with your Microsoft email"
    else
        echo "✓ config.json appears configured"
    fi
else
    echo "✗ config.json missing"
    echo "  Copy from: cp config.template.json config.json"
fi
echo ""

# Summary and recommendations
echo "=========================================="
echo "SUMMARY & RECOMMENDATIONS"
echo "=========================================="
echo ""

# Check if JavaScript version is ready
JS_READY=true
if ! command -v node &> /dev/null; then
    JS_READY=false
fi
if ! [ -d "node_modules/mineflayer" ]; then
    JS_READY=false
fi

# Check if Rust version is ready
RUST_READY=true
if ! command -v cargo &> /dev/null; then
    RUST_READY=false
fi

if [ "$JS_READY" = true ]; then
    echo "✓ JavaScript version: READY TO USE"
    echo "  Run: npm start"
    echo ""
else
    echo "⚠ JavaScript version: NOT READY"
    if ! command -v node &> /dev/null; then
        echo "  1. Install Node.js: https://nodejs.org/"
    fi
    if ! [ -d "node_modules/mineflayer" ]; then
        echo "  2. Install dependencies: npm install"
    fi
    echo "  3. Edit config.json with your email"
    echo "  4. Run: npm start"
    echo ""
fi

if [ "$RUST_READY" = true ]; then
    echo "⚠ Rust version: PARTIALLY COMPLETE"
    echo "  Core works but missing auction house interaction"
    echo "  See RUST-TODO.md for what's needed"
    echo ""
else
    echo "⚠ Rust version: NOT READY"
    if ! command -v cargo &> /dev/null; then
        echo "  1. Install Rust: https://rustup.rs/"
        echo "  2. Switch to nightly: rustup default nightly"
    fi
    echo "  3. Complete missing features (see RUST-TODO.md)"
    echo ""
fi

echo "=========================================="
echo "RECOMMENDATION:"
echo "=========================================="
if [ "$JS_READY" = true ]; then
    echo ""
    echo "  Use JavaScript version (fully functional)"
    echo "  Run: npm start"
    echo ""
else
    echo ""
    echo "  Install JavaScript dependencies first:"
    echo "  npm install"
    echo "  Then run: npm start"
    echo ""
fi

echo "For more help:"
echo "  - STATUS.md - Explains what works and what doesn't"
echo "  - QUICKSTART.md - Get JavaScript version running"
echo "  - RUST-TODO.md - Complete Rust version"
echo ""
