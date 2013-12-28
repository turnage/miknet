Miknet
================================================================================

Never fill a sockaddr_in struct by hand again! Miknet is a TCP networking
library. It's fairly simple to use; no networking knowledge is required. It is
also IPV6 friendly!

For up-to-date information whether you should use, and how you can use Miknet,
visit [the wiki](https://github.com/PaytonTurnage/Miknet/wiki).

To install, clone the repo and

    mkdir build && cd build
    cmake ..
    make
    make install

```make install``` will need to be run as root.

Once it is installed, you can write programs with it by appending ```-lmiknet``` to
your compiler invocation, and include ```<miknet/miknet.h>``` in your source code.

Note: There are helpful debugging messages available. If you want to see these
while programming with miknet, replace ```cmake ..``` above with this command:
```cmake .. -DMIK_DEBUG=on```. You will probably not want this version when you
release your program, however.
