#include <grpcpp/grpcpp.h>
#include <iostream>

#include <detection.grpc.pb.h>

// FInal inheritance of RadarService::Service
class RadarServiceImpl final : public RadarService::Service {

    // context: info about the grpc
    // detection is the object we received from the client
    // reply is the object we send back to the client
    grpc::Status SendDetection(grpc::ServerContext *context,
                               const Detection *detection,
                               DetectionReply *reply) override {
        // Look at this!
        //
        // We ALREADY HAVE a Detection object.
        //
        // NO:
        //
        //     recv()
        //     ParseFromArray()
        //     ParseFromString()
        //
        // gRPC already received + deserialized it.
        std::cout << "id: " << detection->id() << std::endl;
        std::cout << "range: " << detection->range() << std::endl;
        std::cout << "velocity: " << detection->velocity() << std::endl;

        // Send metadata back to client
        context->AddInitialMetadata("my-key", "my-value");
        // Actually send the reply
        reply->set_accepted(true);

        return grpc::Status::OK;
    }
};

int main() {
    RadarServiceImpl service;

    grpc::ServerBuilder builder;

    // =====================================
    // Create the server and register the service
    // =====================================
    builder.AddListeningPort("localhost:50051",
                             grpc::InsecureServerCredentials());
    builder.RegisterService(&service);

    std::unique_ptr<grpc::Server> server = builder.BuildAndStart();

    std::cout << "Server listening on port 50051..." << std::endl;

    server->Wait();
}
