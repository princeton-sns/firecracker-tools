#[macro_use(crate_version, crate_authors)]
extern crate clap;
extern crate firerunner;
extern crate serde;
extern crate serde_json;

use std::io::BufRead;
use std::collections::btree_map::{BTreeMap, Entry};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};

use clap::{App, Arg};

use firerunner::runner::{VmApp, VmAppConfig};

mod config;
mod request;
mod listener;

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
        .get_matches();

    let kernel = cmd_arguments.value_of("kernel").unwrap().to_string();
    let cmd_line = cmd_arguments.value_of("command line").unwrap().to_string();
    let requests_file = std::fs::File::open(cmd_arguments.value_of("requests file").unwrap()).unwrap();

    // We disable seccomp filtering when testing, because when running the test_gnutests
    // integration test from test_unittests.py, an invalid syscall is issued, and we crash
    // otherwise.
    let seccomp_level = 0;

    // init config
    let mut app_configs = config::Configuration::new("../images/", "../images");
    app_configs.insert(config::lorem_js());
    app_configs.insert(config::lorem_py2());

    let active_functions: Arc<Mutex<BTreeMap<String, (Sender<request::Request>, usize, VmApp)>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let warm_functions: Arc<Mutex<BTreeMap<String, (Sender<request::Request>, VmApp)>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let channels = Arc::new(Mutex::new(BTreeMap::new()));
    let (resp_sender, resp_receiver) = channel();
    let mut max_channel: u32 = 3;

    let manager_handle = listener::RequestManager::new(channels.clone()).spawn();
    let response_handle = {
        let active = active_functions.clone();
        let warm = warm_functions.clone();
        std::thread::spawn(move || {
            for (function, response) in resp_receiver.iter() {
                println!("{}: {}", function, String::from_utf8(response).unwrap());
                let mut a = active.lock().unwrap();
                let mut w = warm.lock().unwrap();
                let (sender, mut outstanding, app) = a.remove(&function).expect("active function not in active_functions?");
                outstanding -= 1;
                if outstanding > 0 {
                    a.insert(function, (sender, outstanding, app));
                } else {
                    w.insert(function, (sender, app));
                }
                println!("Warm {}, Active: {}", w.len(), a.len());
            }
        })
    };

    for line in std::io::BufReader::new(requests_file).lines().map(|l| l.unwrap()) {
        match request::parse_json(line) {
            Ok(req) => {
                match active_functions.lock().unwrap().entry(req.function.clone()) {
                    Entry::Occupied(mut entry) => {
                        let (sender, outstanding, _app) = entry.get_mut();
                        *outstanding += 1;
                        sender.send(req).expect("sending request");
                    },
                    Entry::Vacant(entry) => {
                        if let Some((sender, app)) = warm_functions.lock().unwrap().remove(entry.key()) {
                            sender.send(req).expect("sending request");
                            entry.insert((sender, 1, app));
                        } else if let Some(config) = app_configs.get(entry.key()) {
                            let cid = max_channel;
                            let (req_sender, req_receiver) = channel();
                            channels.lock().expect("poisoned lock").insert(cid, (entry.key().clone(), req_receiver, resp_sender.clone()));
                            max_channel += 1;
                            let app = VmAppConfig {
                                kernel: kernel.clone(),
                                instance_id: String::from("lambda"),
                                rootfs: config.runtimefs,
                                appfs: Some(config.appfs),
                                cmd_line: cmd_line.clone(),
                                seccomp_level,
                                vsock_cid: cid,
                            }.run();

                            req_sender.send(req).expect("sending request");
                            entry.insert((req_sender, 1, app));
                        } else {
                            panic!("Bad function name {}", entry.key());
                        }

                    },
                }
            },
            Err(e) => panic!("{:?}", e)
        }
    }

    drop(resp_sender);

    manager_handle.join().expect("Joining manager thread");
    response_handle.join().expect("Joining response handler thread");

}

