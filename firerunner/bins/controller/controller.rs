use std::collections::btree_map::{BTreeMap, Entry};
use std::default::Default;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::fs::File;
use std::os::unix::io::FromRawFd;

use super::config;
use super::listener;
use super::request;

use firerunner::runner::{VmApp, VmAppConfig};
//use firerunner::vsock::VsockCloser;
use firerunner::pipe_pair::PipePair;

pub struct Inner {
    active_functions: Mutex<BTreeMap<String, (Sender<request::Request>, usize, VmApp)>>,
    warm_functions: Mutex<BTreeMap<String, (Sender<request::Request>, VmApp)>>,
    channels: Arc<Mutex<BTreeMap<u32, (String, Receiver<request::Request>, PipePair)>>>,
    max_channel: AtomicUsize,

    function_configs: config::Configuration,
    seccomp_level: u32,
    cmd_line: String,
    kernel: String,

    notifier: File,
}

pub struct Controller {
    inner: Arc<Inner>,
    //vsock_closer: Option<VsockCloser>,
    listener: File,  // this is cloned and used by RequestManger
}

impl Controller {
    pub fn new(function_configs: config::Configuration, seccomp_level: u32, cmd_line: String, kernel: String) -> Controller {
        let (listener, notifier) = nix::unistd::pipe().expect("Failed to create a pipe");
        Controller {
            inner: Arc::new(Inner {
                active_functions: Default::default(),
                warm_functions: Default::default(),
                channels: Default::default(),
                max_channel: AtomicUsize::new(3),
                seccomp_level,
                cmd_line,
                kernel,
                function_configs,
                notifier: unsafe{ File::from_raw_fd(notifier) },
            }),
            //vsock_closer: None,
            listener: unsafe{ File::from_raw_fd(listener) },
        }
    }

    pub fn schedule(&mut self, req: request::Request) {
        self.inner.schedule(req);
    }

    pub fn ignite(&mut self) -> Handle {
        let (response_sender, response_receiver) = channel();

        //let (manager_handle, vsock_closer) = listener::RequestManager::new(self.inner.channels.clone(), response_sender).spawn();
        //self.vsock_closer = Some(vsock_closer);
        let listener = self.listener.try_clone().expect("Failed to clone pipe listener");
        let manager_handle = listener::RequestManager::new(self.inner.channels.clone(), response_sender, listener).spawn();

        let inner = self.inner.clone();
        let response_handle = thread::spawn(move || {
            for response in response_receiver.iter() {
                inner.process_response(response);
            }
        });

        Handle (vec![manager_handle, response_handle])
    }

    pub fn kill_all(&mut self) {
        while self.inner.active_functions.lock().unwrap().len() > 0 {
            thread::yield_now();
        }
        //self.vsock_closer.take().map(|mut c| c.close()).unwrap().unwrap();
        self.inner.warm_functions.lock().unwrap().clear();
    }
}

impl Inner {
    pub fn schedule(&self, req: request::Request) {
        match self.active_functions.lock().unwrap().entry(req.function.clone()) {
            Entry::Occupied(mut entry) => {
                let (sender, outstanding, _app) = entry.get_mut();
                *outstanding += 1;
                sender.send(req).expect("sending request");
            },
            Entry::Vacant(entry) => {
                if let Some((sender, app)) = self.warm_functions.lock().unwrap().remove(entry.key()) {
                    sender.send(req).expect("sending request");
                    entry.insert((sender, 1, app));
                } else if let Some(config) = self.function_configs.get(entry.key()) {
                    let cid = self.max_channel.fetch_add(1, Ordering::Relaxed) as u32;
                    let (req_sender, req_receiver) = channel();
                    let app = VmAppConfig {
                        kernel: self.kernel.clone(),
                        instance_id: config.name.clone(),
                        rootfs: config.runtimefs,
                        appfs: Some(config.appfs),
                        cmd_line: self.cmd_line.clone(),
                        seccomp_level: self.seccomp_level,
                        vsock_cid: cid,
                        notifier: self.notifier.try_clone().expect("Failed to clone notifier"),
                        // we really want this to be a function of VPU and memory count, so that
                        // cpu_share is proportional to the size of the function
                        cpu_share: config.vcpus,
                        mem_size_mib: Some(config.memory),
                        load_dir: None, // ignored by now
                        dump_dir: None, // ignored by now
                    }.run();
                    self.channels.lock().expect("poisoned lock").insert(cid,
                        (entry.key().clone(), req_receiver, app.connection.try_clone().expect("Failed to clone VmApp's pipe pair")));

                    req_sender.send(req).expect("sending request");
                    entry.insert((req_sender, 1, app));
                } else {
                    panic!("Bad function name {}", entry.key());
                }

            },
        }
    }

    pub fn process_response(&self, response: (String, Vec<u8>)) {
        let (function, response) = response;
        println!("{}: {}", function, String::from_utf8(response).unwrap());
        let mut active = self.active_functions.lock().unwrap();
        let mut warm = self.warm_functions.lock().unwrap();
        let (sender, mut outstanding, app) = active.remove(&function).expect("active function not in active_functions?");
        outstanding -= 1;
        if outstanding > 0 {
            active.insert(function, (sender, outstanding, app));
        } else {
            warm.insert(function, (sender, app));
        }
        println!("Warm {}, Active: {}", warm.len(), active.len());
    }
}

pub struct Handle(Vec<JoinHandle<()>>);

impl Handle {
    #[allow(dead_code)]
    pub fn join(mut self) -> std::thread::Result<()> {
        let mut result = Ok(());
        while let Some(handle) = self.0.pop() {
            match handle.join() {
                Ok(_) => (),
                Err(e) => result = Err(e),
            }
        }
        result
    }
}
