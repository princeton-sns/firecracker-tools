use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{channel, Sender};
use std::thread::JoinHandle;

use futures::Future;
use futures::sync::oneshot;
use vmm::{VmmAction, VmmActionError, VmmData};
use vmm::vmm_config::instance_info::{InstanceInfo, InstanceState};
use vmm::vmm_config::boot_source::BootSourceConfig;
use vmm::vmm_config::drive::BlockDeviceConfig;
use vmm::vmm_config::vsock::VsockDeviceConfig;
use sys_util::EventFd;

pub struct VmmWrapper {
    sender: Sender<Box<VmmAction>>,
    event_fd: Rc<EventFd>,
    shared_info: Arc<RwLock<InstanceInfo>>,
    thread_handle: JoinHandle<()>,
}

impl VmmWrapper {

    pub fn new(instance_id: String, seccomp_level: u32) -> VmmWrapper {
            let (sender, receiver) = channel();
            let event_fd = Rc::new(EventFd::new().expect("Cannot create EventFd"));

            let shared_info = Arc::new(RwLock::new(InstanceInfo {
                state: InstanceState::Uninitialized,
                id: instance_id,
                vmm_version: "0.1".to_string(),
            }));

            let thread_handle =
                vmm::start_vmm_thread(shared_info.clone(), event_fd.try_clone().expect("Couldn't clone event_fd"), receiver, seccomp_level);

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
