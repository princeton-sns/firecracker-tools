use std::collections::btree_map::BTreeMap;
use std::default::Default;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::fs::File;
use std::os::unix::io::FromRawFd;

use super::config;
use super::listener;
use super::request;
use super::cluster;
use super::metrics::Metrics;

use firerunner::runner::{VmApp, VmAppConfig};
use firerunner::pipe_pair::PipePair;

const MEM_4G: usize = 4096;  // in MB
const VM_SIZE_INCREMENT: usize = 128; // in MB
const CPU_SHARE_INCREMENT: usize = 64;
// represent an VM from a management perspective
// differs from runner::VmApp or VmAppConfig that represent an Vm from execution perspective
#[derive(Debug)]
pub struct Vm {
    pub id: u32,
    pub req_sender: Sender<request::Request>,
    pub app: VmApp,
}

pub struct Inner {
    cluster: cluster::Cluster,       // track physical resources
    running_functions: BTreeMap<String, BTreeMap<u32, Vec<Vm>>>,
    idle_functions: BTreeMap<String, BTreeMap<u32, Vec<Vm>>>,

    channels: Arc<Mutex<BTreeMap<u32, (String, u32, Receiver<request::Request>, PipePair)>>>,
    vm_id_counter: AtomicUsize,     // monotonically increase for each vm created

    function_configs: config::Configuration,    // in-memory function config store
    seccomp_level: u32,
    cmd_line: String,
    kernel: String,
    stat: Arc<Mutex<Metrics>>,
    notifier: File,
    debug: bool,          // whether VMs keeps stdout
    snapshot: bool,
    one_hyperthread_mem_size: usize,
}

pub struct Controller {
    inner: Arc<Mutex<Inner>>,
    listener: File,       // this is cloned and used by RequestManger
}

impl Controller {
    pub fn new(function_configs: config::Configuration, seccomp_level: u32,
               cmd_line: String, kernel: String, debug: bool,
               snapshot: bool, mem_size: usize) -> Controller {

        let (listener, notifier) = nix::unistd::pipe().expect("Failed to create a pipe");

        // initialize running and idle lists upfront
        let mut running_functions = BTreeMap::new();
        let mut idle_functions = BTreeMap::new();

        for f in function_configs.configs.keys() {
            let mut user_running_functions = BTreeMap::new();
            let mut user_idle_functions = BTreeMap::new();
            let num_users = function_configs.get(&f).unwrap().users;

            for u in 0..num_users {
                user_running_functions.insert(u, Vec::new());
                user_idle_functions.insert(u, Vec::new());
            }

            running_functions.insert(f.clone(), user_running_functions);
            idle_functions.insert(f.clone(), user_idle_functions);
        }

        let my_cluster = cluster::Cluster::new(mem_size);

        let mut one_hyperthread_mem_size: usize = 1024;

        if mem_size == 0 {
            one_hyperthread_mem_size = my_cluster.total_mem / my_cluster.total_cpu as usize;
        }

        Controller {
            inner: Arc::new(Mutex::new(Inner {
                cluster: my_cluster,
                running_functions,
                idle_functions,

                channels: Default::default(),
                vm_id_counter: AtomicUsize::new(3),
                seccomp_level,
                cmd_line,
                kernel,
                function_configs,
                stat: Arc::new(Mutex::new(Metrics::new())),
                notifier: unsafe{ File::from_raw_fd(notifier) },
                debug,
                snapshot,
                one_hyperthread_mem_size,
            })),

            listener: unsafe{ File::from_raw_fd(listener) },
        }
    }

    pub fn schedule(&mut self, req: request::Request) {
        self.inner.lock().unwrap().aws_schedule(req);
    }

    pub fn ignite(&mut self) -> Handle {
        let (response_sender, response_receiver) = channel();

        // Create RequestManager thread
        let listener = self.listener.try_clone().expect("Failed to clone pipe listener");
        let (chans, stats) = {
            let inner = self.inner.lock().unwrap();
            (inner.channels.clone(), inner.stat.clone())
        };

        let manager_handle = listener::RequestManager::new(chans,
                                                           stats,
                                                           response_sender, listener)
                                                      .spawn();

        // Create ResponseHandler thread
        let inner = self.inner.clone();
        let response_handle = thread::spawn(move || {
            for response in response_receiver.iter() {
                inner.lock().unwrap().process_response(response);
            }
        });

        Handle (vec![manager_handle, response_handle])
    }

    // check if there's any running function
    pub fn check_running(&self) -> u64{
        let mut num_running: u64 = 0;
        for (func_name, run_tree) in self.inner.lock().unwrap().running_functions.iter() {
            for (user_id, run_list) in run_tree.iter() {
                num_running = num_running + run_list.len() as u64;
                //println!("Function {} still has {} VMs running", func_name, run_list.len());
            }
        }
        return num_running;
    }

    // kill all vms
    pub fn kill_all(&mut self) {
        for idle_tree in self.inner.lock().unwrap().idle_functions.values_mut() {
            for vms in idle_tree.values_mut(){
                vms.clear()
            }
        }

        for run_tree in self.inner.lock().unwrap().running_functions.values_mut() {
            for vms in run_tree.values_mut() {
                vms.clear()
            }
        }
    }

