#!/usr/bin/env bash
set -euo pipefail

if command -v protoc >/dev/null 2>&1; then
  protoc --version
  exit 0
fi

if command -v apt-get >/dev/null 2>&1; then
  sudo apt-get update
  sudo apt-get install -y protobuf-compiler
  protoc --version
  exit 0
fi

echo "protoc is required for the CRM Helm scenario; install protobuf-compiler or set PROTOC" >&2
exit 1
