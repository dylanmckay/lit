# lit

[![Crates.io](https://img.shields.io/crates/v/lit.svg)](https://crates.io/crates/lit)
[![Build Status](https://travis-ci.org/dylanmckay/lit.svg?branch=master)](https://travis-ci.org/dylanmckay/lit)
[![license](https://img.shields.io/github/license/dylanmckay/lit.svg)]()

An integrated testing tool, similar to LLVM's integrated testing tool (`llvm-lit`) but in library form.

[Rust API Documentation](https://docs.rs/lit)

## Usage

Point `lit` at a directory containing test files and it will execute the commands and run the checks
contained within the tests.

Any plain-text based file format that supports comments can be used as a test file, provided the comment
character has been added to lit's source code.

All testing is done based on text comparisons with the output of various command line tools executed
on each test file. The `CHECK` directives inside each test file validate that the command line tool
contains the expected text.

### Testing a bash script

Here is an example test file, it is a bash script. Assertions are added
that ensure the bash script outputs the correct text to stdout/stderr.
```bash
# RUN: sh -ea @file

# CHECK: hello world
echo hello world

# CHECK: number 1
# CHECK: number 2
# CHECK: number 3
# CHECK: number 4
for i in $(seq 1 4); do echo number $i; done
```

### Testing a C/C++ program

Here is an example C/C++ test file. Assertions are added
that ensure the C compiler name mangles the main method as a particular way.
```c
// RUN: gcc @file -S
//
// Note: '-S' compiles this C program to plain-text assembly.

// This next line verifies that there is an assembly label
// with a name mangled underscore prefix.
// CHECK: _main:
int main() {
  return 0;
}
```

### The `RUN` directive

This directive runs an executable, almost always operating on the test file as the source file.

```
RUN: <command-line>
```

Each `RUN` directive runs the same test file in different conditions.

### The `CHECK` directive

This directive is used to assert that the output of the `RUN` command
contains a specific string.

```
CHECK: <substring to look for>
```

If the substring is not found, then the test immediately fails.

## Variables

Variables can be used in directives by `@<variable name>`. The variable is substituted in-place with
the value of the variable at the time of the test.

## Default variables available to tests

These variables can be used by tests in directives.

| Name  (`*` = wildcard) | Description | Substituted value                          |
|-------------------------|--------------|---------------------------------------------|
| `@file`                 |              | The path the the test file being executed.  |
| `@*tempfile*`           | Any variable containing the text `tempfile` | A temporary file path. Subsequent uses of the same tempfile variable will give the same path. It is possible to use multiple tempfiles in one test by giving them separate names, like `@first_tempfile` and `@second_tempfile` |



