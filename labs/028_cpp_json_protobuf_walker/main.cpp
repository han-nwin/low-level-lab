#include "common.pb.h"
#include "customer.pb.h"
#include "order.pb.h"
#include "shipping.pb.h"

#include <fstream>
#include <iostream>

#include <boost/json.hpp>
#include <boost/log/trivial.hpp>

#include <google/protobuf/descriptor.h>

using namespace std::string_view_literals;

void example() {

    std::ifstream jsonFile("order.json");
    if (!jsonFile) {
        std::println("Failed to open file");
        return;
    }

    // parse json to object
    // NOTE: json::value : internal boost json value
    boost::json::value json = boost::json::parse(jsonFile);

    // NOTE: if that json can be an object we can then do this
    if (!json.is_object()) {
        return;
    }
    boost::json::object &jsonObj = json.as_object();
    BOOST_LOG_TRIVIAL(info) << "Channel: " << jsonObj.at("labels").at("channel").as_string();

    // NOTE: turn a boost json value to string, call serialize()
    BOOST_LOG_TRIVIAL(info) << "JSON Value: " << boost::json::serialize(json);

    // Parse proto message
    // NOTE: to access top level message children -> call descriptor()
    const auto *descriptor = walker::practice::Order::descriptor();

    BOOST_LOG_TRIVIAL(info) << "PROTO: " << descriptor->field(0)->json_name();
    BOOST_LOG_TRIVIAL(info) << "PROTO: " << descriptor->field(0)->cpp_type();      // enum
    BOOST_LOG_TRIVIAL(info) << "PROTO: " << descriptor->field(0)->cpp_type_name(); // string name

    if (descriptor->field(0)->cpp_type() == google::protobuf::FieldDescriptor::CPPTYPE_MESSAGE) {

        // NOTE: to access other level message's chidren -> call message_type()
        const auto *next_descriptor = descriptor->field(0)->message_type();
        BOOST_LOG_TRIVIAL(info) << "PROTO: " << next_descriptor->field(0)->json_name();
        BOOST_LOG_TRIVIAL(info) << "PROTO: " << next_descriptor->field(0)->cpp_type(); // enum
        BOOST_LOG_TRIVIAL(info) << "PROTO: " << next_descriptor->field(0)->cpp_type_name();
    }
}

const std::string jsonT(const boost::json::value &json) {
    if (json.is_object()) {
        return "object";
    }
    if (json.is_array()) {
        return "array";
    }
    return "scalar";
}

const std::string protoT(const google::protobuf::FieldDescriptor *field) {
    if (field->is_map()) {
        return "object";
    }
    if (field->is_repeated()) {
        return "array";
    }
    // NOTE: this is put after is repeated ententionally for reapeated message type
    if (field->cpp_type() == google::protobuf::FieldDescriptor::CPPTYPE_MESSAGE) {
        return "object";
    }

    return "scalar";
}

