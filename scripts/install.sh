#!/usr/bin/env bash
# ==============================================================================
# Prism Network Node: Universal 1-Line Installer (Linux & macOS)
# Usage: curl -sSL https://raw.githubusercontent.com/jsepkt/prism/main/scripts/install.sh | bash
# ==============================================================================

set -e

CYAN='\033[0;36m'
GREEN='\033[0;32m'
PURPLE='\033[0;35m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${PURPLE}"
cat << "EOF"
  _____  _____  _____  _____ __  __   _   _ ______ _______ 
 |  __ \|  __ \|_   _|/ ____|  \/  | | \ | |  ____|__   __|
 | |__) | |__) | | | | (___ | \  / | |  \| | |__     | |   
 |  ___/|  _  /  | |  \___ \| |\/| | | . ` |  __|    | |   
 | |    | | \ \ _| |_ ____) | |  | | | |\  | |____   | |   
 |_|    |_|  \_\_____|_____/|_|  |_| |_| \_|______|  |_|   
      Sovereign Context & Edge-AI Ledger (Layer-1)
EOF
echo -e "${NC}"

echo -e "${CYAN}==>${NC} Installing Prism Full Node & Validator Client..."

# 1. Check OS
OS="$(uname -s)"
case "${OS}" in
    Linux*)     PLATFORM=linux;;
    Darwin*)    PLATFORM=darwin;;
    *)          echo -e "${RED}Error: Unsupported operating system: ${OS}${NC}"; exit 1;;
esac

# 2. Check Prerequisites
if ! command -v git &> /dev/null; then
    echo -e "${RED}Error: git is required. Please install git first.${NC}"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Warning: Rust & Cargo not found. Installing Rust toolchain...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# 3. Clone or Update Repository
INSTALL_DIR="$HOME/.prism"
if [ -d "$INSTALL_DIR" ]; then
    echo -e "${CYAN}==>${NC} Updating existing Prism repository at ${INSTALL_DIR}..."
    cd "$INSTALL_DIR"
    git fetch origin main
    git reset --hard origin/main
else
    echo -e "${CYAN}==>${NC} Cloning Prism Network repository into ${INSTALL_DIR}..."
    git clone https://github.com/jsepkt/prism.git "$INSTALL_DIR"
    cd "$INSTALL_DIR"
fi

# 4. Compile with Release Optimizations
echo -e "${CYAN}==>${NC} Compiling prism-node with release optimizations..."
cargo build --release -p prism-node

# 5. Install Binary into PATH
BIN_SRC="${INSTALL_DIR}/target/release/prism-node"
BIN_DEST="/usr/local/bin/prism-node"

if [ -w "/usr/local/bin" ]; then
    cp "$BIN_SRC" "$BIN_DEST"
    chmod +x "$BIN_DEST"
else
    echo -e "${YELLOW}Notice: Sudo required to install prism-node to /usr/local/bin${NC}"
    sudo cp "$BIN_SRC" "$BIN_DEST"
    sudo chmod +x "$BIN_DEST"
fi

echo -e "\n${GREEN}✔ Prism Node successfully installed to ${BIN_DEST}!${NC}\n"
echo -e "To start your node:"
echo -e "  ${CYAN}prism-node${NC}"
echo -e ""
echo -e "Dashboard will be available at:"
echo -e "  ${PURPLE}http://127.0.0.1:8545${NC}"
echo -e ""
echo -e "To view public explorer & sovereign web wallet:"
echo -e "  ${CYAN}https://jsepkt.github.io/prism/${NC}"
echo -e ""
