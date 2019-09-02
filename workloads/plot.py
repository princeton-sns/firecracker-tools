import json
import sys
import numpy as np
import matplotlib.pyplot as plt

def list_to_tuple_list(l):
    if len(l) == 0:
        return l

    if len(l) % 2 == 1:
        sys.exit("list has odd number of elements")

    return [(l[i], l[i+1]) for i in range(0,len(l),2)]

def in_tuple_range(tsp, tuple_range):
    return tsp>=tuple_range[0] and tsp<=tuple_range[1]

class VM(object):
    def __init__(self, id, boot_tsp, req_res_tsp, evict_tsp, resource):
        self.id = id
        self.boot = list_to_tuple_list(boot_tsp)
        self.req_res = list_to_tuple_list(req_res_tsp)
        self.evict = list_to_tuple_list(evict_tsp)
        self.stage = 0 
        self.req_res_idx = 0
        self.resource = resource

    def __str__(self):
        return "id: " + str(self.id) + " boot: " + str(self.boot) + \
                " req_res: " + str(self.req_res) + " evict: " + str(self.evict)

    def is_running(self, tsp):

        for req_res_tuple in self.req_res:
            if in_tuple_range(tsp, req_res_tuple):
                return self.resource

        return 0



measurement_file = open(sys.argv[1], 'r')
data = json.load(measurement_file)

window_size = 8000000 #ns
start_time = data['start time']
end_time = data['end time']
num_vm = len(data['boot timestamps'])

vms = []
for vm_id in range(3, 3+num_vm,1):
    boot_tsp = data['boot timestamps'][str(vm_id)]
    req_res_tsp = data['request/response timestamps'][str(vm_id)]
    try:
        evict_tsp = data['eviction timestamps'][str(vm_id)]
    except:
        evict_tsp = []

    vm = VM(vm_id, boot_tsp, req_res_tsp, evict_tsp, 1)
    vms.append(vm)

#for vm in vms:
#    print(vm)

resource_limit = data['total cpu'] # the maximum number of 128MB VMs that the cluster can support

print("experiment lasted " + str((end_time - start_time) / 1000000) +"ms")
print("booted a total of " + str(num_vm) + " VMs")


count_running = []
ws = start_time + window_size / 2 # sampling tick
i = 0
while ws < end_time:
    running = 0
    for vm in vms:
        running = running + vm.is_running(ws)

    count_running.append(running)

    ws = ws + window_size

utilization = np.array(count_running)
utilization = utilization/resource_limit*100



# plot
x = np.linspace(0, (end_time - start_time )/1000000, len(utilization)  )

fig = plt.figure()
fig.set_size_inches(8,5)
plt.plot(x, utilization)
plt.xlabel('time(ms)')
plt.ylabel('Utilization')
plt.title('Utilization')
plt.legend()

plt.show()
