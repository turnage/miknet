# bench

This code has no SLA.

Simulations and benchmarking for network protocols. This depends on Linux tc to
configure the network interface. By default simulations run on a loopback
configured this way, but the client and server applications can be run anywhere.

Install deps with `./install_deps`.

Run benchmarks with `./run`, for example
`./run --start-server --rate 200 --transfers 1:800:60 enet` will measure ENet on
a 200kbit connection sending 800 bytes at 60 hertz.


