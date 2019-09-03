// This module represents states of the physical cluster.
// The states currently tracked are cpu and memory
extern crate num_cpus;

use std::fs::File;
use std::io::{BufReader, BufRead};

const MEM_FILE: &str = "/proc/meminfo";     // meminfo file on linux
const KB_IN_MB: usize = 1024;
const MEM_4G: usize = 4096;  // in MB

#[derive(Clone, Debug)]
pub struct MachineInfo {
    id: String,
    total_cpu: u64,       // number of cores
    total_mem: usize,       // amount of memory (MB)
//    total_storage: u32,   // amount of storage (MB)
    free_cpu: u64,
    free_mem: usize,
}

#[derive(Clone, Debug)]
pub struct Cluster {
    num_hosts: u32,     // number of physical hosts in the cluster
    host_list: Vec<MachineInfo>,       // host name to machine config mapping
    pub total_cpu: u64,     // total number of core across all hosts in the cluster
    pub total_mem: usize,     // total amount of memroy across all hosts in the cluster (MB)
    total_free_cpu: u64,
    total_free_mem: usize,
}

impl Cluster{
    pub fn new() -> Cluster {
        Cluster::single_machine_cluster()
    }

    // Currently we only support one-machine clusters.
    // So this function acquires physical resource information from the host
    // on which the controller is running which is just one machine.
    fn single_machine_cluster() -> Cluster {
        let cpus = num_cpus::get() as u64;     // logical CPUs

        let mut mem: usize = 0;

        let memfile = File::open(MEM_FILE).expect("Couldn't open /proc/meminfo");
        for line in BufReader::new(memfile).lines(){
            match line {
                Ok(c) => {
                    let parts: Vec<&str> = c.split(':').map(|s| s.trim()).collect();
                    if parts[0] == "MemTotal" {
                        mem = parts[1].split(' ').collect::<Vec<&str>>()[0].parse::<usize>().unwrap();
                        break;
                    }
                },
                Err(e) => println!("Reading meminfo file error: {:?}", e)
            }
        }

        let mem = mem / KB_IN_MB - MEM_4G;

        let mc = MachineInfo{
            id: String::from("1"),
            total_cpu: cpus,
            total_mem: mem,
            free_cpu: cpus,
            free_mem: mem,
        };

        Cluster{
            num_hosts: 1,
            host_list: vec![mc],
            total_cpu: cpus,
            total_mem: mem,
            total_free_cpu: cpus,
            total_free_mem: mem,
        }
    }

    // Find a machine in the cluster that has enough resources to boot a new VM for a function
    pub fn find_free_machine(&self, req_mem: usize) -> Option<(u32, &MachineInfo)> {

        for (i, m) in self.host_list.iter().enumerate() {
            if m.free_mem >= req_mem {
                return Some((i as u32,m));
            }
        }
        return None;
    }

    pub fn allocate(&mut self, id: u32, req_mem: usize) {
        self.total_free_mem = self.total_free_mem - req_mem;
        self.host_list.get_mut(id as usize).unwrap().allocate(req_mem);
    }

    pub fn free_resources(&self) -> (u64, usize) {
        (self.total_free_cpu, self.total_free_mem)
    }

    pub fn free(&mut self, id: u32, mem: usize) {
        self.total_free_mem = self.total_free_mem + mem;
        self.host_list.get_mut(id as usize).unwrap().free(mem);
    }

}

impl MachineInfo {
    pub fn allocate(&mut self, req_mem: usize) {
        self.free_mem = self.free_mem - req_mem;
    }

    pub fn free(&mut self, req_mem: usize) {
        self.free_mem = self.free_mem + req_mem;
    }
}
