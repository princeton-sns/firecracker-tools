use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use nix::unistd::{self, Pid, ForkResult};
use vmm::vmm_config::boot_source::BootSourceConfig;
use vmm::vmm_config::drive::BlockDeviceConfig;
use vmm::vmm_config::vsock::VsockDeviceConfig;
use vmm::vmm_config::machine_config::VmConfig;
use vmm::vmm_config::instance_info::{InstanceInfo, InstanceState};

use crate::vmm_wrapper::VmmWrapper;

pub struct VmAppConfig {
    pub instance_id: String,
    pub vsock_cid: u32,
    pub kernel: String,
    pub rootfs: String,
    pub appfs: Option<String>,
    pub cmd_line: String,
    pub seccomp_level: u32,
    pub mem_size_mib: Option<usize>,
    pub vcpu_count: Option<u8>,
    pub load_dir: Option<PathBuf>,
    pub dump_dir: Option<PathBuf>,
}

pub struct VmApp {
    pub config: VmAppConfig,
    pub process: Pid,
}

impl VmApp {
    pub fn kill(self) {
        nix::sys::signal::kill(self.process, nix::sys::signal::Signal::SIGKILL).expect("Failed to kill child");
        nix::sys::wait::waitpid(self.process, None).expect("Failed to kill child");
    }
}

impl VmAppConfig {
    pub fn run(self) -> VmApp {
        match unistd::fork() {
            Err(_) => panic!("Couldn't fork!!"),
            Ok(ForkResult::Parent { child, .. }) => {
                return VmApp {
                    config: self,
                    process: child,
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

                let mut vmm = VmmWrapper::new(shared_info, self.seccomp_level);

                let machine_config = VmConfig{
                    vcpu_count: self.vcpu_count,
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
                    path_on_host: PathBuf::from(self.rootfs),
                    is_root_device: true,
                    is_read_only: true,
                    partuuid: None,
                    rate_limiter: None,
                };
                vmm.insert_block_device(block_config).expect("Rootfs");
                if let Some(appfs) = self.appfs {
                    let block_config = BlockDeviceConfig {
                        drive_id: String::from("appfs"),
                        path_on_host: PathBuf::from(appfs),
                        is_root_device: false,
                        is_read_only: true,
                        partuuid: None,
                        rate_limiter: None,
                    };
                    vmm.insert_block_device(block_config).expect("AppBlk");
                }

                vmm.add_vsock(VsockDeviceConfig { id: self.instance_id, guest_cid: self.vsock_cid }).expect("vsock");

                vmm.start_instance().expect("Start");
                vmm.join();
                std::process::exit(0);
            }
        }
    }
}
