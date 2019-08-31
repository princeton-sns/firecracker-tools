const { execSync, exec } = require("child_process");
const process = require("process");
const readline = require("readline");
const fs = require("fs");

// for snapshot
// this approach relies on that we are currently being executed on cpu 0
// and that other cpus writes to the port before us
// since as of now snapshots are created offline, we are fine
const cpu_count = require("os").cpus().length;
for (var i = 0; i < cpu_count; i++) {
    exec(`taskset -c ${i} outl 124 0x3f0`);
}
execSync("taskset -c 0 outl 124 0x3f0");

execSync("mount -r /dev/vdb /srv");

module.paths.push("/srv/node_modules");
const app = require("/srv/workload");

rl = readline.createInterface({
    input: fs.createReadStream('/dev/ttyS1'),
    crlfDelay: Infinity
});
// signal Firerunner that we are ready to receive requests
execSync("outl 126 0x3f0");

rl.on('line', (line) => {
  var hrstart = process.hrtime()
  let req = JSON.parse(line);
  app.handle(req, function(resp) {
    var hrend = process.hrtime(hrstart)
    resp.runtime_sec = hrend[0];
    resp.runtime_ms = hrend[1] / 1000000;
    let respJS = JSON.stringify(resp);
    process.stdout.write(Buffer.from([respJS.length]));
    process.stdout.write(respJS, "utf8");
  });
});
