use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::fd::{AsRawFd, RawFd};

#[derive(Debug)]
enum Connection {
    Listener(TcpListener),
    Stream(TcpStream),
}

#[inline]
fn new_kevent(ident: usize, filter: i16, flags: u16) -> libc::kevent {
    libc::kevent {
        ident,
        filter,
        flags,
        fflags: 0,
        data: 0,
        udata: std::ptr::null_mut(),
    }
}

#[inline]
fn empty_kevent() -> libc::kevent {
    new_kevent(0, 0, 0)
}

fn process_events(
    eventlist: &mut [libc::kevent],
    n_events: i32,
    registry: &mut HashMap<RawFd, Connection>,
    changelist: &mut Vec<libc::kevent>,
) {
    for i in 0..n_events {
        let event = eventlist[i as usize];
        match event.filter {
            libc::EVFILT_READ => {
                let conn = registry.get_mut(&(event.ident as i32));
                match conn {
                    Some(Connection::Listener(listener)) => if let Ok((stream,_)) = listener.accept() {
                            stream.set_nonblocking(true).unwrap();
                            let event = new_kevent(
                                stream.as_raw_fd() as usize,
                                libc::EVFILT_READ,
                                libc::EV_ADD | libc::EV_ENABLE,
                            );

                            changelist.push(event);
                            registry.insert(stream.as_raw_fd(), Connection::Stream(stream));
                    },
                    Some(Connection::Stream(stream)) => {
                        let mut buf = [0u8; 1024];
                        let n_bytes = stream.read(&mut buf);
                        match n_bytes {
                            Ok(0) => {
                                registry.remove(&(event.ident as i32));
                            }
                            Ok(_) => {
                                stream.write("Response : ".as_bytes()).unwrap();
                                stream.write(buf.as_slice()).unwrap();
                            }
                            _ => {}
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }
}

fn main() {
    let mut registry = HashMap::<RawFd, Connection>::new();

    let listener = TcpListener::bind("127.0.0.1:6969").unwrap();
    listener
        .set_nonblocking(true)
        .expect("cannot set non blocking true");

    let listener_event = new_kevent(
        listener.as_raw_fd() as usize,
        libc::EVFILT_READ,
        libc::EV_ADD | libc::EV_ENABLE,
    );

    registry.insert(listener.as_raw_fd(), Connection::Listener(listener));
    let kq_fd: libc::c_int = unsafe { libc::kqueue() };

    let mut changelist = Vec::<libc::kevent>::new();
    changelist.push(listener_event);

    const NUM_EVENTS: libc::c_int = 64;
    let mut eventlist = [empty_kevent(); NUM_EVENTS as usize];

    unsafe {
        loop {
            let n_events = libc::kevent(
                kq_fd,
                changelist.as_ptr(),
                changelist.len() as i32,
                eventlist.as_mut_ptr(),
                NUM_EVENTS,
                std::ptr::null(),
            );

            changelist.clear();

            if n_events > 0 {
                process_events(&mut eventlist, n_events, &mut registry, &mut changelist);
            }
        }
    }
}
