use std::collections::BTreeMap;

pub struct Metrics {
    pub num_drop: u32,  // number of dropped requests
    pub num_complete: u32,  // number of requests completed
    pub boot_timestamp: BTreeMap<u32, u64>,
    pub req_e2e_latency: BTreeMap<u32, BTreeMap<u64, u64>> //
}

impl Metrics {
    pub fn new() -> Metrics {
        Metrics {
            num_drop: 0,
            num_complete: 0,
            boot_timestamp: Default::default(),
            req_e2e_latency: Default::default(),
        }
    }

    pub fn drop_req(&mut self, num: u32) {
        self.num_drop = self.num_drop + num;
    }

    pub fn complete_req(&mut self, num: u32) {
        self.num_complete = self.num_complete + num;
    }

    pub fn log_boot_timestamp(&mut self, vm_id: u32, btp: u64) {
        if let Some(_) = self.boot_timestamp.insert(vm_id, btp) {
            panic!("Booting the same vm (id: {}) twice", vm_id);
        }
    }
}
