# easysocketexhaustion

Socket Exhaustion occurs when a computer or server reaches the maximum number of network connections it can manage simultaneously. This exhaustion results in the inability to establish new network connections.

With easysocketexhaustion, it is possible to benchmark socket exhaustion and check the network configuration.

Simple example:

```
easysocketexhaustion --ip_listen 127.0.0.1:1337 --ip_bench 127.0.0.1:1337 --wait_new_client_microseconds=10000
```

check the number of open connections
```
lsof -i -P | grep ":1337" | wc -l
```

# Parameters in Detail


`--ip_listen 127.0.0.1:1337` creates a new "server" or socket that listens.
`--ip_bench 127.0.0.1:1337` creates [--parallel] clients which all try to establish a new connection to the interface every [--microseconds].

The parameters can be repeated as desired. It is also possible to bind to multiple interfaces by specifying several --ip_listen arguments.

The output shows how many and what kind of errors there were. This way, you can test what the server can handle :).






