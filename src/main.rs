use std::env;
use std::net::ToSocketAddrs;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

fn dns_lookup(dname: &String) -> Vec<String> {
    let mut ip_addr: Vec<String> = Vec::new();
    let port = ":443";
    let mut dname_with_port = dname.clone();
    dname_with_port.push_str(port);
    let addr_iter = dname_with_port.to_socket_addrs().unwrap();
    for addr in addr_iter {
        if addr.is_ipv4() {
            let ipv4: String = Vec::from_iter(addr.to_string().split(":").map(String::from))[0].clone();
            ip_addr.push(ipv4);
        } else {
            let ipv6: String = Vec::from_iter(addr.to_string().split("]").map(String::from))[0].clone();
            ip_addr.push(ipv6[1..].to_string());
        }
    
    }

    return ip_addr;
}

fn main() {
   // 
    for arg in env::args().skip(1) {
        let ip_addr = dns_lookup(&arg);

        let (client_tx, client_rx): (Sender<String>, Receiver<String>) =  mpsc::channel();

        let mut conn_attempt_channels: Vec<Sender<String>> = Vec::new();
        let mut threads: Vec<_> = Vec::new();

        // Create connection attempt threads and channels used for communicating ip addresses from main thread.
        for _i in 0..ip_addr.len() {
            let (tx, rx): (Sender<String>, Receiver<String>) =  mpsc::channel();
            conn_attempt_channels.push(tx);
            let client_tx_clone = client_tx.clone();
            threads.push(thread::spawn(move || {
                rx.recv();
                client_tx_clone.send("tolo".to_string());
                // todo: add other things
            }))
        }

        // After all the connection attempt threads have been created, send each an ip addrss from the ip_addr
        for i in 0..conn_attempt_channels.len() {
            let _ = conn_attempt_channels[i].send(ip_addr[i].to_owned());
        }

        // Connected client thread
        thread::spawn(move|| {
            match client_rx.recv() {
                Ok(value ) => {
                    
                },
                Err(error) =>
                {}
            }
        });
    }
}