extern crate craft_server;

//use std::net::TcpListener;
//use std::sync::{mpsc, Arc, Mutex};
//use std::thread;
//use craft_server::*;
use craft_server::Server;

fn main() {
    Server::run();

    /*
    let listener = TcpListener::bind("127.0.0.1:4080").unwrap();

    let clients = Arc::new(Mutex::new(Vec::new()));
    let mut current_id: client::Id = 1;

    let (tx, rx) = mpsc::channel();

    server(rx, clients.clone());

    for i in listener.incoming() {
        let stream = i.unwrap();

        if let Ok(c) = client::Client::run(stream, tx.clone(), current_id) {
            clients.lock().unwrap().push(c);
        }

        //for x in clients {
        //    println!("client {}: {}", x.id(), x.alive());
        //}

        current_id += 1;
    }
    */
}

/*
fn server(rx: mpsc::Receiver<event::Event>, clients: Arc<Mutex<Vec<client::Client>>>) {
    thread::spawn(move || {
        loop {
            if let Ok(ev) = rx.recv() {
                println!("{:?}", ev);

                for c in clients.lock().unwrap().iter() {
                    println!("{:?}", c.id());
                }
            }
        }
    });
}
*/

/*
fn client_thread(mut stream: TcpStream, id: client::Id) {
    /*
    let mut received = Vec::with_capacity(1024);
    stream.read(&mut received).unwrap();

    let received = String::from_utf8(received).unwrap();

    if received.lines().next().unwrap_or("") == "V,1" {
        println!("Client {:?} has joined.", stream.peer_addr().unwrap());
    } else {
        println!("Client {:?} denied. Wrong first version message, expected `V,1`, got: {}",
                 stream.peer_addr().unwrap(),
                 received.lines().next().unwrap());
    }
    */

    let mut version_buf: [u8; 4] = [0; 4];
    stream.read_exact(&mut version_buf).unwrap();

    let addr_str = match stream.peer_addr() {
        Ok(addr) => addr.to_string(),
        Err(_) => "<unknown addr>".to_string(),
    };

    if version_buf == [b'V', b',', b'1', b'\n'] {
        println!("{:?} joined.", addr_str);

        Client::run(stream, id);
    } else {
        println!("{:?} denied.", addr_str);

        return;
    }

    println!("A client left.");
}
*/
