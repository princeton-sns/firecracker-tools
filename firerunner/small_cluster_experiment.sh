#!/bin/bash
kernel="/users/davidhl/Dev/serverless/snapfaas/firerunner/images/vmlinux-tty"
rt_dir="/users/davidhl/Dev/serverless/snapfaas/firerunner/images"
app_dir="/users/davidhl/Dev/serverless/snapfaas/firerunner/images"
function_config_file="/users/davidhl/Dev/serverless/snapfaas/firerunner/example_func_configs.yaml"
request_file="/users/davidhl/Dev/serverless/snapfaas/firerunner/bins/controller/example_workload.json"

mkdir -p measurements

for c in 1 2 3 4 5
do
	function_config_file="/users/davidhl/Dev/serverless/snapfaas/firerunner/thrput_func_config$c.yaml"
	echo "Concurrency limit: $c"
	echo "Function config file: $function_config_file"

	for inter_time in 3 2 1 0.5 0.25
	do
		request_file="/users/davidhl/Dev/serverless/snapfaas/workloads/thrput$inter_time.json"
		echo "Inter-arrival time is $inter_time times that of 17% workload"
		echo "Request file: $request_file"

		echo "No snapshot"
		for mem_size in 1024
		do
			echo "Cluster size: $mem_size"
			sudo RUST_BACKTRACE=full ~/Dev/serverless/snapfaas/firerunner/target/release/controller \
				-k $kernel \
				--runtimefs_dir $rt_dir \
				--appfs_dir $app_dir \
				-f $function_config_file \
				--requests "$request_file"  \
				--mem_size $mem_size \
				> "measurements/$mem_size-cluster-$c-concurrency-$inter_time-intertime-nosnapshot.log"

			tail -n 1 "measurements/$mem_size-cluster-$c-concurrency-$inter_time-intertime-nosnapshot.log"
			mv -f `tail -n 1 "measurements/$mem_size-cluster-$c-concurrency-$inter_time-intertime-nosnapshot.log" | cut -d ':' -f 2 | tr -d [:space:]` \
				"measurements/measurement-$mem_size-cluster-$c-concurrency-$inter_time-intertime-nosnapshot.log"

		done

		echo "Snapshot"
		for mem_size in 1024
		do
			echo "Cluster size: $mem_size"
			sudo RUST_BACKTRACE=full ~/Dev/serverless/snapfaas/firerunner/target/release/controller \
				-k $kernel \
				--runtimefs_dir $rt_dir \
				--appfs_dir $app_dir \
				-f $function_config_file \
				--requests "$request_file"  \
				--mem_size $mem_size --snapshot \
				> "measurements/$mem_size-cluster-$c-concurrency-$inter_time-intertime-snapshot.log"

			tail -n 1 "measurements/$mem_size-cluster-$c-concurrency-$inter_time-intertime-snapshot.log"
			mv -f `tail -n 1 "measurements/$mem_size-cluster-$c-concurrency-$inter_time-intertime-snapshot.log" | cut -d ':' -f 2 | tr -d [:space:]` \
				"measurements/measurement-$mem_size-cluster-$c-concurrency-$inter_time-intertime-snapshot.log"
		done

		echo ""

	done
done

: <<'END'
echo "No snapshot"
for mem_size in 768 1024 1280
do
	echo "Cluster size: $mem_size"
	sudo RUST_BACKTRACE=full ~/Dev/serverless/snapfaas/firerunner/target/release/controller \
		-k $kernel \
		--runtimefs_dir $rt_dir \
		--appfs_dir $app_dir \
		-f $function_config_file \
		--requests "$request_file"  \
		--mem_size $mem_size > "$mem_size-nosnapshot.log"

	tail -n 1 "$mem_size-nosnapshot.log"
	mv -f `tail -n 1 "$mem_size-nosnapshot.log" | cut -d ':' -f 2 | tr -d [:space:]` \
		"measurements/measurement-$mem_size-cluster-nosnapshot.json"

done

echo "Snapshot"
for mem_size in 768 1024 1280
do
	echo "Cluster size: $mem_size"
	sudo RUST_BACKTRACE=full ~/Dev/serverless/snapfaas/firerunner/target/release/controller \
		-k $kernel \
		--runtimefs_dir $rt_dir \
		--appfs_dir $app_dir \
		-f $function_config_file \
		--requests "$request_file"  \
		--mem_size $mem_size --snapshot > "$mem_size-snapshot.log"

	tail -n 1 "$mem_size-snapshot.log"
	mv -f `tail -n 1 "$mem_size-snapshot.log" | cut -d ':' -f 2 | tr -d [:space:]` \
		"measurements/measurement-$mem_size-cluster-snapshot.json"

done
END
