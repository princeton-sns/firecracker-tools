#[macro_use(crate_version, crate_authors)]
extern crate clap;
extern crate cgroups;
extern crate firerunner;

use std::io::{BufRead, Read, Write};
use std::path::PathBuf;
use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::time::Instant;

use clap::{App, Arg};

use firerunner::runner::VmAppConfig;

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
        .arg(
            Arg::with_name("load_dir")
                .long("load_from")
                .takes_value(true)
                .required(false)
                .help("if specified start VM from a snapshot under the given directory")
        )
        .arg(
            Arg::with_name("dump_dir")
                .long("dump_to")
                .takes_value(true)
                .required(false)
                .help("if specified creates a snapshot right after runtime is up under the given directory")
        )
        .arg(
            Arg::with_name("mem_size")
                 .long("mem_size")
                 .value_name("MEMSIZE")
                 .takes_value(true)
                 .required(false)
                 .help("Guest memory size in MB (default is 128)")
        )
        .arg(
            Arg::with_name("vcpu_count")
                 .long("vcpu_count")
                 .value_name("VCPUCOUNT")
                 .takes_value(true)
                 .required(false)
                 .help("Number of vcpus (default is 1)")
        )
        .arg(
            Arg::with_name("copy_base_memory")
                 .long("copy_base")
                 .value_name("COPYBASE")
                 .takes_value(false)
                 .required(false)
                 .help("Restore base snapshot memory by copying")
        )
        .arg(
            Arg::with_name("hugepage")
                 .long("hugepage")
                 .value_name("HUGEPAGE")
                 .takes_value(false)
                 .required(false)
                 .help("Use huge pages to back virtual machine memory")
        )
        .arg(
            Arg::with_name("diff_dirs")
                 .long("diff_dirs")
                 .value_name("DIFFDIRS")
                 .takes_value(true)
                 .required(false)
                 .help("Comma-separated list of diff snapshots")
        )
        .arg(
            Arg::with_name("copy_diff_memory")
                 .long("copy_diff")
                 .value_name("COPYDIFF")
                 .takes_value(false)
                 .required(false)
                 .help("If a diff snapshot is provided, restore its memory by copying")
        )
        .arg(
            Arg::with_name("network")
                 .long("network")
                 .value_name("NETWORK")
                 .takes_value(true)
                 .required(false)
                 .help("configure a network device for the VM with the provided MAC address")
        )
        .get_matches();

    let kernel = cmd_arguments.value_of("kernel").unwrap().to_string();
    let rootfs = cmd_arguments.value_of("rootfs").map(PathBuf::from).unwrap();
    let appfs = cmd_arguments.value_of("appfs").map(PathBuf::from);
    let cmd_line = cmd_arguments.value_of("command line").unwrap().to_string();
    let mem_size_mib = cmd_arguments.value_of("mem_size").map(|x| x.parse::<usize>().unwrap());
    let vcpu_count = cmd_arguments.value_of("vcpu_count").map(|x| x.parse::<u64>().unwrap());
    let load_dir = cmd_arguments.value_of("load_dir").map(PathBuf::from);
    let dump_dir = cmd_arguments.value_of("dump_dir").map(PathBuf::from);
    let diff_dirs = cmd_arguments.value_of("diff_dirs").map_or(Vec::new(), |x| x.split(',').collect::<Vec<&str>>()
        .iter().map(PathBuf::from).collect());
    let hugepage = cmd_arguments.is_present("hugepage");
    let copy_base = cmd_arguments.is_present("copy_base_memory");
    let copy_diff = cmd_arguments.is_present("copy_diff_memory");
    let network = cmd_arguments.value_of("network").map(|x| x.to_string());

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

    let (checker, notifier) = nix::unistd::pipe().expect("Could not create a pipe");

        let mut app = VmAppConfig {
            kernel: kernel.clone(),
            instance_id: instance_id.clone(),
            rootfs: rootfs.clone(),
            appfs: appfs.clone(),
            cmd_line: cmd_line.clone(),
            seccomp_level,
            vsock_cid: 42,
            notifier: unsafe{ File::from_raw_fd(notifier) },
            cpu_share: 1024,
            vcpu_count: vcpu_count.unwrap_or(1),
            mem_size_mib,
            load_dir: load_dir.clone(),
            dump_dir: dump_dir.clone(),
            diff_dirs,
            hugepage,
            copy_base,
            copy_diff,
            network,
        }.run(true);

        // We need to wait for the ready signal from Firecracker
        let data = &mut[0u8; 4usize];
        unsafe{ File::from_raw_fd(checker) }.read_exact(data).expect("Failed to receive ready signal");
        //println!("VM with notifier id {} is ready", u32::from_le_bytes(*data));

    let stdin = std::io::stdin();

    for mut line in stdin.lock().lines().map(|l| l.unwrap()) {
        let t1 = Instant::now();
        line.push('\n');
        app.connection.write_all(line.as_bytes()).expect("Failed to write to request pipe");
        let mut lens = [0; 4];
        app.connection.read_exact(&mut lens).expect("Failed to read response size");
        let len = u32::from_be_bytes(lens);
        let mut response = vec![0; len as usize];
        app.connection.read_exact(response.as_mut_slice()).expect("Failed to read response");
        let t2 = Instant::now();
        eprintln!("e2e {} us", t2.duration_since(t1).as_micros());
        println!("{}", String::from_utf8(response).unwrap());
    }
    app.kill();
}
