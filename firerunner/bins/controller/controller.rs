use std::collections::btree_map::{BTreeMap, Entry};
use std::default::Default;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

use super::config;
use super::listener;
use super::request;
use super::cluster;
use super::metrics::Metrics;

use firerunner::runner::{VmApp, VmAppConfig};
use firerunner::vsock::VsockCloser;
use std::borrow::BorrowMut;
use std::ops::Deref;

// represent an VM from a management perspective
// differs from runner::VmApp or VmAppConfig that represent an Vm from execution perspective
#[derive(Debug)]
pub struct Vm {
    cid: u32,
    req_sender: Sender<request::Request>,
    app: VmApp,
}


pub struct Inner {
    cluster: Mutex<cluster::Cluster>,
    running_functions: Mutex<BTreeMap<String, Vec<Vm>>>,
    idle_functions: Mutex<BTreeMap<String, Vec<Vm>>>,

    active_functions: Mutex<BTreeMap<String, (Sender<request::Request>, usize, VmApp)>>,
    warm_functions: Mutex<BTreeMap<String, (Sender<request::Request>, VmApp)>>,
    channels: Arc<Mutex<BTreeMap<u32, (String, Receiver<request::Request>)>>>,
    max_channel: AtomicUsize,

    function_configs: config::Configuration,
    seccomp_level: u32,
    cmd_line: String,
    kernel: String,
    stat: Mutex<Metrics>,
}

pub struct Controller {
    inner: Arc<Inner>,
    vsock_closer: Option<VsockCloser>,
}

impl Controller {
    pub fn new(function_configs: config::Configuration, seccomp_level: u32,
               cmd_line: String, kernel: String) -> Controller {

        // initialize running and idle lists upfront
        let running_functions = Mutex::new(BTreeMap::new());
        let idle_functions = Mutex::new(BTreeMap::new());

        for f in function_configs.configs.keys() {
            running_functions.lock().unwrap().insert(f.clone(), Vec::new());
            idle_functions.lock().unwrap().insert(f.clone(), Vec::new());
        }

        Controller {
            inner: Arc::new(Inner {
                cluster: Mutex::new(cluster::Cluster::new()),
                running_functions,
                idle_functions,

                active_functions: Default::default(),
                warm_functions: Default::default(),
                channels: Default::default(),
                max_channel: AtomicUsize::new(3),
                seccomp_level,
                cmd_line,
                kernel,
                function_configs,
                stat: Mutex::new(Metrics::new()),
            }),
            vsock_closer: None,
        }
    }

    pub fn schedule(&mut self, req: request::Request) {
        self.inner.aws_schedule(req);
    }

    pub fn ignite(&mut self) -> Handle {
        let (response_sender, response_receiver) = channel();

        let (manager_handle, vsock_closer) = listener::RequestManager::new(self.inner.channels.clone(), response_sender).spawn();
        self.vsock_closer = Some(vsock_closer);

        let inner = self.inner.clone();
        let response_handle = thread::spawn(move || {
            for response in response_receiver.iter() {
                inner.process_response(response);
            }
        });

        Handle (vec![manager_handle, response_handle])
    }

    pub fn check_running(&self) -> bool {
        for (func_name, run_list) in self.inner.running_functions.lock().unwrap().iter() {
            if run_list.len() > 0 {
                return true;
            }
        }
        return false;
    }

    pub fn kill_all(&mut self) {
        while self.inner.active_functions.lock().unwrap().len() > 0 {
            thread::yield_now();
        }
        self.vsock_closer.take().map(|mut c| c.close()).unwrap().unwrap();
        self.inner.warm_functions.lock().unwrap().clear();
    }

    pub fn get_cluster_info(&self) -> MutexGuard<cluster::Cluster> {
        self.inner.cluster.lock().unwrap()
    }
}

impl Inner {

