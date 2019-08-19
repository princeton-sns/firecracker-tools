#!/usr/bin/env python2

import sys
import os
import imp
import json
from subprocess import call, Popen, PIPE

# for snapshot
call('outl 124 0x3f0', shell=True)

os.system("mount -r /dev/vdb /srv")

sys.path.append('/srv/package')
app = imp.load_source('app', '/srv/workload')

with open('/dev/ttyS1', 'r') as tty:
    # signal firerunner we are ready
    call('outl 126 0x3f0', shell=True)
    while True:
        request = json.loads(tty.readline())

        response = app.handle(request)

        responseJson = json.dumps(response)

        sys.stdout.write(responseJson)
        sys.stdout.write('\n')
        sys.stdout.flush()
