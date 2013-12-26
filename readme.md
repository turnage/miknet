Miknet
================================================================================

Never fill a sockaddr_in struct by hand again! Miknet is a networking library
for people that like networking libraries.

    Notice: under construction.

Miknet can be installed on POSIX compliant Unix-like operating systems and
Windows. For Windows, compile in Cygwin or Msys.

Run these

    git clone https://github.com/PaytonTurnage/Miknet.git && cd Miknet
    mkdir build
    cd build
    cmake ..
    cmake --build .

This should get you libmiknet.a, which you can link against in your programs.

To install (reccomended for new users), run the following as root

    make install

For users unfamiliar with linking against libraries, perform the above command
to install, and append the flag

    -lmiknet

to your compiler invocation.