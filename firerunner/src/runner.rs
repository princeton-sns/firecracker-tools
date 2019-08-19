use cgroups::{self, Cgroup, cgroup_builder::CgroupBuilder};
use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::os::unix::io::FromRawFd;
use std::sync::{Arc, RwLock};
use nix::unistd::{self, Pid, ForkResult};
use vmm::vmm_config::boot_source::BootSourceConfig;
use vmm::vmm_config::drive::BlockDeviceConfig;
use vmm::vmm_config::machine_config::VmConfig;
use vmm::vmm_config::instance_info::{InstanceInfo, InstanceState};

use crate::vmm_wrapper::VmmWrapper;

pub struct VmAppConfig {
    pub instance_id: String,
    pub vsock_cid: u32,
    pub kernel: String,
    pub rootfs: PathBuf,
    pub appfs: Option<PathBuf>,
    pub cmd_line: String,
    pub seccomp_level: u32,
    pub cpu_share: u64,
    pub mem_size_mib: Option<usize>,
    pub load_dir: Option<PathBuf>, // ignored by now
    pub dump_dir: Option<PathBuf>, // ignored by now
}

pub struct VmApp {
    pub config: VmAppConfig,
    cgroup_name: PathBuf,
    pub process: Pid,
    pub requests_input: File,
    pub response_reader: BufReader<File>,
    pub ready_checker: File,
}

impl VmApp {
    pub fn kill(&mut self) {
        nix::sys::signal::kill(self.process, nix::sys::signal::Signal::SIGKILL).expect("Failed to kill child");
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
    pub fn run(self) -> VmApp {
        let (request_reader, request_writer) = nix::unistd::pipe().unwrap();
        let (checker, notifier) = nix::unistd::pipe().unwrap();
        let (response_reader, response_writer) = nix::unistd::pipe().unwrap();
        match unistd::fork() {
            Err(_) => panic!("Couldn't fork!!"),
            Ok(ForkResult::Parent { child, .. }) => {
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
                    requests_input: unsafe { File::from_raw_fd(request_writer) },
                    response_reader: BufReader::new(unsafe { File::from_raw_fd(response_reader) }),
                    ready_checker: unsafe { File::from_raw_fd(checker) },
                }
            },
            Ok(ForkResult::Child) => {
                let shared_info = Arc::new(RwLock::new(InstanceInfo {
                    state: InstanceState::Uninitialized,
                    id: self.instance_id.clone(),
                    vmm_version: "0.1".to_string(),
                    load_dir: self.load_dir,
                    dump_dir: self.dump_dir,
                }));

                let mut vmm = VmmWrapper::new(shared_info, self.seccomp_level,
                                              unsafe { File::from_raw_fd(response_writer) },
                                              unsafe { File::from_raw_fd(request_reader) },
                                              unsafe { File::from_raw_fd(notifier) });

                let machine_config = VmConfig{
                    vcpu_count: Some(self.cpu_share as u8),
                    mem_size_mib: self.mem_size_mib,
                    ..Default::default()
                };
                vmm.set_configuration(machine_config).expect("set config");

                let boot_config = BootSourceConfig {
                    kernel_image_path: self.kernel,
                    boot_args: Some(self.cmd_line),
                };
                vmm.set_boot_source(boot_config).expect("bootsource");

                let block_config = BlockDeviceConfig {
                    drive_id: String::from("rootfs"),
                    path_on_host: self.rootfs,
                    is_root_device: true,
                    is_read_only: true,
                    partuuid: None,
                    rate_limiter: None,
                };
                vmm.insert_block_device(block_config).expect("Rootfs");
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

                vmm.start_instance().expect("Start");
                vmm.join();
                std::process::exit(0);
            }
        }
    }
}
