# Protobuf JSON walking practice

Walk `order.json` against the root message `walker.practice.Order` in
`order.proto`. Resolve imported definitions from `customer.proto`,
`shipping.proto`, and `common.proto`. The JSON is syntactically valid but contains 20 deliberate
schema mismatches, including nested fields, array elements, map values,
an enum, and a `oneof` conflict.

The shipping branch goes through an array of shipments, a map of packages,
an array of tracking events, and a location containing an address and a map.
Package contents also contain an array of components. Some message types are
declared inside other messages, so practice resolving nested type names too.

Try reporting each mismatch with its JSON path, expected type or constraint,
and actual value. Continue walking after finding an error. Count the `oneof`
conflict as one mismatch and stop descending into a value with the wrong
container type. Check schema compatibility, not business rules such as totals.

Use [ProtoJSON rules](https://protobuf.dev/programming-guides/json/):
lowerCamelCase keys are valid, `int64` values serialize as decimal strings,
numeric strings are accepted for numeric fields, and omitted fields are
allowed here. Reject unknown fields for this exercise.

Compile the schema into a descriptor set (including imports) from this directory:

```sh
protoc -I . --include_imports --descriptor_set_out=/tmp/walker-practice.pb order.proto
```

<details>
<summary>Answer key — open after trying the exercise</summary>

| JSON path | Mismatch |
| --- | --- |
| `$.customer.verified` | Expected boolean; got string `"true"`. |
| `$.customer.shippingAddress.postalCode` | Expected string; got number `60601`. |
| `$.customer.shippingAddress.apartment` | Unknown field in `Address`. |
| `$.customer.tags[1]` | Expected string element; got number `42`. |
| `$.customer` | Both `email` and `phone` are set in the `contact` oneof. |
| `$.items[1].quantity` | Expected unsigned 32-bit integer; got `-1`. |
| `$.items[1].unitPrice.nanos` | Expected int32-compatible value; `"half"` is not numeric. |
| `$.items[1].options` | Expected array of strings; got a string. |
| `$.status` | `ORDER_STATUS_DELIVERED` is not a declared enum name. |
| `$.labels.priority` | Expected string map value; got number `3`. |
| `$.deliveryLocations.office` | Expected `Address` object; got a string. |
| `$.previousAddresses` | Expected array of `Address` objects; got an object. |
| `$.shipments[0].packages['box-a'].contents.components[1].quantity` | Expected uint32-compatible value; `"several"` is not numeric. |
| `$.shipments[0].packages['box-a'].contents.insuredValue.currencyCode` | Expected string; got an array. |
| `$.shipments[0].packages['box-a'].events[1].location.address.city` | Expected string; got boolean `false`. |
| `$.shipments[0].packages['box-a'].events[1].location.facilityDetails.dock` | Expected string map value; got number `8`. |
| `$.shipments[0].packages['box-a'].events[1].scans[1]` | Expected string element; got an object. |
| `$.shipments[0].packages['box-a'].events[1].operatorName` | Unknown field in `TrackingEvent`. |
| `$.shipments[0].packages['box-b'].contents.components` | Expected array of `Component` objects; got an object. |
| `$.shipments[1].packages` | Expected map represented as an object; got an array. |

</details>
