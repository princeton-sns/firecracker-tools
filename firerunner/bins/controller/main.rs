#[macro_use(crate_version, crate_authors)]
extern crate clap;
extern crate firerunner;
extern crate serde;
extern crate serde_json;
extern crate vmm;
extern crate nix;
extern crate cgroups;

use std::io::BufRead;

use clap::{App, Arg};

mod config;
mod controller;
mod request;
mod listener;
mod cluster;

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
        .get_matches();

    let kernel = cmd_arguments.value_of("kernel").unwrap().to_string();
    let cmd_line = cmd_arguments.value_of("command line").unwrap().to_string();
    let requests_file = std::fs::File::open(cmd_arguments.value_of("requests file").unwrap())
        .expect("Request file not found");
    let runtimefs_dir = cmd_arguments.value_of("runtimefs dir").unwrap();
    let appfs_dir = cmd_arguments.value_of("appfs dir").unwrap();
    let func_config = std::fs::File::open(cmd_arguments.value_of("function config file").unwrap())
        .expect("Function config file not found");

    // We disable seccomp filtering when testing, because when running the test_gnutests
    // integration test from test_unittests.py, an invalid syscall is issued, and we crash
    // otherwise.
    let seccomp_level = 0;

    // init config
    let app_configs = config::Configuration::new(runtimefs_dir, appfs_dir, func_config);
    println!("{} functions loaded", app_configs.num_func());

    let mut controller = controller::Controller::new(app_configs, seccomp_level, cmd_line, kernel);
    println!("{:?}", controller.get_cluster_info());

    controller.ignite();

    for line in std::io::BufReader::new(requests_file).lines().map(|l| l.unwrap()) {
        match request::parse_json(line) {
            Ok(req) => controller.schedule(req),
            Err(e) => panic!("{:?}", e)
        }
    }

    controller.kill_all();
}