    fn aws_schedule(&self, req: request::Request) {
        // Check if I have an idle VM
        match self.get_idle_vm(&req) {
            Some(vm) => {
                println!("Found idle VM for {}", req.function);
                let function_name = req.function.clone();
                match vm.req_sender.send(req) {
                    Ok(_) => {
                        self.running_functions.lock().unwrap().get_mut(&function_name)
                            .unwrap().push(vm);
                    },
                    Err(e) => {
                        println!("Request failed to send to vm: {:?}", vm);
                        self.idle_functions.lock().unwrap().get_mut(&function_name)
                            .unwrap().push(vm);
                    }
                }
            },
            None => {
                // check if creating a new vm for this function would exceeds concurrency
                // limit. This includes both boot a new vm and evicting an existing vm.
                let curr_concur = self.get_current_concurrency(&req);
//                println!("Current concurrency for function {}: {}",
//                         &req.function, &curr_concur);

                if curr_concur >= self.function_configs.get(&req.function)
                                      .unwrap().concurrency_limit {
                    // drop the request
                    println!("Dropping request for {}", &req.function);
                    self.stat.lock().unwrap().drop_req(1);
                    return;
                }

                // Check if there's free (unallocated) resource to launch a new VM
                let req_cpu: u64 = self.function_configs.get(&req.function).unwrap().vcpus;
                let req_mem: usize = self.function_configs.get(&req.function).unwrap().memory;

                let mut cluster = self.cluster.lock().unwrap();
                match cluster.find_free_machine(req_cpu, req_mem) {
                    Some((id,_)) => {
//                        let (free_cpu, free_mem) = cluster.free_resources();
//                        println!("Found machine {} with free resources, cpu: {}, mem: {}",
//                                 id, free_cpu, free_mem);

                        cluster.allocate(id, req_cpu, req_mem);
                        let new_vm = self.launch_new_vm(&req);
//                        println!("New VM: {:?}", new_vm);
                        let function_name = req.function.clone();

                        match new_vm.req_sender.send(req) {
                            Ok(_) => {
                                self.running_functions.lock().unwrap().get_mut(&function_name)
                                    .unwrap().push(new_vm);
                            },
                            Err(e) => {
                                println!("Request failed to send to vm: {:?}", new_vm);
                                self.idle_functions.lock().unwrap().get_mut(&function_name)
                                    .unwrap().push(new_vm);
                            }
                        }

                    },
                    // Evict an idle VM running some other functions
                    None => {
                        //TODO
                        return;
                    }
                }
            }
        }
    }

    fn omni_schedule(&self, req: request::Request) {

    }

    fn get_current_concurrency(&self, req: &request::Request) -> usize {
        self.running_functions.lock().unwrap().get(&req.function).unwrap().len() +
        self.idle_functions.lock().unwrap().get(&req.function).unwrap().len()

    }

    // For a particular function, acquire an idle VM instance
    pub fn get_idle_vm(&self, req: &request::Request) -> Option<Vm> {
        match self.idle_functions.lock().unwrap().get_mut(&req.function) {
            Some(vms) => {
                if vms.len() == 0 {
                    return None;
                }
                vms.pop()
            },
            None => None
        }
    }

    pub fn launch_new_vm(&self, req: &request::Request) -> Vm {
        let config = self.function_configs.get(&req.function).unwrap();

        let cid = self.max_channel.fetch_add(1, Ordering::Relaxed) as u32;
        let (req_sender, req_receiver) = channel();
        self.channels.lock().expect("poisoned lock").insert(cid, (req.function.clone(), req_receiver));
        let app = VmAppConfig {
            kernel: self.kernel.clone(),
            instance_id: config.name.clone(),
            rootfs: config.runtimefs,
            appfs: Some(config.appfs),
            cmd_line: self.cmd_line.clone(),
            seccomp_level: self.seccomp_level,
            vsock_cid: cid,
            // we really want this to be a function of VPU and memory count, so that
            // cpu_share is proportional to the size of the function
            cpu_share: config.vcpus,
            mem_size_mib: Some(config.memory),
            load_dir: None, // ignored by now
            dump_dir: None, // ignored by now
        }.run();

        Vm{
            cid,
            req_sender,
            app,
        }
    }


    pub fn process_response(&self, response: (u32, String, Vec<u8>)) {
        let (cid, function, response) = response;
        println!("{}, {}: {}", cid, function, String::from_utf8(response).unwrap());

        let mut running_tree = self.running_functions.lock().unwrap();
        let mut running_list = running_tree.get_mut(&function).unwrap();
        let mut idle_tree = self.idle_functions.lock().unwrap();
        let mut idle_list = idle_tree.get_mut(&function).unwrap();

        for (idx, vm) in running_list.iter().enumerate() {
            if vm.cid == cid {
                let vm = running_list.remove(idx);
                idle_list.push(vm);
                break;
            }
        }
        println!("Function {}, running: {}, idle: {}", function, running_list.len(), idle_list.len());

//        let mut active = self.active_functions.lock().unwrap();
//        let mut warm = self.warm_functions.lock().unwrap();
//        let (sender, mut outstanding, app) = active.remove(&function).expect("active function not in active_functions?");
//        outstanding -= 1;
//        if outstanding > 0 {
//            active.insert(function, (sender, outstanding, app));
//        } else {
//            warm.insert(function, (sender, app));
//        }
//        println!("Warm {}, Active: {}", warm.len(), active.len());
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
