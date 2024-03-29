use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};
//use firerunner::vsock::{self, VsockCloser, VsockListener};
use std::fs::File;

use super::request;
use firerunner::pipe_pair::PipePair;
use super::metrics::Metrics;
use time::precise_time_ns;

pub struct RequestManager {
    listener: File, // read end of the pipe through which VM signals it is ready to receive requests.
    channels: Arc<Mutex<BTreeMap<u32, (String, u32, Receiver<request::Request>, PipePair)>>>,
    stat: Arc<Mutex<Metrics>>,
    connections: BTreeMap<u32, JoinHandle<()>>,
    response_sender: Sender<(u32, u32, String, Vec<u8>)>,
}

impl RequestManager {
    pub fn new(channels: Arc<Mutex<BTreeMap<u32, (String, u32, Receiver<request::Request>, PipePair)>>>,
                stat: Arc<Mutex<Metrics>>,
                response_sender: Sender<(u32, u32, String, Vec<u8>)>,
                listener: File) -> RequestManager
    {
        RequestManager {
            listener: listener,
            channels,
            stat,
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
            self.stat.lock().unwrap().log_boot_timestamp(id, time::precise_time_ns());

//            println!("Connection from VM {}", &id);

            if let Some((function, user_id, request_receiver, connection)) =
                    self.channels.lock().expect("poisoned lock").remove(&id) {

                let response_sender = self.response_sender.clone();
                let stat = self.stat.clone();
                self.connections.insert(id, thread::spawn(move || {
                    let mut conn_mgr = ConnectionManager {
                        id,
                        user_id,
                        function,
                        request_receiver,
                        response_sender,
                        connection,
                        stat,
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
    user_id: u32,
    function: String,
    request_receiver: Receiver<request::Request>,
    response_sender: Sender<(u32, u32, String, Vec<u8>)>,
    connection: T,
    stat: Arc<Mutex<Metrics>>,
}

impl<T: Read + Write> ConnectionManager<T> {

    fn handle_request(connection: &mut T, request: request::Request) -> std::io::Result<Vec<u8>> {

        let mut request = serde_json::to_vec(&request).unwrap();
        request.push(0xa); // newline
        connection.write_all(request.as_slice())?;
        let mut lens = [0; 4];
        connection.read_exact(&mut lens)?;
        let len = u32::from_be_bytes(lens);
        if len == 0 {
            return Ok(vec![]);
        }
        let mut response = Vec::with_capacity(len as usize);
        response.resize(len as usize, 0);
        connection.read_exact(response.as_mut_slice())?;
        Ok(response)
    }

    fn handle_connection(&mut self) {
        for request in self.request_receiver.iter() {
            self.stat.lock().unwrap().log_request_timestamp(self.id,precise_time_ns());
            if let Ok(response) = Self::handle_request(&mut self.connection, request) {
                self.response_sender.send((self.id, self.user_id, self.function.clone(), response)).unwrap();
                self.stat.lock().unwrap().log_request_timestamp(self.id,precise_time_ns());
            } else {
                println!("Error response from VM");
                break;
            }
        }
//        println!("Connection Manager exit");
    }
}
