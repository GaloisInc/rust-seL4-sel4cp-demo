#![no_std]
#![no_main]
#![feature(never_type)]

extern crate alloc;


use sel4cp::{protection_domain, memory_region_symbol, Channel, Handler};
use sel4cp::debug_print;

use smoltcp::phy::{Device, TxToken, RxToken};
use smoltcp::wire::{IpEndpoint, IpAddress, IpCidr, EthernetAddress};
use smoltcp::storage::PacketMetadata;
use smoltcp::time::Instant;
use smoltcp::socket::{udp, tcp};

#[allow(unused_imports)]
use eth_driver_interface as interface;
use smoltcp::iface;

const DRIVER: Channel = Channel::new(2);
const ETH_TEST: Channel = Channel::new(3);

const PING: [u8; 4] = ['P' as u8, 'I' as u8, 'N' as u8, 'G' as u8];

#[protection_domain]
fn init() -> ThisHandler {
    let mut device = unsafe {
        interface::EthDevice::new(
            DRIVER,
            memory_region_symbol!(tx_free_region_start: *mut interface::RawRingBuffer),
            memory_region_symbol!(tx_used_region_start: *mut interface::RawRingBuffer),
            memory_region_symbol!(tx_buf_region_start: *mut [interface::Buf], n = interface::TX_BUF_SIZE),
            memory_region_symbol!(rx_free_region_start: *mut interface::RawRingBuffer),
            memory_region_symbol!(rx_used_region_start: *mut interface::RawRingBuffer),
            memory_region_symbol!(rx_buf_region_start: *mut [interface::Buf], n = interface::RX_BUF_SIZE),
        )
    };

    let netcfg = iface::Config::new(EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]).into());

    let mut netif = iface::Interface::new(netcfg, &mut device, Instant::from_millis(100));
    netif.update_ip_addrs(|ip_addrs| {
        ip_addrs
            .push(IpCidr::new(IpAddress::v4(127, 0, 0, 1), 8))
            .unwrap(); // TODO Handle this error
        });

    // `cnt` will simulate system clock
    let cnt = 0;
    ThisHandler{
        device,
        netif,
        cnt,
    }
}


struct ThisHandler{
    device: interface::EthDevice,
    netif: iface::Interface,
    cnt: u32,
}

