#!/bin/bash
# pack-to-fsimg.sh - Pack ch18_file* binaries into tg-ch18 fs.img
#
# This script:
# 1. Copies ch18_file* binaries to tg-ch18/linux-user/
# 2. Updates tg-ch18/tg-user/cases.toml to include them
# 3. Rebuilds tg-ch18 kernel (which packs them into fs.img)

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TG18_DIR="/home/chyyuu/thecodes/os-compare/tg-ch18"
LINUX_USER_DIR="$TG18_DIR/linux-user"
CASES_FILE="$TG18_DIR/tg-user/cases.toml"

echo -e "${BLUE}=== Packing ch18_file* into tg-ch18 filesystem ===${NC}"
echo

# Check if ch18_file* binaries exist
echo -e "${YELLOW}1. Checking for ch18_file* binaries...${NC}"
for prog in ch18_file0 ch18_file1 ch18_file2 ch18_file3; do
    if [ ! -f "$SCRIPT_DIR/$prog" ]; then
        echo -e "${RED}❌ Error: $SCRIPT_DIR/$prog not found${NC}"
        echo "   Please build first: CC=riscv64-linux-gnu-gcc make build-ch18-only"
        exit 1
    fi
    echo -e "${GREEN}✓ Found $prog${NC}"
done
echo

# Create linux-user directory if it doesn't exist
echo -e "${YELLOW}2. Setting up linux-user directory...${NC}"
mkdir -p "$LINUX_USER_DIR"

# Copy binaries to linux-user
echo -e "${YELLOW}3. Copying ch18_file* to linux-user...${NC}"
for prog in ch18_file0 ch18_file1 ch18_file2 ch18_file3; do
    cp "$SCRIPT_DIR/$prog" "$LINUX_USER_DIR/$prog"
    echo -e "${GREEN}✓ Copied $prog${NC}"
done
echo

# Backup cases.toml
echo -e "${YELLOW}4. Updating cases.toml...${NC}"
if [ ! -f "$CASES_FILE.bak" ]; then
    cp "$CASES_FILE" "$CASES_FILE.bak"
    echo -e "${GREEN}✓ Backed up $CASES_FILE to $CASES_FILE.bak${NC}"
else
    # Restore from backup before modification
    cp "$CASES_FILE.bak" "$CASES_FILE"
fi
echo

# Add ch18_file entries to cases.toml
# Find the [ch8] section and add our entries
echo -e "${YELLOW}5. Checking cases.toml for ch18_file* entries...${NC}"

# Check if already present
if grep -q "ch18_file0" "$CASES_FILE"; then
    echo -e "${GREEN}✓ ch18_file* entries already in cases.toml${NC}"
else
    # Use sed to add entries before "initproc"
    sed -i 's/    "initproc",/    "ch18_file0",\n    "ch18_file1",\n    "ch18_file2",\n    "ch18_file3",\n    "initproc",/' "$CASES_FILE"
    
    if grep -q "ch18_file0" "$CASES_FILE"; then
        echo -e "${GREEN}✓ Updated cases.toml with ch18_file* entries${NC}"
    else
        echo -e "${RED}❌ Failed to update cases.toml${NC}"
        exit 1
    fi
fi
echo

# Rebuild tg-ch18
echo -e "${YELLOW}6. Rebuilding tg-ch18 kernel...${NC}"
echo -e "${YELLOW}   This may take a few minutes...${NC}"
echo -e "${BLUE}   Using CHAPTER=test_ch18 to launch ch18_file0 directly${NC}"
cd "$TG18_DIR"

if CHAPTER=test_ch18 cargo build 2>&1; then
    echo -e "${GREEN}✓ tg-ch18 build successful${NC}"
    
    FS_IMG="$TG18_DIR/target/riscv64gc-unknown-none-elf/debug/fs.img"
    KERNEL="$TG18_DIR/target/riscv64gc-unknown-none-elf/debug/tg-ch18"
    
    echo
    echo -e "${GREEN}=== Success! ===${NC}"
    echo
    echo "Kernel image: $KERNEL"
    echo "Filesystem image: $FS_IMG"
    echo
    echo -e "${BLUE}Next: Run with qemu-system-riscv64${NC}"
    echo "cd $SCRIPT_DIR"
    echo "./run-in-qemu-system.sh"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi
