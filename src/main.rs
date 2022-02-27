use std::env;
use std::net::ToSocketAddrs;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::net::TcpStream;
use std::io::Read;
use std::io::Write;

fn dns_lookup(dname: &String) -> Vec<String> {
    let mut ip_addr_vec: Vec<String> = Vec::new();
    let port = ":80";
    let mut dname_with_port = dname.clone();
    dname_with_port.push_str(port);

    let ip_addr_iter = dname_with_port.to_socket_addrs().unwrap();
    for ip_addr in ip_addr_iter {
        ip_addr_vec.push(ip_addr.to_string());
        println!("DNS ip was {}", ip_addr.to_string());
    }

    return ip_addr_vec;
}

fn main() {
    for arg in env::args().skip(1) {
        let dname = arg;
        let ip_addr_vec = dns_lookup(&dname);

        let (client_tx, client_rx): (Sender<TcpStream>, Receiver<TcpStream>) =  mpsc::channel();
        let mut con_attempt_tx_vec: Vec<Sender<String>> = Vec::new();

        // Create connection attempt threads and channels used for communicating ip addresses from the main thread.
        for _i in 0..ip_addr_vec.len() {
            let (tx, rx): (Sender<String>, Receiver<String>) =  mpsc::channel();
            con_attempt_tx_vec.push(tx);

            let client_tx_clone = client_tx.clone();
            thread::spawn(move || {
                match rx.recv() {
                    Ok(ip_addr ) => {
                        let ip_addr_clone = ip_addr.clone();
                        if let Ok(stream) = TcpStream::connect(ip_addr) {
                            println!("Thread received ip {}", ip_addr_clone); // remove this later
                            let _ = client_tx_clone.send(stream);
                        }
                    },
                    Err(_error) => {
                        println!("Error occurred when 'connection attempt' thread tried to receive a message.");
                    }
                }        
            });
        }

        // Connected client thread
        let handle = thread::spawn(move|| {
            match client_rx.recv() {
                Ok(mut tcp_stream ) => {
                    println!("The peer address of the first connection received {}", 
                                tcp_stream.peer_addr().unwrap().ip().to_string());

                    let mut host_string = "Host: ".to_string();
                    host_string.push_str(&dname);

                    let mut request_text = String::new();
                    request_text.push_str("GET / HTTP/1.1");
                    request_text.push_str("\r\n");
                    request_text.push_str(&host_string);
                    request_text.push_str("\r\n");
                    // After request is complete, the server can close the connection.
                    request_text.push_str("Connection: close"); 
                    request_text.push_str("\r\n");
                    request_text.push_str("\r\n");

                    let _write_request = tcp_stream.write(request_text.as_bytes());
                    let mut read_buffer = String::new();
                    let _read_result = tcp_stream.read_to_string(&mut read_buffer);
                    println!("Data returned from the server = {}", read_buffer);

                    drop(tcp_stream);
                },
                Err(_error) => {
                    println!("Error occurred when 'client' thread tried to receive a message.");
                }
            }

            // Unbounded receiver waiting for the rest of the ip addresses that 
            // it will simply close without sending a request.
            while let Ok(tcp_stream) = client_rx.recv() {
                println!("thehee {}", tcp_stream.peer_addr().unwrap().ip().to_string());
                // Connection is closed when the TcpStream value is dropped. 
                drop(tcp_stream);
            }
        });

        // After all the connection attempt threads have been created, send each an ip addrss from the ip_addr
        for i in 0..ip_addr_vec.len() {
            let _ = con_attempt_tx_vec[i].send(ip_addr_vec[i].to_owned());
        }

        // Drop the send side of the channel, so that the `client_rx` stops waiting for messages.
        drop(client_tx);
        // Stop main thread from exiting until client thread is finished.
        handle.join().unwrap();
    }
}