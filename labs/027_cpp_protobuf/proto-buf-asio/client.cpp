
#include <array>
#include <boost/asio.hpp>
#include <iostream>

#include "detection.pb.h"

using boost::asio::ip::tcp;

int main() {
    boost::asio::io_context io;

    //-------
    // Create C++ Object
    //--------
    Detection detection;
    detection.set_id(9);
    detection.set_range(900.2);
    detection.set_velocity(90.5);

    //-----------
    // Serialize the object
    // ------------

    std::string buffer;
    bool success = detection.SerializeToString(&buffer);
    if (!success) {
        std::cerr << "Failed to serialize the message\n";
        return -1;
    }

    // --------
    // Connect to server
    // --------
    tcp::socket sock(io);

    sock.connect(tcp::endpoint(tcp::v4(), 50051));

    // ------
    // Send bytes over tcp
    // -------
    sock.write_some(boost::asio::buffer(buffer));
    std::cout << "Sent " << buffer.size() << " bytes\n";

    return 0;
}