int main() {

    // example();
    std::ifstream jsonFile("order.json");
    if (!jsonFile) {
        std::println("Failed to open file");
        return -1;
    }

    // parse json to internal boost json value
    boost::json::value json = boost::json::parse(jsonFile);

    // Parse proto message
    // to access top level message children -> call descriptor()
    const auto *descriptor = walker::practice::Order::descriptor();

    // json walker
    std::string msg{""};
    std::function<void(const boost::json::value json, const google::protobuf::Descriptor *descriptor,
                       std::string_view pos)>
        walker;

    walker = [&](const boost::json::value json, const google::protobuf::Descriptor *descriptor,
                 std::string_view pos) -> void {
        // Base case 1: reach the end,
        if (descriptor == nullptr) {
            std::println("Reach the end. Current pos: {}", pos);
            return;
        }

        // Error: this shouldn't happen. If json is a valid object
        if (!json.is_object()) {

            BOOST_LOG_TRIVIAL(error) << "JSON is not an object: " << pos;
            return;
        }

        // Loop through each field in proto and check then recursively walk
        // NOTE: field->message_type() describe what that field is
        // and return descriptor* if it's another message or
        // descriptor* with key/value if it's a map
        // otherwise return nullptr
        for (int i = 0; i < descriptor->field_count(); i++) {
            const std::string currentPos = std::string(pos) + "." + std::string(descriptor->field(i)->json_name());

            const auto &jsonObj = json.as_object();

            // Check if the json has that field
            if (!jsonObj.try_at(descriptor->field(i)->json_name())) {
                // this condition is to see if it actually has presence in the field
                // since optional type is allowed to be missing
                if (descriptor->field(i)->has_presence()) {
                    msg += "\nMISMATCH\n" + currentPos +
                           "\nJSON doesn't have: " + std::string(descriptor->field(i)->json_name()) + "\n";
                }

                continue;
            }

            // WALK into next level
            const auto *field = descriptor->field(i);
            const auto &jsonChild = jsonObj.at(field->json_name());

            // Predicate: Check what type is json and proto to see if they match
            const std::string jsonType = jsonT(jsonChild);
            const std::string protoType = protoT(field);

            // check mismatch type
            if (jsonType != protoType) {
                msg += "\nMISMATCH:\n" + currentPos + "\nJSON: " + jsonType + "\nPROTO: " + protoType + "\n";

                // WALKING CONDITION 1:
                // even there's a mismatch, we can continue
                continue;
            }

            // WALKING CONDITION 2: both is objects, but proto is a message not a map
            // This is easy case
            if (jsonType == "object" && !field->is_map()) {
                walker(jsonChild, field->message_type(), currentPos);
            }
            // WALKING CONDITION 3: both is objects, but proto is a map not a mesasge
            // if message_type() return a map when we ->field(i).json_name() it will return "key", "value" not the
            // actual name for us to do obj.at()
            else if (field->is_map()) {
                const auto *valueField = field->message_type()->FindFieldByName("value");

                if (!jsonChild.is_object()) {

                    BOOST_LOG_TRIVIAL(error) << "JSON is not an object: " << pos;
                }

                // no need validate keyField since it'll turn into string in json anyways
                // Validate map value field against every json value
                for (const auto &jsonEntry : jsonChild.as_object()) {

                    // walk one level down without recursion
                    // This is divergence from parent path, so need a separate position identifier
                    const std::string entryPos =
                        currentPos + "." + std::string(jsonEntry.key().data(), jsonEntry.key().size());

                    // Validate type first
                    const std::string jsonType = jsonT(jsonEntry.value());
                    const std::string protoType = protoT(valueField);
                    if (jsonType != protoType) {
                        msg += "\nMISMATCH:\n" + entryPos + "\nJSON: " + jsonType + "\nPROTO: " + protoType + "\n";
                        continue;
                    }

                    // valueField cannot be another map so no need a nother smaller recursion
                    //  This case only when valueFiled is a message, so ->message_type() should work
                    //  if valueField is another object, walk
                    if (jsonType == "object") {
                        walker(jsonEntry.value(), valueField->message_type(), currentPos);
                    }
                }
            }
            // WALKING CONDITION 4: Both is array
            else if (jsonType == "array") {
                // Every json element must be checked against the next field descriptor
                const auto &jsonArray = jsonChild.as_array();

                for (size_t j = 0; j < jsonArray.size(); j++) {
                    const auto &element = jsonArray.at(j);
                    const std::string elementPos = currentPos + "[" + std::to_string(j) + "]";

                    // Validate type first
                    const std::string jsonType = jsonT(element);
                    // protobuf doesn't allow nested array, so the next child can only be sclar or object
                    const std::string protoType = field->message_type() == nullptr ? "scalar" : "object";
                    if (jsonType != protoType) {
                        msg += "\nMISMATCH:\n" + elementPos + "\nJSON: " + jsonType + "\nPROTO: " + protoType + "\n";
                        continue;
                    }

                    walker(element, field->message_type(), elementPos);
                }
            }
        };
    };
    walker(json, descriptor, "$"sv);
    std::println("{}", msg);

    return 0;
}
