#pragma once

#include <boost/json.hpp>
#include <boost/log/trivial.hpp>
#include <google/protobuf/descriptor.h>

#include <string>
#include <string_view>

namespace jsonProtobufWalker {

/**
 * Walks a Boost.JSON value against a Protobuf message descriptor.
 *
 * @param json JSON value to validate.
 * @param descriptor Protobuf message descriptor to validate against.
 * @param pos JSON path used in diagnostics. Defaults to `root`.
 * @return Mismatch messages found while walking the value.
 */
inline std::string walker(const boost::json::value &json, const google::protobuf::Descriptor *descriptor,
                          std::string_view pos = "root") {
    const auto jsonT = [](const boost::json::value &value) {
        if (value.is_object()) {
            return std::string{"object"};
        }
        if (value.is_array()) {
            return std::string{"array"};
        }
        return std::string{"scalar"};
    };

    const auto protoT = [](const google::protobuf::FieldDescriptor *field) {
        if (field->is_map()) {
            return std::string{"object"};
        }
        if (field->is_repeated()) {
            return std::string{"array"};
        }
        // NOTE: this is put after is_repeated ententionally for reapeated message type
        if (field->cpp_type() == google::protobuf::FieldDescriptor::CPPTYPE_MESSAGE) {
            return std::string{"object"};
        }

        return std::string{"scalar"};
    };

    std::string msg{""};

    // Base case 1: reach the end,
    if (descriptor == nullptr) {
        BOOST_LOG_TRIVIAL(info) << "Reach the end. Current position: " << pos;
        return msg;
    }

    // Error: this shouldn't happen. If json is a valid object
    if (!json.is_object()) {
        BOOST_LOG_TRIVIAL(error) << "JSON is not an object: " << pos;
        return msg;
    }

    // Loop through each field in proto and check then recursively walk
    // NOTE: field->message_type() describe what that field is
    // and return descriptor* if it's another message or
    // descriptor* with key/value if it's a map
    // otherwise return nullptr
    const auto &jsonObj = json.as_object();
    for (int i = 0; i < descriptor->field_count(); i++) {
        const std::string currentPos = std::string(pos) + "." + std::string(descriptor->field(i)->json_name());

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
            msg += walker(jsonChild, field->message_type(), currentPos);
        }
        // WALKING CONDITION 3: both is objects, but proto is a map not a mesasge
        // if message_type() return a map when we ->field(i).json_name() it will return "key", "value" not the
        // actual name for us to do obj.at()
        else if (field->is_map()) {
            const auto *valueField = field->message_type()->FindFieldByName("value");

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
                // This case only when valueFiled is a message, so ->message_type() should work
                // if valueField is another object, walk
                if (jsonType == "object") {
                    msg += walker(jsonEntry.value(), valueField->message_type(), entryPos);
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

                msg += walker(element, field->message_type(), elementPos);
            }
        }
    }

    return msg;
}

} // namespace jsonProtobufWalker
