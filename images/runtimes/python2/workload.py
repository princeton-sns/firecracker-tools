#!/usr/bin/env python2

import sys
import os
import imp
import struct
import json

os.system("mount -r /dev/vdb /srv");

sys.path.append('/srv/package')
app = imp.load_source('app', '/srv/workload')

while True:
    jsonlen = struct.unpack("@B", sys.stdin.read(1))[0]
    request = json.loads(sys.stdin.read(jsonlen))

    response = app.handle(request)

    responseJson = json.dumps(response)

    sys.stderr.write(struct.pack("@B", len(responseJson)))
    sys.stderr.write(bytes(responseJson))
    sys.stderr.flush()

