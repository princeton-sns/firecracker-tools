use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};
//use firerunner::vsock::{self, VsockCloser, VsockListener};
use std::fs::File;

use super::request;
use firerunner::pipe_pair::PipePair;

pub struct RequestManager {
    listener: File, // read end of the pipe through which VM signals it is ready to receive requests.
    channels: Arc<Mutex<BTreeMap<u32, (String, Receiver<request::Request>, PipePair)>>>,
    connections: BTreeMap<u32, JoinHandle<()>>,
    response_sender: Sender<(u32, String, Vec<u8>)>,
}

impl RequestManager {
    pub fn new(channels: Arc<Mutex<BTreeMap<u32, (String, Receiver<request::Request>, PipePair)>>>,
                response_sender: Sender<(u32, String, Vec<u8>)>,
                listener: File) -> RequestManager
    {
        RequestManager {
            listener: listener,
            channels,
            connections: BTreeMap::new(),
            response_sender,
        }
    }

    pub fn serve(&mut self) {
        println!("RequestManager Started");

        let mut id_buf = [0u8; 4]; // u32
        loop {
//            println!("waiting for connection");
            self.listener.read_exact(&mut id_buf).expect("Failed to read from listener pipe");
            let id = u32::from_le_bytes(id_buf);

//            println!("Connection from VM {}", &id);

            if let Some((function, request_receiver, connection)) =
                    self.channels.lock().expect("poisoned lock").remove(&id) {

                let response_sender = self.response_sender.clone();
                self.connections.insert(id, thread::spawn(move || {
                    let mut conn_mgr = ConnectionManager {
                        id,
                        function,
                        request_receiver,
                        response_sender,
                        connection,
                    };
                    conn_mgr.handle_connection();
                }));
            }
        }
    }

    pub fn spawn(mut self) -> JoinHandle<()> {
        thread::spawn(move || { self.serve() })
    }
}

struct ConnectionManager<T>{
    id:  u32,
    function: String,
    request_receiver: Receiver<request::Request>,
    response_sender: Sender<(u32, String, Vec<u8>)>,
    connection: T,
}

impl<T: Read + Write> ConnectionManager<T> {

    fn handle_request(connection: &mut T, request: request::Request) -> std::io::Result<Vec<u8>> {
        let mut request = serde_json::to_vec(&request).unwrap();
        request.push(0xa); // newline
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
                self.response_sender.send((self.id, self.function.clone(), response)).unwrap();
            } else {
                println!("Error response from VM");
                break;
            }
        }
        println!("Connection Manager exit");
    }
}
