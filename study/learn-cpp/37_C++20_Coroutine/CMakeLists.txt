cmake_minimum_required(VERSION 3.28)
project(asio_demo LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

include(FetchContent)

set(BOOST_INCLUDE_LIBRARIES asio system)

FetchContent_Declare(
    Boost
    URL https://github.com/boostorg/boost/releases/download/boost-1.92.0/boost-1.92.0-cmake.tar.xz
    DOWNLOAD_EXTRACT_TIMESTAMP ON
)

FetchContent_MakeAvailable(Boost)

find_package(Threads REQUIRED)

add_executable(asio_demo main.cpp)

target_link_libraries(asio_demo PRIVATE
    Boost::asio
    Boost::system
    Threads::Threads
)
