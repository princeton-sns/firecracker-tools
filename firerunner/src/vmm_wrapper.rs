use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{channel, Sender};
use std::thread::JoinHandle;
use std::fs::File;
use std::path::PathBuf;

use futures::Future;
use futures::sync::oneshot;
use vmm::{Snapshot, VmmAction, VmmActionError, VmmData, SnapFaaSConfig};
use vmm::vmm_config::instance_info::InstanceInfo;
use vmm::vmm_config::boot_source::BootSourceConfig;
use vmm::vmm_config::drive::BlockDeviceConfig;
use vmm::vmm_config::net::NetworkInterfaceConfig;
use vmm::vmm_config::vsock::VsockDeviceConfig;
use vmm::vmm_config::machine_config::VmConfig;
use sys_util::EventFd;

pub struct VmmWrapper {
    sender: Sender<Box<VmmAction>>,
    event_fd: Rc<EventFd>,
    shared_info: Arc<RwLock<InstanceInfo>>,
    thread_handle: JoinHandle<()>,
}

impl VmmWrapper {

    pub fn new(
        shared_info: Arc<RwLock<InstanceInfo>>,
        seccomp_level: u32,
        load_dir: Option<PathBuf>,
        parsed_json: Option<Snapshot>,
        dump_dir: Option<PathBuf>,
        diff_dirs: Vec<PathBuf>,
        response_writer: File,
        requests_input: File,
        notifier: File,
        notifier_id: u32,
        huge_page: bool,
        copy_base: bool,
        copy_diff: bool,
    ) -> VmmWrapper {
            let (sender, receiver) = channel();
            let event_fd = Rc::new(EventFd::new().expect("Cannot create EventFd"));

            let snapfaas_config = SnapFaaSConfig {
                load_dir,
                parsed_json,
                // TODO: open memory file before firecracker
                memory_to_load: None,
                dump_dir,
                ready_notifier: Some(notifier),
                notifier_id,
                second_serial: Some(response_writer),
                second_input: Some(requests_input),
                copy_base,
                copy_diff,
                huge_page,
                diff_dirs,
            };
            let thread_handle =
                vmm::start_vmm_thread(shared_info.clone(), event_fd.try_clone().expect("Couldn't clone event_fd"), receiver, seccomp_level, snapfaas_config);

            VmmWrapper {
                sender,
                event_fd,
                shared_info,
                thread_handle,
            }
    }

    pub fn read_shared_info(&self) -> std::sync::RwLockReadGuard<vmm::vmm_config::instance_info::InstanceInfo> {
        self.shared_info.read().expect("shared_info lock poinsoned")
    }

    pub fn join(self) {
        self.thread_handle.join().expect("failed to join vmm thread")
    }

    pub fn set_configuration(&mut self, machine_config: VmConfig) -> Result<VmmData, VmmActionError> {
        let (sync_sender, sync_receiver) = oneshot::channel();
        let req = VmmAction::SetVmConfiguration(machine_config, sync_sender);
        self.sender.send(Box::new(req)).map_err(|_| ()).expect("Couldn't set configuration");
        self.event_fd.write(1).map_err(|_| ()).expect("Failed to signal");
        sync_receiver.map(|i| {
            i
        }).wait().expect("set config")
    }

    pub fn get_configuration(&mut self) -> Result<VmmData, VmmActionError> {
        let (sync_sender, sync_receiver) = oneshot::channel();
        let req = VmmAction::GetVmConfiguration(sync_sender);
        self.sender.send(Box::new(req)).map_err(|_| ()).expect("Couldn't send");
        self.event_fd.write(1).map_err(|_| ()).expect("Failed to signal");
        sync_receiver.wait().unwrap()
    }

    pub fn set_boot_source(&mut self, config: BootSourceConfig) -> Result<VmmData, VmmActionError> {
        let (sync_sender, sync_receiver) = oneshot::channel();
        let req = VmmAction::ConfigureBootSource(config, sync_sender);
        self.sender.send(Box::new(req)).map_err(|_| ()).expect("Couldn't send");
        self.event_fd.write(1).map_err(|_| ()).expect("Failed to signal");
        sync_receiver.wait().unwrap()
    }

    pub fn insert_block_device(&mut self, config: BlockDeviceConfig) -> Result<VmmData, VmmActionError> {
        let (sync_sender, sync_receiver) = oneshot::channel();
        let req = VmmAction::InsertBlockDevice(config, sync_sender);
        self.sender.send(Box::new(req)).map_err(|_| ()).expect("Couldn't send");
        self.event_fd.write(1).map_err(|_| ()).expect("Failed to signal");
        sync_receiver.wait().unwrap()
    }

    pub fn insert_net_device(&mut self, config: NetworkInterfaceConfig) -> Result<VmmData, VmmActionError> {
        let (sync_sender, sync_receiver) = oneshot::channel();
        let req = VmmAction::InsertNetworkDevice(config, sync_sender);
        self.sender.send(Box::new(req)).map_err(|_| ()).expect("Couldn't send");
        self.event_fd.write(1).map_err(|_| ()).expect("Failed to signal");
        sync_receiver.wait().unwrap()
    }

    pub fn add_vsock(&mut self, config: VsockDeviceConfig) -> Result<VmmData, VmmActionError> {
        let (sync_sender, sync_receiver) = oneshot::channel();
        let req = VmmAction::InsertVsockDevice(config, sync_sender);
        self.sender.send(Box::new(req)).map_err(|_| ()).expect("Couldn't send");
        self.event_fd.write(1).map_err(|_| ()).expect("Failed to signal");
        sync_receiver.wait().unwrap()
    }


    pub fn start_instance(&mut self) -> Result<VmmData, VmmActionError> {
        let (sync_sender, sync_receiver) = oneshot::channel();
        let req = VmmAction::StartMicroVm(sync_sender);
        self.sender.send(Box::new(req)).map_err(|_| ()).expect("Couldn't send");
        self.event_fd.write(1).map_err(|_| ()).expect("Failed to signal");
        sync_receiver.wait().unwrap()
    }
}
