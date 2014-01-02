# Todo

1. ~~Make sure mikpack_t is the same size (or has appropriate padding) on all
   machines (pointer width).~~
2. ~~Send everything out in NBR; assume everything coming in is in NBR.~~
3. ~~Allow peers to be marked for a "bare" connection, on which no checks are made
   and no packets are sent; this way miknet can be used with other programs
   which do not use miknet.~~
4. Make the test files a non-default target.
5. Implement a memory backup using try_alloc() to reduce the amount of calls to
   calloc made when sending and receiving packets.
6. Allow programmers to cap outgoing bandwidth on a node.
