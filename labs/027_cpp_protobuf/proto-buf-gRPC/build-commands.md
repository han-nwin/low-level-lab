```bash
protoc --cpp_out=. detection.proto

protoc \
    --cpp_out=. \
    --grpc_out=. \
    --plugin=protoc-gen-grpc=$(which grpc_cpp_plugin) \
    detection.proto
```
