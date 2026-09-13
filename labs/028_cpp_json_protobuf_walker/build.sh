#!/bin/sh
# e = stop if fail, u report error
set -eu

g++ $(cat compile_flags.txt) $(pkg-config --cflags protobuf) \
    main.cpp library/*.pb.cc \
    $(pkg-config --libs protobuf) -o main
