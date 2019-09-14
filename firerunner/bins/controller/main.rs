#[macro_use(crate_version, crate_authors)]
extern crate clap;
extern crate firerunner;
extern crate serde;
extern crate serde_json;
extern crate vmm;
extern crate nix;
extern crate cgroups;
extern crate time;
extern crate indicatif;

use std::io::BufRead;
use serde_json::json;

use clap::{App, Arg};
use std::fs::File;
use std::error::Error;
use std::io::{Write, Seek, SeekFrom};

use indicatif::ProgressBar;

mod config;
mod controller;
mod request;
mod listener;
mod cluster;
mod metrics;

fn main() {
    let cmd_arguments = App::new("controller")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Controller for serverless runtime based on Firecracker")
        .arg(
            Arg::with_name("kernel")
                .short("k")
                .long("kernel")
                .value_name("KERNEL")
                .takes_value(true)
                .required(true)
                .help("Path the the kernel binary")
        )
        .arg(
            Arg::with_name("command line")
                .short("c")
                .long("cmd_line")
                .value_name("CMD_LINE")
                .takes_value(true)
                .required(false)
                .default_value("quiet console=none reboot=k panic=1 pci=off")
                .help("Command line to pass to the kernel")
        )
        .arg(
            Arg::with_name("requests file")
                .short("r")
                .long("requests")
                .value_name("REQUEST_FILE")
                .takes_value(true)
                .required(true)
                .help("File containing JSON-lines with requests")
        )
        .arg(
            Arg::with_name("function config file")
                .short("f")
                .long("fconfig")
                .value_name("FUNCTION_CONFIG_FILE")
                .takes_value(true)
                .required(true)
                .help("YAML file defining functions configurations")
        )
        .arg(
            Arg::with_name("runtimefs dir")
                .long("runtimefs_dir")
                .value_name("RUNTIMEFS_DIR")
                .takes_value(true)
                .required(true)
                .help("Directory containing all runtimefs images")
        )
        .arg(
            Arg::with_name("appfs dir")
                .long("appfs_dir")
                .value_name("APPFS_DIR")
                .takes_value(true)
                .required(true)
                .help("Directory containing all appfs images")
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .takes_value(false)
                .required(false)
                .help("Whether VMs get to write to stdout")
        )
        .arg(
            Arg::with_name("snapshot dir")
                .long("snapshot_dir")
                .value_name("SNAPSHOTS_DIR")
                .takes_value(true)
                .required(false)
                .help("Boot VMs from snapshots in this directory")
        )
        .arg(
            Arg::with_name("total memory capacity")
                .long("mem_size")
                .value_name("MEM_SIZE")
                .takes_value(true)
                .required(false)
                .default_value("0")
                .help("Total memory capacity of the cluster")
        )
        .arg(
            Arg::with_name("output path")
                .long("output")
                .value_name("o")
                .takes_value(true)
                .required(false)
                .help("JSON file to store output metrics (outputs to STDOUT by default)")
        )
        .get_matches();

    let kernel = cmd_arguments.value_of("kernel").unwrap().to_string();
    let cmd_line = cmd_arguments.value_of("command line").unwrap().to_string();
    let mut requests_file = File::open(cmd_arguments.value_of("requests file").unwrap())
        .expect("Request file not found");
    let runtimefs_dir = cmd_arguments.value_of("runtimefs dir").unwrap();
    let appfs_dir = cmd_arguments.value_of("appfs dir").unwrap();
    let func_config = File::open(cmd_arguments.value_of("function config file").unwrap())
        .expect("Function config file not found");
    let debug = cmd_arguments.is_present("debug");
    let snapshots = cmd_arguments.value_of("snapshot dir").map(|sd| [sd].iter().collect());
    let mem_size: usize = cmd_arguments.value_of("total memory capacity").unwrap()
                                       .parse::<usize>().unwrap();
    let output_file = cmd_arguments.value_of("output path")
        .map(|fname| Box::new(File::create(fname).expect("Could not create output file")) as Box<Write>).unwrap_or(Box::new(std::io::stdout()));

    // We disable seccomp filtering when testing, because when running the test_gnutests
    // integration test from test_unittests.py, an invalid syscall is issued, and we crash
    // otherwise.
    let seccomp_level = 0;

    // init config
    // Current implementation assumes that function config do not change after controller
    // starts. That is, no live updates of function configs or adding functions.
    let app_configs = config::Configuration::new(runtimefs_dir, appfs_dir, func_config);
    println!("{} functions loaded", app_configs.num_func());

    let mut controller = controller::Controller::new(app_configs.clone(),
                                                     seccomp_level,
                                                     cmd_line,
                                                     kernel,
                                                     debug,
                                                     snapshots.clone(),
                                                     mem_size);
    println!("{:?}", controller.get_cluster_info());

    controller.ignite();

    std::thread::sleep(std::time::Duration::from_secs(1));

    let mut request_schedule_latency: Vec<u64> = Vec::new();

    let workload_start = time::precise_time_ns();

    let num_requests = std::io::BufReader::new(&mut requests_file).lines().count();

    requests_file.seek(SeekFrom::Start(0)).expect("Couldn't seek to start of request file");

    let progress = ProgressBar::new(num_requests as u64);
    for line in std::io::BufReader::new(requests_file).lines().map(|l| l.unwrap()) {
        match request::parse_json(line) {
            Ok(req) => {
                // Check function existence at the gateway
                if !app_configs.exist(&req.function){
//                    println!("function {} doesn't exist", &req.function);
                    continue;
                }

                let timestamp = req.timestamp;
                let left = (timestamp * 1000000).checked_sub(time::precise_time_ns() - workload_start).unwrap_or(0);
                std::thread::sleep(std::time::Duration::from_nanos(left));

                let t1 = time::precise_time_ns();
                controller.schedule(req);
                let t2 = time::precise_time_ns();
                request_schedule_latency.push(t2-t1);
                progress.inc(1);
            },
            Err(e) => panic!("Invalid request: {:?}", e)
        }
    }

    while controller.check_running() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    let workload_end = time::precise_time_ns();

    let total_time = (workload_end - workload_start) / 1_000_000; // in ms
    let num_complete = controller.get_stat().num_complete;
    let num_drop_resource = controller.get_stat().num_drop_resource;
    let num_drop_concurrency = controller.get_stat().num_drop_concurrency;
    let num_vm = controller.get_stat().boot_timestamp.len();
    let num_evict = controller.get_stat().eviction_timestamp.len();
    let throughput = (num_complete as f32) / (total_time as f32 / 1000f32);

    // Output time measurement as a json string
    let res = json!({
        "snapshot": snapshots.is_some(),
        "total cpu": controller.get_cluster_info().total_cpu,
        "total mem": controller.get_cluster_info().total_mem,
        "app config file": cmd_arguments.value_of("function config file").unwrap(),
        "requests file": cmd_arguments.value_of("requests file").unwrap(),
        "start time": workload_start,
        "end time": workload_end,
        "boot timestamps": controller.get_stat().boot_timestamp,
        "request/response timestamps": controller.get_stat().request_response_timestamp,
        "eviction timestamps": controller.get_stat().eviction_timestamp,
        "vm mem sizes": controller.get_stat().vm_mem_size,
        "drop requests (resource)": num_drop_resource,
        "drop requests (concurrency)": num_drop_concurrency,
        "number of evictions": num_evict,
        "number of vms booted": num_vm,
        "number of completed requests": num_complete,
        "cumulative throughput": throughput,
        "request schedule latency": request_schedule_latency
    });


    if let Err(e) = serde_json::to_writer_pretty(output_file, &res) {
        panic!("failed to write measurement results as json: {}", e.description());
    }

    controller.kill_all();
}

