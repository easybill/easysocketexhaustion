# easysocketexhaustion

Socket Exhaustion occurs when a computer or server reaches the maximum number of network connections it can handle simultaneously. This results in the inability to establish new network connections.

With easysocketexhaustion, it is possible to benchmark socket exhaustion and check the network configuration.

Simple example:

```
easysocketexhaustion --ip_listen 127.0.0.1:1337 --ip_bench 127.0.0.1:1337 --wait_new_client_microseconds=10000
```

check the number of open connections
```
lsof -i -P | grep ":1337" | wc -l
```

## Downloads

You can download the latest binaries from the [releases page](https://github.com/easybill/easysocketexhaustion/releases) or use these permalinks for the latest version:
- [easysocketexhaustion_linux_latest_x86_64](https://github.com/easybill/easysocketexhaustion/releases/latest/download/easysocketexhaustion_ubuntu-latest_x86_64)
- [easysocketexhaustion_linux_latest_aarch64](https://github.com/easybill/easysocketexhaustion/releases/latest/download/easysocketexhaustion_ubuntu-latest_aarch64)
- [easysocketexhaustion_mac_latest_aarch64](https://github.com/easybill/easysocketexhaustion/releases/latest/download/easysocketexhaustion_mac_aarch64)
- [easysocketexhaustion_mac_x86_64](https://github.com/easybill/easysocketexhaustion/releases/latest/download/easysocketexhaustion_mac_x86_64)

## Parameters --ip_listen --ip_bench


`--ip_listen 127.0.0.1:1337` creates a new "server" or socket that listens.
`--ip_bench 127.0.0.1:1337` creates [--parallel] clients which all try to establish a new connection to the interface every [--wait_new_client_microseconds].

The parameters `ip_listen` and `ip_bench` can be repeated.

```
easysocketexhaustion
--ip_listen 127.0.0.1:443 # simulate a nginx
--ip_listen 127.0.0.1:3063 # simulate a mysql server
--ip_bench 127.0.0.1:443 # simulate an aggressive crawler
--ip_bench 127.0.0.1:3063 # simulate something that opens mysql connections
```

The output shows how many and what kind of errors there were. This way, you can test what the server can handle :).

## Parameters --wait_new_client_microseconds and --parallel

with the parameter `--wait_new_client_microseconds` you can make clients more aggressive, with `--parallel` you can define how many clients you want.
most time you want a small number of clients (e.g. 5) and that are more aggressive (low wait_new_client_microseconds).




