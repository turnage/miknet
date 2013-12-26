Miknet
================================================================================

Never fill a sockaddr_in struct by hand again! Miknet is a networking library
for people that like networking libraries.

    Notice: under construction.

Build instructions:

    mkdir build
    cd build
    cmake ..
    cmake --build .

This should get you libmiknet.a, which you can link against in your programs. To
install it for linking anywhere, run the following command as root

    mv libmiknet.a /usr/local/lib/libmiknet.a

Features

* TCP ~~and/or UDP~~
* IPv4 and/or IPv6
* Bandwidth caps
* Thread safety