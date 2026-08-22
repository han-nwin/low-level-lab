```bash
protoc --cpp_out=. detection.proto

protoc \
    --cpp_out=. \
    --grpc_out=. \
    --plugin=protoc-gen-grpc=$(which grpc_cpp_plugin) \
    detection.proto
```

```bash
# Compile
  clang++ -std=c++20 -I. \
    server.cpp detection.pb.cc detection.grpc.pb.cc \
    $(pkg-config --cflags --libs grpc++ protobuf) \
    -o server

  clang++ -std=c++20 -I. \
    client.cpp detection.pb.cc detection.grpc.pb.cc \
    $(pkg-config --cflags --libs grpc++ protobuf) \
    -o client
```
