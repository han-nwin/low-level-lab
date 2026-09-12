#include "common.pb.h"
#include "customer.pb.h"
#include "order.pb.h"
#include "shipping.pb.h"

#include <fstream>
#include <iostream>

#include <boost/json.hpp>
#include <boost/log/trivial.hpp>

#include <google/protobuf/descriptor.h>

int main() {
    std::ifstream jsonFile("order.json");
    if (!jsonFile) {
        std::println("Failed to open file");
        return -1;
    }

    boost::json::value json = boost::json::parse(jsonFile);
    boost::json::object &jsonObj = json.as_object();

    std::println("{}", jsonObj.at("labels").at("channel").as_string());
    BOOST_LOG_TRIVIAL(info)
        << "Channel: " << jsonObj.at("labels").at("channel").as_string();

    // json walker
    std::function<void(boost::json::value json,
                       google::protobuf::Descriptor * descriptor)>
        walker;
    walker = [&](boost::json::value json,
                 google::protobuf::Descriptor *descriptor) {

    };
}
