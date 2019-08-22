use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};
use firerunner::vsock::{self, VsockCloser, VsockListener, VsockStream};

use super::request;

pub struct RequestManager {
    listener: VsockListener,
    channels: Arc<Mutex<BTreeMap<u32, (String, Receiver<request::Request>)>>>,
    connections: BTreeMap<u32, JoinHandle<()>>,
    response_sender: Sender<(u32, String, Vec<u8>)>,
}

impl RequestManager {
    pub fn new(channels: Arc<Mutex<BTreeMap<u32, (String, Receiver<request::Request>)>>>, response_sender: Sender<(u32, String, Vec<u8>)>) -> RequestManager {
        RequestManager {
            listener: VsockListener::bind(vsock::VMADDR_CID_ANY, 1234).expect("vsock listen"),
            channels,
            connections: BTreeMap::new(),
            response_sender,
        }
    }

    pub fn serve(&mut self) {
        while let Ok((connection, addr)) = self.listener.accept() {
            if let Some((function, request_receiver)) = self.channels.lock().expect("poisoned lock").remove(&addr.cid) {
                let response_sender = self.response_sender.clone();
                let cid = addr.cid;
                self.connections.insert(addr.cid, thread::spawn(move || {
                    let mut conn_mgr = ConnectionManager {
                        cid,
                        function,
                        request_receiver,
                        response_sender,
                        connection,
                    };
                    conn_mgr.handle_connection();
                }));
            }
        }
        println!("Done listening");
    }

    pub fn spawn(mut self) -> (JoinHandle<()>, VsockCloser) {
        let closer = self.listener.closer();
        (thread::spawn(move || { self.serve() }), closer)
    }
}

struct ConnectionManager {
    cid:  u32,
    function: String,
    request_receiver: Receiver<request::Request>,
    response_sender: Sender<(u32, String, Vec<u8>)>,
    connection: VsockStream,
}

impl ConnectionManager {

    fn handle_request(connection: &mut VsockStream, request: request::Request) -> std::io::Result<Vec<u8>> {
        let request = serde_json::to_vec(&request).unwrap();
        connection.write_all(&[request.len() as u8])?;
        connection.write_all(request.as_slice())?;
        let mut lens = [0];
        connection.read_exact(&mut lens)?;
        if lens[0] == 0 {
            return Ok(vec![]);
        }
        let mut response = Vec::with_capacity(lens[0] as usize);
        response.resize(lens[0] as usize, 0);
        connection.read_exact(response.as_mut_slice())?;
        Ok(response)
    }

    fn handle_connection(&mut self) {
        for request in self.request_receiver.iter() {
            if let Ok(response) = Self::handle_request(&mut self.connection, request) {
                self.response_sender.send((self.cid, self.function.clone(), response)).unwrap();
            } else {
                println!("Error response from VM");
                break;
            }
        }
        println!("Connection Manager exit");
    }
}
