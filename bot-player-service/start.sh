#!/bin/sh

protoc -I=./protos -I=/usr/include/google/protobuf --python_out=./protos_py ./protos/*.proto

PROTO_DIR="./protos_py"

update_imports() {
  local file="$1"
  echo "Processing $file"
  if sed --version >/dev/null 2>&1; then
    sed -i -E 's/^import /from protos_py import /' "$file"
  else
    sed -i '' -E 's/^import /from protos_py import /' "$file"
  fi
}


find "$PROTO_DIR" -type f -name "*.py" | while read -r file; do
  update_imports "$file"
done

echo "All import lines updated in .proto files within $PROTO_DIR"


python3 server.py



