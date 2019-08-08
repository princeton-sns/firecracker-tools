const { execSync, spawn } = require("child_process");

execSync("mount -r /dev/vdb /srv");

module.paths.push("/srv/node_modules");
const app = require("/srv/workload");

let child = spawn("/usr/bin/nc-vsock", ["0", "1234"]);

child.stdout.on("readable", () => {
  let nbytes;
  while (!nbytes) { nbytes = child.stdout.read(1); }
  if (!nbytes) {
    process.exit(0);
  }
  let body;
  while (!body) { body = child.stdout.read(nbytes[0]); }
  if (!body) {
    console.log("empty body", nbytes);
    process.exit(0);
  }
  let req = JSON.parse(body);
  app.handle(req, function(resp) {
    let respJS = JSON.stringify(resp);
    child.stdin.write(Buffer.from([respJS.length]));
    child.stdin.write(respJS, "utf8");
  });
});
