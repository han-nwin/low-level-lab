```bash
clang++ -std=c++20 \
    server.cpp \
    detection.pb.cc \
    $(pkg-config --cflags --libs protobuf) \
    -pthread \
    -o server

```


```bash
clang++ -std=c++20 \
    client.cpp \
    detection.pb.cc \
    $(pkg-config --cflags --libs protobuf) \
    -pthread \
    -o client

```
