"""
This is a wrapper which after bringing up the python runtime
signals Firecracker VMM to allow a snapshot to be taken if
requested.
Then it mounts application file system at /my-app,
imports application as a module named trigger,
and executes application by calling its main()
"""
from importlib import import_module
import sys
import subprocess
from portio import outl
import os

# snapshot signal
for i in range(1, os.cpu_count()):
    subprocess.Popen('taskset -c %d /srv/ts 124 1008'%(i), shell=True)
subprocess.Popen('taskset -c 0 /srv/ts 124 1008', shell=True)

try:
    subprocess.check_call('mount /dev/vdb /my-app', shell=True)
except subprocess.CalledProcessError:
    outl(125, 0x03f0)
outl(126, 0x03f0)

sys.path.append('/my-app')
APP = import_module('trigger')
sys.path.pop()
APP.main()
