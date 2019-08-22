pub struct Metrics {
    num_drop: u32,  // number of dropped requests
    num_complete: u32,  // number of requests completed
}

impl Metrics {
    pub fn new() -> Metrics {
        Metrics {
            num_drop: 0,
            num_complete: 0,
        }
    }

    pub fn drop_req(&mut self, num: u32) {
        self.num_drop = self.num_drop + num;
    }

    pub fn complete_req(&mut self, num: u32) {
        self.num_complete = self.num_complete + num;
    }
}