impl Handler for ThisHandler {
    type Error = !;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        match channel {
            ETH_TEST => {
                self.cnt = self.cnt + 100;
                debug_print!("Got notification!\n");

                for i in 0..7 {
                    debug_print!("Attempt {i}\n");
                    //test_ethernet_loopback(self);
                    //test_udp_loopback(self);
                    test_tcp_loopback(self);
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

fn test_ethernet_loopback(h: &mut ThisHandler) {
    debug_print!("Testing ethernet loopback\n");

    match h.device.transmit(Instant::from_millis(h.cnt)) {
        None => debug_print!("[test_ethernet_loopback] Didn't get a TX token\n"),
        Some(tx) => {
            debug_print!(
                "[test_ethernet_loopback] Got a TX token\nSending some data: {}\n",
                core::str::from_utf8(&PING).unwrap(),
            );
            tx.consume(4, |buffer| buffer.copy_from_slice(PING.as_ref()))
        }
    }
    let mut x = 0;
    while x < 10 {
        x = x + 1;
        match h.device.receive(Instant::from_millis(h.cnt + x)) {
            None => {
                if x == 9 {
                    debug_print!("[test_ethernet_loopback] Nothing was received :-(\n");
                } else {
                    continue
                }
            },
            Some((rx, _tx)) => {
                rx.consume(|buffer|
                    debug_print!(
                        "[test_ethernet_loopback] Got an RX token of length {}: {}\n",
                        buffer.len(),
                        core::str::from_utf8(buffer).unwrap(),
                    )
                );
                break;
            }
        }
    }
}

fn test_udp_loopback(h: &mut ThisHandler) {
    debug_print!("Testing UDP loopback\n");

    let socket = {
        // These values don't seem to have an effect on the number of iterations we can perform
        static mut UDP_SERVER_RX_PACKET_BUFFERS: [u8; 1024] = [0; 1024];
        static mut UDP_SERVER_RX_PACKET_METADATA: [PacketMetadata<udp::UdpMetadata>; 2] = [PacketMetadata::EMPTY; 2];
        static mut UDP_SERVER_TX_PACKET_BUFFERS: [u8; 1024] = [0; 1024];
        static mut UDP_SERVER_TX_PACKET_METADATA: [PacketMetadata<udp::UdpMetadata>; 2] = [PacketMetadata::EMPTY; 2];
        let udp_rx_buffer = udp::PacketBuffer::new(
            unsafe { &mut UDP_SERVER_RX_PACKET_METADATA[..] },
            unsafe { &mut UDP_SERVER_RX_PACKET_BUFFERS[..] },
        );
        let udp_tx_buffer = udp::PacketBuffer::new(
            unsafe { &mut UDP_SERVER_TX_PACKET_METADATA[..] },
            unsafe { &mut UDP_SERVER_TX_PACKET_BUFFERS[..] },
        );
        udp::Socket::new(udp_rx_buffer, udp_tx_buffer)
    };
    let mut sockets: [_; 1] = Default::default();
    let mut socket_set = iface::SocketSet::new(&mut sockets[..]);
    let handle = socket_set.add(socket);

    let endpoint = IpEndpoint {
        addr: IpAddress::v4(127, 0, 0, 1),
        port: 9001,
    };

    {
        h.netif.poll(
            Instant::from_millis(h.cnt),
            &mut h.device,
            &mut socket_set,
        );
        let socket: &mut udp::Socket = socket_set.get_mut(handle);

        match socket.bind(endpoint) {
            Ok(()) => debug_print!("Bound UDP socket {endpoint}\n"),
            Err(e) => debug_print!("Failed to bind UDP socket {endpoint}: {e}\n"),
        }

        match socket.send_slice(PING[..].as_ref(), udp::UdpMetadata::from(endpoint)) {
            Ok(()) => debug_print!(
                "Sent a UDP packet to {endpoint}: {}\n",
                core::str::from_utf8(&PING[..]).unwrap(),
            ),
            Err(e) => debug_print!("Faied to send a UDP packet to {endpoint}: {e}\n"),
        }
    }

    // NOTE: a loop is a bad idea, some other timing is needed
    for x in 0..10 {
        h.netif.poll(
            // we need to provide increasing timestamp, but it doesn't seem to matter how much it increases between calls
            Instant::from_millis(h.cnt + x),
            &mut h.device,
            &mut socket_set,
        );
        let socket: &mut udp::Socket = socket_set.get_mut(handle);

        match socket.recv() {
            Ok((packet, source)) => {
                debug_print!("Got a UDP packet from {source}: {}\n", core::str::from_utf8(packet).unwrap());
                break;
            },
            Err(e) => {
                // print the last error we see before leaving the loop
                if x == 9 {
                    debug_print!("Error:{:?}\n",e);
                }
            }
        }
    }
}

fn test_tcp_loopback(h: &mut ThisHandler) {
    debug_print!("Testing TCP loopback\n");

    let server_socket = {
        static mut TCP_SERVER_RX_DATA: [u8; 1024] = [0; 1024];
        static mut TCP_SERVER_TX_DATA: [u8; 1024] = [0; 1024];
        let tcp_rx_buffer = tcp::SocketBuffer::new(unsafe { &mut TCP_SERVER_RX_DATA[..] });
        let tcp_tx_buffer = tcp::SocketBuffer::new(unsafe { &mut TCP_SERVER_TX_DATA[..] });
        tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer)
    };

    let client_socket = {
        static mut TCP_CLIENT_RX_DATA: [u8; 1024] = [0; 1024];
        static mut TCP_CLIENT_TX_DATA: [u8; 1024] = [0; 1024];
        let tcp_rx_buffer = tcp::SocketBuffer::new(unsafe { &mut TCP_CLIENT_RX_DATA[..] });
        let tcp_tx_buffer = tcp::SocketBuffer::new(unsafe { &mut TCP_CLIENT_TX_DATA[..] });
        tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer)
    };

    let mut sockets: [_; 2] = Default::default();
    let mut sockets = iface::SocketSet::new(&mut sockets[..]);
    let server_handle = sockets.add(server_socket);
    let client_handle = sockets.add(client_socket);

    let endpoint = IpEndpoint {
        addr: IpAddress::v4(127, 0, 0, 1),
        port: 9001,
    };

    let mut did_listen = false;
    let mut did_connect = false;
    let mut done = false;
    while !done {
        h.netif.poll(Instant::from_millis(100), &mut h.device, &mut sockets);

        let socket = sockets.get_mut::<tcp::Socket>(server_handle);
        if !socket.is_active() && !socket.is_listening() {
            if !did_listen {
                debug_print!("Listening on {endpoint}\n");
                socket.listen(endpoint).unwrap();
                did_listen = true;
            }
        }

        if socket.can_recv() {
            debug_print!(
                "Got a TCP packet: {:?}\n",
                socket.recv(|buffer| { (buffer.len(), core::str::from_utf8(buffer).unwrap()) })
            );
            socket.close();
            done = true;
        }

        let socket = sockets.get_mut::<tcp::Socket>(client_handle);
        let cx = h.netif.context();
        if !socket.is_open() {
            if !did_connect {
                debug_print!("Connecting to {endpoint}\n");
                socket
                    .connect(cx, endpoint, 65000)
                    .unwrap();
                did_connect = true;
            }
        }

        if socket.can_send() {
            match socket.send_slice(PING[..].as_ref()) {
                Ok(n) => debug_print!(
                    "Sent a TCP packet to {endpoint}: {}\n",
                    core::str::from_utf8(&PING[..n]).unwrap(),
                ),
                Err(e) => debug_print!("Faied to send a TCP packet to {endpoint}: {e}\n"),
            }
            socket.close();
        }
    }

    debug_print!("Done testing TCP loopback\n")
}
