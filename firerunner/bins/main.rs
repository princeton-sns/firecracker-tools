#[macro_use(crate_version, crate_authors)]
extern crate clap;
extern crate firerunner;

use std::io::{BufRead, Read, Write};

use clap::{App, Arg};

use firerunner::runner::VmAppConfig;
use firerunner::vsock::{self, VsockListener};

fn main() {
    let cmd_arguments = App::new("firecracker")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Launch a microvm.")
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
            Arg::with_name("rootfs")
                .long("r")
                .long("rootfs")
                .value_name("ROOTFS")
                .takes_value(true)
                .required(true)
                .help("Path to the root file system")
        )
        .arg(
            Arg::with_name("appfs")
                .long("a")
                .long("appfs")
                .value_name("APPFS")
                .takes_value(true)
                .required(false)
                .help("Path to the root file system")
        )
        .arg(
            Arg::with_name("id")
                .long("id")
                .help("MicroVM unique identifier")
                .default_value("abcde1234")
                .takes_value(true)
                ,
        )
        .arg(
            Arg::with_name("seccomp-level")
                .long("seccomp-level")
                .help(
                    "Level of seccomp filtering.\n
                            - Level 0: No filtering.\n
                            - Level 1: Seccomp filtering by syscall number.\n
                            - Level 2: Seccomp filtering by syscall number and argument values.\n
                        ",
                )
                .takes_value(true)
                .default_value("0")
                .possible_values(&["0", "1", "2"]),
        )
        .get_matches();

    let kernel = cmd_arguments.value_of("kernel").unwrap().to_string();
    let rootfs = cmd_arguments.value_of("rootfs").unwrap().to_string();
    let appfs = cmd_arguments.value_of("appfs").map(|s| s.to_string());
    let cmd_line = cmd_arguments.value_of("command line").unwrap().to_string();

    // It's safe to unwrap here because clap's been provided with a default value
    let instance_id = cmd_arguments.value_of("id").unwrap().to_string();

    // We disable seccomp filtering when testing, because when running the test_gnutests
    // integration test from test_unittests.py, an invalid syscall is issued, and we crash
    // otherwise.
    #[cfg(test)]
    let seccomp_level = seccomp::SECCOMP_LEVEL_NONE;
    #[cfg(not(test))]
    // It's safe to unwrap here because clap's been provided with a default value,
    // and allowed values are guaranteed to parse to u32.
    let seccomp_level = cmd_arguments
        .value_of("seccomp-level")
        .unwrap()
        .parse::<u32>()
        .unwrap();

    let app = VmAppConfig {
        kernel,
        instance_id,
        rootfs,
        appfs,
        cmd_line,
        seccomp_level,
        vsock_cid: 42,
    }.run();

    let mut listener = VsockListener::bind(vsock::VMADDR_CID_ANY, 1234).expect("vsock listen");
    if let Ok((mut connection, addr)) = listener.accept() {
        println!("Connection from {:?}", addr);
        std::thread::spawn(move || {
            fn handle_connection<C: Read + Write>(connection: &mut C, request: Vec<u8>) -> std::io::Result<Vec<u8>> {
                connection.write_all(&[request.len() as u8])?;
                connection.write_all(request.as_slice())?;
                let mut lens = [0];
                connection.read_exact(&mut lens)?;
                if lens[0] == 0 {
                    return Ok(vec![]);
                }
                let mut response = Vec::with_capacity(lens[0] as usize);
                response.resize(lens[0] as usize, 0);
                connection.read_exact(response.as_mut_slice())?;
                Ok(response)
            }

            let stdin = std::io::stdin();

            for line in stdin.lock().lines().map(|l| l.unwrap()) {
                if let Ok(response) = handle_connection(&mut connection, line.into_bytes()) {
                    println!("{}", String::from_utf8(response).unwrap());
                } else {
                    break;
                }
            }
            app.kill();
        }).join().expect("join");
    }
}
