use std::collections::btree_map::BTreeMap;

pub struct Metrics {
    pub num_drop: u32,  // number of dropped requests
    pub num_complete: u32,  // number of requests completed
    pub boot_timestamp: BTreeMap<u32, Vec<u64>>, // key is vm_id, value is boot timestamp
    pub eviction_timestamp: BTreeMap<u32, Vec<u64>>,
    pub request_response_timestamp: BTreeMap<u32, Vec<u64>> // key is vm_id, value is request send time and response receive time
}

impl Metrics {
    pub fn new() -> Metrics {
        Metrics {
            num_drop: 0,
            num_complete: 0,
            boot_timestamp: Default::default(),
            eviction_timestamp: Default::default(),
            request_response_timestamp: Default::default(),
        }
    }

    pub fn drop_req(&mut self, num: u32) {
        self.num_drop = self.num_drop + num;
    }

    pub fn complete_req(&mut self, num: u32) {
        self.num_complete = self.num_complete + num;
    }

    pub fn log_boot_timestamp(&mut self, vm_id: u32, tsp: u64) {
        self.boot_timestamp.entry(vm_id).or_insert(Vec::new()).push(tsp);

    }

    pub fn log_request_timestamp(&mut self, vm_id: u32, tsp: u64) {
        self.request_response_timestamp.entry(vm_id).or_insert(Vec::new()).push(tsp);

    }

    pub fn log_eviction_timestamp(&mut self, vm_id: u32, tsp: u64) {
        self.eviction_timestamp.entry(vm_id).or_insert(Vec::new()).push(tsp);
    }
}
