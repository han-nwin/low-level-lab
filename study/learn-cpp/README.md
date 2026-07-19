# C++ Learning Notes

Runnable examples and lecture notes migrated from the C/C++ Notion syllabus. Each
numbered directory is one topic and contains a standalone `main.cpp`. The lessons
currently run from `0.Introduction` through `28.RAII`.

## Run a lesson

```bash
clang++ -std=c++17 -Wall -Wextra -pedantic 9.Pointers/main.cpp -o /tmp/learn-cpp
/tmp/learn-cpp
```

The examples use C++17 unless a comment says otherwise. `27.OpenCV` is a
dependency-free introduction; once OpenCV is installed, use the compile command
shown in that lesson.
