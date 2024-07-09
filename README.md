barebone basic tcp server using rust with smoltcp library.

# How to run
## preparation
```bash
sudo ip tuntap add name tap0 mode tap user $USER
sudo ip link set tap0 up
sudo ip addr add 192.168.69.100/24 dev tap0
```

## run the application
```bash
cargo run -- --tap tap0
```

## test the application
in other terminal, send packet to application host ip `192.168.69.11` tcp port `6969` by using `socat`, where the application will respond "hello" to any incoming connection and immediately close the connection.
```bash
socat stdio tcp4-connect:192.168.69.1:6969
```

