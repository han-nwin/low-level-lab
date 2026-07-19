# Set the minimum version of CMake that can be used
# To find the cmake version run
# $ cmake --version
cmake_minimum_required(VERSION 3.30.5)

# Set project name
project(log_analyer)

# set cpp version to 20
set(CMAKE_CXX_STANDARD 20)
# set to export compile commands
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# add an executable
add_executable(main src/main.cpp)
