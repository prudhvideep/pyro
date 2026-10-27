use std::{io::Write, net::TcpListener, time::Duration};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6969").unwrap();
    // set the listener to be non-blocking
    // an accecpt call returns immediately returning `WouldBlock` if the 
    // socket is not ready.
    listener.set_nonblocking(true).expect("cannot set non blocking true");

    loop {
        // this loops calls accept every 2 seconds
        // if accept is ready, processes the socket  
        // check the error code returned by accept
        // `WouldBlock` implies the socket is not ready.
        // if accept returns any other error panic!. 
        match listener.accept() {
            Ok((mut stream,socket)) => {
                println!("socket ready");
                let msg = format!("hello to {:?}\n", socket);
                stream.write_all(msg.as_bytes()).unwrap();        
            },
            Err(e) => match e.kind() {
               std::io::ErrorKind::WouldBlock => {
                   println!("socket not ready");
               }
               _ => {
                   panic!("unexpected error")
               }
            }           
        }
        
        std::thread::sleep(Duration::from_secs(2));
    }
}