    pub fn get_cluster_info(&self) -> cluster::Cluster {
        self.inner.lock().unwrap().cluster.clone()
    }

    pub fn get_stat(&self) -> Metrics {
        self.inner.lock().unwrap().stat.lock().unwrap().clone()
    }
}

impl Inner {

    // Send a request to the vm. If success, push the vm to the running_function vector.
    // If not, push the vm the idle_function vector.
    fn send_request(&mut self, req: request::Request, vm: Vm) {
        let function_name = req.function.clone();
        let request_sender = vm.req_sender.clone();
        let user_id = req.user_id;
        let vm_id = vm.id;

        self.running_functions.get_mut(&function_name).unwrap().get_mut(&user_id).unwrap().push(vm);

        if let Err(e) = request_sender.send(req) {
            println!("Request failed to send to vm: {}, error: {}", vm_id, e);

            let vm = self.find_and_remove_running_vm(&function_name, &user_id, vm_id).unwrap();
            let idle_list = self.idle_functions.get_mut(&function_name).unwrap().get_mut(&user_id).unwrap();
            idle_list.push(vm);
        }

    }


    fn aws_schedule(&mut self, req: request::Request) {
        // Check if I have an idle VM
        match self.get_idle_vm(&req) {
            Some(vm) => {
//                println!("Found idle VM for {}", req.function);
//                self.stat.lock().unwrap().log_request_timestamp(vm.id, time::precise_time_ns());
                self.send_request(req, vm);
            },
            None => {
                if self.check_concurrency(&req) {
//                    println!("Dropping request for {}", &req.function);
                    self.stat.lock().unwrap().drop_req_concurrency(1);
                    self.stat.lock().unwrap().drop_req(1);
                    return;
                }

                // Check if there's enough free resource to launch a new VM
                let (_, req_mem) = self.function_configs.resource_req(&req.function).unwrap();

                match self.cluster.find_free_machine(req_mem) {
                    Some((host_id,_)) => {
                        self.cluster.allocate(host_id, req_mem);

                        let new_vm = self.launch_new_vm(&req);
//                        println!("New VM: {:?}", new_vm);
//                        self.stat.lock().unwrap()
//                            .log_request_timestamp(new_vm.id, time::precise_time_ns());
                        self.send_request(req, new_vm);
                    },
                    // Evict an idle VM running some other functions
                    None => {
//                        println!("No free resources, picking a VM to evict");
                        if let Some((evict_vm, evict_mem)) = self.get_evictable_vm(&req) {

                            self.stat.lock().unwrap().evict_vm(1);
                            let new_vm = self.evict_and_swap(&req, evict_vm);

                            self.cluster.free(0, evict_mem);
                            let (_, req_mem) = self.function_configs
                                                         .resource_req(&req.function)
                                                         .unwrap();
                            self.cluster.allocate(0, req_mem);

//                            println!("new vm {:?}", &new_vm);
//                            self.stat.lock().unwrap()
//                                .log_request_timestamp(new_vm.id, time::precise_time_ns());
                            self.send_request(req, new_vm);
                       } else {
                            println!("Dropping request for {}", &req.function);
                            self.stat.lock().unwrap().drop_req_resource(1);
                            self.stat.lock().unwrap().drop_req(1);
                        }
                        return;
                    }
                }
            }
        }
    }

//    fn omni_schedule(&self, req: request::Request) {
//
//    }

    fn get_current_concurrency(&self, req: &request::Request) -> usize {
        self.running_functions.get(&req.function).unwrap().get(&req.user_id).unwrap().len() +
        self.idle_functions.get(&req.function).unwrap().get(&req.user_id).unwrap().len()

    }

    fn check_concurrency(&self, req: &request::Request) -> bool {
        let curr_concur = self.get_current_concurrency(&req);
        curr_concur >= self.function_configs.get(&req.function).unwrap().concurrency_limit

    }

        // For a particular function, acquire an idle VM instance
    pub fn get_idle_vm(&mut self, req: &request::Request) -> Option<Vm> {
        if let Some(idle_tree) = self.idle_functions.get_mut(&req.function){
            if let Some(vms) = idle_tree.get_mut(&req.user_id) {
                return vms.pop();
            }
        }
        None
    }

    pub fn get_evictable_vm(&mut self, req: &request::Request) -> Option<(Vm, usize)> {
        let req_cpu: u64 = self.function_configs.get(&req.function).unwrap().vcpus;
        let req_mem: usize = self.function_configs.get(&req.function).unwrap().memory;
        let user_id: u32 = req.user_id;

        for (func_name, idle_tree) in self.idle_functions.iter_mut() {
            for (id, idle_list) in idle_tree.iter_mut(){
                if func_name != &req.function {
                    let evict_mem: usize = self.function_configs.get(&func_name).unwrap().memory;

                    if evict_mem >= req_mem && idle_list.len() > 0 {
//                    println!("Found evictable VM of function {}", func_name);
                        return Some((idle_list.pop().unwrap(), evict_mem));
                    }
                } else {
                    if *id != user_id && idle_list.len() > 0 {
                        return Some((idle_list.pop().unwrap(), req_mem));
                    }
                }
            }
        }
//        println!("Couldn't find evictable vm that meets resources requirements");
        None

    }

