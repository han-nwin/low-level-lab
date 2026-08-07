// A .cppm file is conventionally a module interface unit.
// The module declaration must appear before ordinary declarations.
export module learn.math;

// Only exported declarations are visible to files that import this module.
export namespace learn::math {

int add(int left, int right) { return left + right; }

int square(int value) { return value * value; }

} // namespace learn::math

// A declaration without export would remain an implementation detail of the
// module, even though it appears in the interface unit.
