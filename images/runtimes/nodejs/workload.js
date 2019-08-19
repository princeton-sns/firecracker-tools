const { execSync, spawn } = require("child_process");
const process = require("process");
const readline = require("readline");
const fs = require("fs");

// for snapshot
execSync("outl 124 0x3f0");

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
  let req = JSON.parse(line);
  app.handle(req, function(resp) {
    let respJS = JSON.stringify(resp);
    process.stdout.write(respJS, "utf8");
    process.stdout.write('\n');
  });
});
