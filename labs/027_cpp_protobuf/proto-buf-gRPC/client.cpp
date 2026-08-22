#include <grpcpp/grpcpp.h>
#include <iostream>

#include <detection.grpc.pb.h>

int main() {

    // =====================================
    // Generate Detection Object
    // =====================================
    Detection detection;
    detection.set_id(1);
    detection.set_range(9.2);
    detection.set_velocity(991.32);

    // =====================================
    // Connect to server
    // =====================================
    std::shared_ptr<grpc::Channel> channel = grpc::CreateChannel(
        "localhost:50051", grpc::InsecureChannelCredentials());

    // Generate client object
    // Stub basically means a small stand-in/proxy for something else.
    // In gRPC, the client stub is a local object that stands in for the remote
    // server.
    auto stub = RadarService::NewStub(channel);

    // Send the object
    DetectionReply reply;
    grpc::ClientContext context;

    grpc::Status status = stub->SendDetection(&context, detection, &reply);

    if (!status.ok()) {
        std::cout << "Error: " << status.error_code() << ": "
                  << status.error_message() << std::endl;
        return -1;
    }

    std::cout << "Server accepted: " << reply.accepted() << std::endl;
}
