use cgroups::{self, Cgroup, cgroup_builder::CgroupBuilder};
use std::path::PathBuf;
use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::sync::{Arc, RwLock};
use std::collections::BTreeSet;
use nix::unistd::{self, Pid, ForkResult};
use vmm::vmm_config::boot_source::BootSourceConfig;
use vmm::vmm_config::drive::BlockDeviceConfig;
use vmm::vmm_config::net::NetworkInterfaceConfig;
use vmm::vmm_config::machine_config::VmConfig;
use vmm::vmm_config::instance_info::{InstanceInfo, InstanceState};
use net_util::MacAddr;
use fc_util::{now_monotime_us, now_cputime_us};
use memory_model::MemoryMapping;
use std::io::BufReader;

use crate::vmm_wrapper::VmmWrapper;
use super::pipe_pair::PipePair;

#[derive(Debug)]
pub struct VmAppConfig {
    pub instance_id: String,
    pub vsock_cid: u32,
    pub notifier: File, // write end of a pipe
    pub kernel: String,
    pub rootfs: PathBuf,
    pub appfs: Option<PathBuf>,
    pub cmd_line: String,
    pub seccomp_level: u32,
    pub cpu_share: u64,
    pub vcpu_count: u64,
    pub mem_size_mib: Option<usize>,
    pub load_dir: Option<PathBuf>,
    pub dump_dir: Option<PathBuf>,
    pub diff_dirs: Vec<PathBuf>,
    pub hugepage: bool,
    pub copy_base: bool, // ignored when load_dir is none
    pub copy_diff: bool, // ignored when diff_dir is none
    pub network: Option<String>,
}

#[derive(Debug)]
pub struct VmApp {
    pub config: VmAppConfig,
    cgroup_name: PathBuf,
    pub process: Pid,
    pub connection: PipePair,
}

impl VmApp {
    pub fn kill(&mut self) {
//        println!("issuing kill signal to process: {}", &self.process);
        nix::sys::signal::kill(self.process, nix::sys::signal::Signal::SIGKILL).expect("Failed to kill child");
//        println!("waiting for process: {}", &self.process);
        self.wait();
    }

    pub fn wait(&mut self) {
        nix::sys::wait::waitpid(self.process, None).expect("Failed to kill child");
    }
}

impl Drop for VmApp {
    fn drop(&mut self) {
        self.kill();
        let v1 = cgroups::hierarchies::V1::new();
        let cgroup = Cgroup::load(&v1, self.cgroup_name.to_str().unwrap());
        cgroup.delete();
    }
}

