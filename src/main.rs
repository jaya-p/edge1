
// to run: 
/*
// preparation
sudo ip tuntap add name tap0 mode tap user $USER
sudo ip link set tap0 up
sudo ip addr add 192.168.69.100/24 dev tap0
//sudo ip -6 addr add fe80::100/64 dev tap0
//sudo ip -6 addr add fdaa::100/64 dev tap0
//sudo ip -6 route add fe80::/64 dev tap0
//sudo ip -6 route add fdaa::/64 dev tap0

// run the application
cargo run -- --tap tap0

// in other terminal, send packet to tcp port 6969
//   TCP connections on port 6969 (socat stdio tcp4-connect:192.168.69.1:6969), 
//   where it will respond "hello" to any incoming connection and immediately close it;
*/


use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::phy::{wait as phy_wait, Device, Medium, TunTapInterface};
use smoltcp::socket::{tcp};
use smoltcp::time::{Instant};
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address};

use getopts::{Options};

use std::env;
use std::process;
use std::os::unix::io::AsRawFd;
use std::fmt::Write;


pub fn create_options() -> (Options, Vec<&'static str>) {
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    (opts, Vec::new())
}

fn main() {
    //
    let (mut opts, free) = create_options();

    // add tuntap options
    opts.optopt("", "tun", "TUN interface to use", "tun0");
    opts.optopt("", "tap", "TAP interface to use", "tap0");
    // add middleware options: not implemented

    // parse options
    let matches = match opts.parse(env::args().skip(1)) {
        Err(err) => {
            println!("{err}");
            process::exit(1)
        }
        Ok(matches) => {
            if matches.opt_present("h") || matches.free.len() != free.len() {
                let brief = format!(
                    "Usage: {} [OPTION]... {}",
                    env::args().next().unwrap(),
                    free.join(" ")
                );
                print!("{}", opts.usage(&brief));
                process::exit((matches.free.len() != free.len()) as _);
            }
            matches
        }
    };

    // parse tuntap options
    let tun = matches.opt_str("tun");
    let tap = matches.opt_str("tap");
    let mut device = match (tun, tap) {
        (Some(tun), None) => TunTapInterface::new(&tun, Medium::Ip).unwrap(),
        (None, Some(tap)) => TunTapInterface::new(&tap, Medium::Ethernet).unwrap(),
        _ => panic!("You must specify exactly one of --tun or --tap"),
    };

    let fd = device.as_raw_fd();

    // parse middleware options: not implemented

    // Create interface
    let mut config = match device.capabilities().medium {
        Medium::Ethernet => {
            Config::new(EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]).into())
        }
        Medium::Ip => Config::new(smoltcp::wire::HardwareAddress::Ip),
        Medium::Ieee802154 => todo!(),
    };

    config.random_seed = rand::random();

    let mut iface = Interface::new(config, &mut device, Instant::now());
    iface.update_ip_addrs(|ip_addrs| {
        ip_addrs
            .push(IpCidr::new(IpAddress::v4(192, 168, 69, 1), 24))
            .unwrap();
    });
    iface
        .routes_mut()
        .add_default_ipv4_route(Ipv4Address::new(192, 168, 69, 100))
        .unwrap();

    // Create socket
    let tcp1_rx_buffer = tcp::SocketBuffer::new(vec![0; 64]);
    let tcp1_tx_buffer = tcp::SocketBuffer::new(vec![0; 128]);
    let tcp1_socket = tcp::Socket::new(tcp1_rx_buffer, tcp1_tx_buffer);

    let mut sockets = SocketSet::new(vec![]);
    let tcp1_handle = sockets.add(tcp1_socket);

    loop {
        let timestamp = Instant::now();
        iface.poll(timestamp, &mut device, &mut sockets);

        // tcp:6969: respond "hello"
        let socket = sockets.get_mut::<tcp::Socket>(tcp1_handle);
        if !socket.is_open() {
            socket.listen(6969).unwrap();
        }

        if socket.can_send() {
            writeln!(socket, "hello").unwrap();
            socket.close();
        }

        phy_wait(fd, iface.poll_delay(timestamp, &sockets)).expect("wait error");
    }

}
