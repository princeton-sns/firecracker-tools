// This is a wrapper which after bringing up a Node runtime
// signals Firecracker VMM to allow a snapshot to be taken
// if requested.
// Then it mounts application file system at /my-app,
// imports and executes application

const child_process = require('child_process');
// This requires ts.node be placed in the same directory as this wrapper
const ts = require('./ts.node');
const os = require('os');
const vcpu_count = os.cpus().length;
for (let i = 1; i < vcpu_count; i++) {
    child_process.exec(`taskset -c ${i} /srv/ts 124 1008`);
}
child_process.exec(`taskset -c 0 /srv/ts 124 1008`);

// mount app fs
child_process.execSync('mount /dev/vdb /my-app');
// This portio signals VMM to record boot up time
// if we are booting from a snapshot
ts.ts(126, 0x03f0);

app = require('/my-app/trigger.js')
app()