impl VmAppConfig {
    pub fn run(self, debug: bool) -> VmApp {
        let (request_reader, request_writer) = nix::unistd::pipe().unwrap();
        let (response_reader, response_writer) = nix::unistd::pipe().unwrap();
        match unistd::fork() {
            Err(_) => panic!("Couldn't fork!!"),
            Ok(ForkResult::Parent { child, .. }) => {
                //println!("I'm Parent. Child Process Pid: {}", child);
                let pid = child.as_raw() as u64;
                let v1 = cgroups::hierarchies::V1::new();
                let cgroup_name = std::path::Path::new("firecracker").join(pid.to_string().as_str());
                let cgroup = CgroupBuilder::new(cgroup_name.to_str().unwrap(), &v1)
                    .cpu()
                        .shares(self.cpu_share)
                        .done()
                    .build();
                {
                    use cgroups::Controller;
                    let cpus: &cgroups::cpu::CpuController = cgroup.controller_of().unwrap();
                    cpus.add_task(&(pid.into())).expect("Adding child to Cgroup");
                }
                return VmApp {
                    config: self,
                    cgroup_name: cgroup_name.clone(),
                    process: child,
                    connection: PipePair {
                        requests_input: unsafe { File::from_raw_fd(request_writer) },
                        response_reader: unsafe { File::from_raw_fd(response_reader) },
                    },
                }
            },
            Ok(ForkResult::Child) => {
                //println!("I'm Child. My Pid: {}", nix::unistd::getpid());
                // Close all open file descriptors in the child process
//                for i in 0..2 {
//                     leave stderr open so we can see panics
//                    if i == 2 {
//                        continue;
//                    }
//
//                     stop when close fails (means the file descriptor doesn't exist
//                    if unistd::close(i).is_err() {
//                        break;
//                    }
//                }

                unistd::close(0);
                if !debug {
                    unistd::close(1);
                }

                //let mut gfns_to_pfns = Vec::new();
                //let gfns_to_pfns = if let Some(ref dir) = self.load_dir {
                //    let mut dir = dir.clone();
                //    dir.push("page_numbers");
                //    let page_number_file = File::open(dir.as_path()).expect("Failed to open page_numbers");
                //    let mut res_vec = BufReader::new(page_number_file)
                //        .lines()
                //        .map(|l| l.expect("Failed to read page_numbers")
                //            .parse::<u64>().expect("Failed to parse a page number"))
                //        .collect::<Vec<u64>>();

                //    // Create a memory mapping backed by /dev/shm/SNAPSHOT
                //    dir.pop();
                //    let mut shm_name = String::from("/dev/shm/");
                //    shm_name.push_str(dir.file_name().unwrap().to_str().unwrap());
                //    let shm_fd = File::open(shm_name).expect("Failed to open snapshot memory").into_raw_fd();
                //    let mem_size_byte = self.mem_size_mib.unwrap_or_else(|| 128) << 20;
                //    let mapping = MemoryMapping::new_from_file(mem_size_byte, shm_fd, 0, false, false)
                //        .expect("Failed to create a memory mapping backed by the given snapshot");
                //    // read ahead
                //    let buf = &mut [0u8; 4096];
                //    for &gfn in res_vec.iter() {
                //        let offset = (gfn as usize) << PAGE_SHIFT;
                //        assert_eq!(mapping.read_slice(buf, offset).expect("Failed to read ahead"), 4096);
                //    }
                //    // get a pagemap from gfn to pfn
                //    let pagemap = mapping.get_pagemap(false, 0, 4096);
                //    // check all gfns present in snapshots have pfns
                //    //assert_eq!(pagemap.len(), res_vec.len());
                //    let gfn_set = res_vec.iter().cloned().collect::<BTreeSet<u64>>();
                //    assert!(pagemap.keys().cloned().collect::<BTreeSet<u64>>()
                //        .is_superset(&gfn_set));
                //    for gfn in gfn_set.iter() {
                //        res_vec.push(pagemap[gfn]);
                //    }
                //    res_vec
                //} else {
                //    Vec::new()
                //};

                let start_monotime_us = now_monotime_us();
                let start_cputime_us = now_cputime_us();

                let json_dir = if let Some(dir) = self.diff_dirs.last() {
                    Some(dir.clone())
                } else if let Some(ref dir) = self.load_dir {
                    Some(dir.clone())
                } else {
                    None
                };
                let parsed_json = json_dir.map(|mut dir| {
                        dir.push("snapshot.json");
                        let reader = BufReader::new(File::open(dir).expect("Failed to open snapshot.json"));
                        serde_json::from_reader(reader).expect("Bad snapshot.json")
                    });
                let json_end = now_monotime_us();

                let shared_info = Arc::new(RwLock::new(InstanceInfo {
                    state: InstanceState::Uninitialized,
                    id: self.instance_id.clone(),
                    vmm_version: "0.1".to_string(),
                    start_monotime_us,
                    start_cputime_us,
                }));

                let mut vmm = VmmWrapper::new(shared_info, self.seccomp_level,
                                              self.load_dir.clone(),
                                              parsed_json,
                                              self.dump_dir,
                                              self.diff_dirs,
                                              unsafe { File::from_raw_fd(response_writer) },
                                              unsafe { File::from_raw_fd(request_reader) },
                                              self.notifier,
                                              self.vsock_cid,
                                              self.hugepage,
                                              self.copy_base,
                                              self.copy_diff,
                                              );

                let new_wrapper_end = now_monotime_us();
                let machine_config = VmConfig{
                    vcpu_count: Some(self.vcpu_count as u8),
                    mem_size_mib: self.mem_size_mib,
                    ..Default::default()
                };
                vmm.set_configuration(machine_config).expect("set config");
                let set_config_end = now_monotime_us();
                if self.load_dir.is_none() {
                    let boot_config = BootSourceConfig {
                        kernel_image_path: self.kernel,
                        boot_args: Some(self.cmd_line),
                    };
                    vmm.set_boot_source(boot_config).expect("bootsource");
                }
                let boot_source_end = now_monotime_us();
                let block_config = BlockDeviceConfig {
                    drive_id: String::from("rootfs"),
                    path_on_host: self.rootfs,
                    is_root_device: true,
                    is_read_only: true,
                    partuuid: None,
                    rate_limiter: None,
                };
                vmm.insert_block_device(block_config).expect("Rootfs");
                let rootfs_end = now_monotime_us();
                if let Some(appfs) = self.appfs {
                    let block_config = BlockDeviceConfig {
                        drive_id: String::from("appfs"),
                        path_on_host: appfs,
                        is_root_device: false,
                        is_read_only: true,
                        partuuid: None,
                        rate_limiter: None,
                    };
                    vmm.insert_block_device(block_config).expect("AppBlk");
                }

                if let Some(mac_addr) = self.network {
                    let netif_config = NetworkInterfaceConfig {
                        iface_id: String::from("eth0"),
                        host_dev_name: String::from("tap0"),
                        guest_mac: Some(MacAddr::parse_str(mac_addr.as_str()).expect("MacAddr")),
                        rx_rate_limiter: None,
                        tx_rate_limiter: None,
                        allow_mmds_requests: false,
                        tap: None,
                    };
                    vmm.insert_net_device(netif_config).expect("Network");
                }
                let end = now_monotime_us();
                vmm.start_instance().expect("Start");
                eprintln!("configuring took {} us, json took {} us, wrapper::new took {} us, set_config took {} us, boot_source took {} us, rootfs took {} us, appfs took {} us",
                    end - start_monotime_us,
                    json_end - start_monotime_us,
                    new_wrapper_end - json_end,
                    set_config_end - new_wrapper_end,
                    boot_source_end - set_config_end,
                    rootfs_end - boot_source_end,
                    end - rootfs_end);
                vmm.join();
                std::process::exit(0);
            }
        }
    }
}
