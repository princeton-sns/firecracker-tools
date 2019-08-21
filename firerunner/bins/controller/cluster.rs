// This module represents states of the physical cluster.
// The states currently tracked are cpu and memory
extern crate num_cpus;

use std::collections::btree_map::BTreeMap;
use vmm::VmmActionError::MachineConfig;
use std::fs::File;
use cgroups::Controllers::Mem;
use std::io::{BufReader, BufRead};

const MEM_FILE: &str = "/proc/meminfo";     // meminfo file on linux

#[derive(Debug)]
pub struct MachineInfo {
    id: String,
    total_cpu: u16,       // number of cores
    total_mem: u32,       // amount of memory (MB)
//    total_storage: u32,   // amount of storage (MB)
    free_cpu: u16,
    free_mem: u32,
}

#[derive(Debug)]
pub struct Cluster {
    num_hosts: u32,     // number of physical hosts in the cluster
    host_list: Vec<MachineInfo>,       // host name to machine config mapping
    total_cpu: u16,     // total number of core across all hosts in the cluster
    total_mem: u32,     // total amount of memroy across all hosts in the cluster (MB)
    total_free_cpu: u16,
    total_free_mem: u32,
}

impl Cluster{
    pub fn new() -> Cluster {
        Cluster::single_machine_cluster()
    }

    // Currently we only support one-machine clusters.
    // So this function acquires physical resource information from the host
    // on which the controller is running which is just one machine.
    fn single_machine_cluster() -> Cluster {
        let cpus = num_cpus::get() as u16;     // logical CPUs

        let mut mem = 0;

        let memfile = File::open(MEM_FILE).expect("Couldn't open /proc/meminfo");
        for line in BufReader::new(memfile).lines(){
            match line {
                Ok(c) => {
                    // example line with total memory information:
                    // MemTotal:       16322876 kB
                    let parts: Vec<&str> = c.split(':').map(|s| s.trim()).collect();
                    if parts[0] == "MemTotal" {
                        mem = parts[1].split(' ').collect::<Vec<&str>>()[0].parse::<u32>().unwrap();
                        break;
                    }
                },
                Err(e) => println!("Reading meminfo file error: {:?}", e)
            }
        }

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
    pub fn find_free_machine(&self, req_cpu: u16, req_mem: u32) -> Option<(usize, &MachineInfo)> {

        for (i, m) in self.host_list.iter().enumerate() {
            if m.free_cpu >= req_cpu && m.free_mem >= req_mem {
                return Some((i,m));
            }
        }
        return None;
    }

}