    // kill the vm process
    // free vm's resources in the cluster
    // free the Vm struct
    pub fn evict(&self, evict_vm: Vm) {
//        println!("trying to evict");
//        let mem = evict_vm.app.config.mem_size_mib.unwrap();
//        let cpu = evict_vm.app.config.cpu_share;
//        println!("{:?}", self.cluster.lock().unwrap());
//        println!("freeing cpu: {}, mem: {}", cpu, mem);
//        self.cluster.lock().unwrap().free(cpu, mem);
//        println!("{:?}", self.cluster.lock().unwrap());
//        evict_vm.app.kill(); // vm is automatically killed with its VmApp instance is dropped
//        println!("vm killed");
    }

    pub fn evict_and_swap(&self, req: &request::Request, evict_vm: Vm) -> Vm {
        let t0 = time::precise_time_ns();
        let id = evict_vm.id;
        self.evict(evict_vm);
        let t1 = time::precise_time_ns();
        {
            let mut stat = self.stat.lock().unwrap();
            stat.log_eviction_timestamp(id, t0);
            stat.log_eviction_timestamp(id, t1);
        }
        self.launch_new_vm(req)
    }

    fn cpu_share(&self, mem: usize) -> u64 {
        ((mem / VM_SIZE_INCREMENT) * CPU_SHARE_INCREMENT) as u64
    }

    fn vcpu_count(&self, mem: usize) -> u64 {
        let count = (mem as f64 / self.one_hyperthread_mem_size as f64).ceil();
        count as u64
    }

    pub fn launch_new_vm(&self, req: &request::Request) -> Vm {
        let config = self.function_configs.get(&req.function).unwrap();

        let id = self.vm_id_counter.fetch_add(1, Ordering::Relaxed) as u32;
        let (req_sender, req_receiver) = channel();

        let mut load_dir = None;
        if self.snapshot{
            load_dir = config.load_dir;
        }

        let mem = config.memory;
        let cpu_share = self.cpu_share(mem);
        let vcpu_count = self.vcpu_count(mem);

        {
            self.stat.lock().unwrap().log_boot_timestamp(id, time::precise_time_ns());
            self.stat.lock().unwrap().log_vm_mem_size(id, mem);
        }

        let app = VmAppConfig {
            kernel: self.kernel.clone(),
            //kernel: String::from("foo"),
            instance_id: config.name.clone(),
            rootfs: config.runtimefs,
            appfs: Some(config.appfs),
            cmd_line: self.cmd_line.clone(),
            seccomp_level: self.seccomp_level,
            vsock_cid: id,
            notifier: self.notifier.try_clone().expect("Failed to clone notifier"),
            // we really want this to be a function of VPU and memory count, so that
            // cpu_share is proportional to the size of the function
            cpu_share: cpu_share,
            vcpu_count: vcpu_count,
            mem_size_mib: Some(config.memory),
            load_dir,
            dump_dir: None, // ignored by now
        }.run(self.debug);

        self.channels.lock()
            .expect("poisoned lock")
            .insert(id,
                    (req.function.clone(),
                     req.user_id,
                     req_receiver,
                     app.connection.try_clone().expect("Failed to clone VmApp's pipe pair"))
            );

        Vm {
            id,
            req_sender,
            app,
        }
    }

    fn find_and_remove_running_vm(&mut self, function_name: &String, user_id: &u32, vm_id: u32) -> Option<Vm> {

        let running_tree = self.running_functions.get_mut(function_name).unwrap();
        let running_list = running_tree.get_mut(user_id).unwrap();

        for (idx, vm) in running_list.iter().enumerate() {
            if vm.id == vm_id {
                let vm = running_list.remove(idx);
                return Some(vm);
            }
        }
        return None;
    }

    pub fn process_response(&mut self, response: (u32, u32, String, Vec<u8>)) {
        let (id, user_id, function, response) = response;
//        self.stat.lock().unwrap().log_request_timestamp(id, time::precise_time_ns());
        //println!("{}, {}, {}: {}", id, user_id, function, String::from_utf8(response).unwrap());

        self.stat.lock().unwrap().complete_req(1);


        if let Some(vm) = self.find_and_remove_running_vm(&function, &user_id, id) {
            let idle_list = self.idle_functions.get_mut(&function).unwrap().get_mut(&user_id).unwrap();
            idle_list.push(vm);
        } else {
            panic!("results returned from a non-running VM (id: {})", id);
        }

//        {
//            let mut running_tree = self.running_functions.lock().unwrap();
//            let running_list = running_tree.get_mut(&function).unwrap();
//            let mut idle_tree = self.idle_functions.lock().unwrap();
//            let idle_list = idle_tree.get_mut(&function).unwrap();
//
//            println!("Function {}, running: {}, idle: {}", function, running_list.len(), idle_list.len());
//        }

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
