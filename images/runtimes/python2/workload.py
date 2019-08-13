#!/usr/bin/env python2

import sys
import os
import imp
import struct
import json
from subprocess import Popen, PIPE

vsock = Popen('nc-vsock 0 1234', shell=True, stdout=PIPE, stdin=PIPE)
os.system("mount -r /dev/vdb /srv");

sys.path.append('/srv/package')
app = imp.load_source('app', '/srv/workload')

while True:
    jsonlen = struct.unpack("@B", vsock.stdout.read(1))[0]
    request = json.loads(vsock.stdout.read(jsonlen))

    response = app.handle(request)

    responseJson = json.dumps(response)

    vsock.stdin.write(struct.pack("@B", len(responseJson)))
    vsock.stdin.write(bytes(responseJson))
    vsock.stdin.flush()
