# Contributing

The goal of the Miknet is to be simple, easy to use, and reliable. Users
should be able to pick it up, send and receive data, and not worry about
reliability, bandwidth, keepalives, etc. Use this to evaluate any design
decisions.

## Development Guidelines

When developing for Miknet, write tests _first_, not second. Write the
tests, then make them pass.

Every module in miknet should do _one_ thing. Each module should be
individually buildable and testable.

## Development Prereqs

Your machine will need

* Check, the C Unit testing library
* CMake
* C Compiler
* Git

This command will take care of everything on a machine with apt:

````apt-get install check cmake build-essential git pkg-config````

If you have never used git before, look up git configure and set up
your user.

## Development Process

If you are editing a package, make sure it is up to date in three places:

* The source file in src/
* The header file in src/include/miknet/
* The test file in tests/

If you are creating a package, make sure it is present in all of these places,
and add it to the build files in CMakeLists.txt and tests/CMakeLists.txt.

To test your code, run this in your build/ directory:

    cmake -DCMAKE_BUILD_TYPE=Debug ..
    make
    ctest

## Development Conclusion

Submit a pull request!
