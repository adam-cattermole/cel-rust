#!/bin/bash

# Script to update CEL protobuf files from the official Google CEL specification

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROTO_DIR="$SCRIPT_DIR/proto/cel/expr"
CEL_SPEC_BASE_URL="https://raw.githubusercontent.com/google/cel-spec/master/proto/cel/expr"

# Create proto directory if it doesn't exist
mkdir -p "$PROTO_DIR"

# List of protobuf files to download
PROTO_FILES=(
    "syntax.proto"
    "checked.proto"
    "value.proto"
    "eval.proto"
    "explain.proto"
)

echo "Updating CEL protobuf files from official specification..."

for proto_file in "${PROTO_FILES[@]}"; do
    echo "Downloading $proto_file..."
    curl -s -o "$PROTO_DIR/$proto_file" "$CEL_SPEC_BASE_URL/$proto_file"

    if [ $? -eq 0 ]; then
        echo "✓ Successfully updated $proto_file"
    else
        echo "✗ Failed to update $proto_file"
        exit 1
    fi
done

echo ""
echo "All protobuf files updated successfully!"
echo "Run 'cargo build --features protobuf' to regenerate Rust code."
