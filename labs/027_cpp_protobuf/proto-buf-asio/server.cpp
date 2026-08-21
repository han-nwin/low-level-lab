#include <array>
#include <boost/asio.hpp>
#include <iostream>

#include "detection.pb.h"

using boost::asio::ip::tcp;

int main() {
    // create an io context
    boost::asio::io_context io;

    // Listener on port 50051
    tcp::acceptor acceptor(io, tcp::endpoint(tcp::v4(), 50051));

    // Create a stream socket
    tcp::socket sock(io);
    std::cout << "Waiting for connection...\n";

    // Accept the connection
    acceptor.accept(sock);

    // ---------------
    // Receive raw byte from TCP
    // - std::array, std::vector, std::string: own the bytes
    // - boost::asio::mutable_buffer: writable view of bytes
    // - boost::asio::const_buffer: read-only view of bytes
    // - boost::asio::buffer(storage): creates the appropriate view
    // ---------------
    std::array<char, 1024> storage;
    boost::asio::mutable_buffer buffer(storage.data(), storage.size());

    std::size_t n = sock.read_some(buffer);

    // ---------------
    // Deserialize the bytes
    // ---------------

    Detection detection;

    // parse the bytes into the struct
    bool success = detection.ParseFromArray(buffer.data(), n);
    if (!success) {
        std::cerr << "Failed to parse the message\n";
        return -1;
    }

    std::cout << "id: " << detection.id() << "\n";
    std::cout << "range: " << detection.range() << "\n";
    std::cout << "velocity: " << detection.velocity() << "\n";

    return 0;
}
