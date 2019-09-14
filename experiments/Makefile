BASE_DIR ?= ..

KERNEL ?= $(BASE_DIR)/store/vmlinux
SUDO ?= sudo

MEMSIZE ?= 1024

RUNTIMES ?= nodejs python2
RUNTIME_FSS = $(patsubst %,$(BASE_DIR)/store/runtimes/%.ext4,$(RUNTIMES))

APPS ?= markdown-to-html
APP_FSS = $(patsubst %,$(BASE_DIR)/store/apps/%.ext4,$(APPS))

CONTROLLER = $(BASE_DIR)/firerunner/target/release/controller

ifeq ($(MODE),snapshot)
	MODE=snapshot
	SNAPSHOT_ARG=--snapshot
else
	MODE=nosnapshot
endif

check_defined = \
		    $(strip $(foreach 1,$1, \
		            $(call __check_defined,$1,$(strip $(value 2)))))
__check_defined = \
		      $(if $(value $1),, \
		            $(error Undefined $1$(if $2, ($2))))


$(call check_defined, EXPERIMENT)

$(KERNEL):
	@echo Fetching kernel image
	@mkdir -p $(BASE_DIR)/store
	@wget -O $@ https://praxis.cs.princeton.edu/snapfaas/vmlinux

$(CONTROLLER): FORCE
	cargo build --release --manifest-path $(BASE_DIR)/firerunner/Cargo.toml --bin controller

$(BASE_DIR)/store/apps/%.ext4:
	@echo Fetching app "$*"
	@mkdir -p $(BASE_DIR)/store/apps
	@wget --show-progress -q -O $@ https://praxis.cs.princeton.edu/snapfaas/apps/$(@F)

$(BASE_DIR)/store/runtimes/%.ext4:
	@echo Fetching app "$*"
	@mkdir -p $(BASE_DIR)/store/runtimes
	@wget --show-progress -q -O $@ https://praxis.cs.princeton.edu/snapfaas/runtimes/$(@F)

$(BASE_DIR)/experiments/$(EXPERIMENT)/workload.json: $(BASE_DIR)/experiments/$(EXPERIMENT)/workload.yaml
	@echo Generating workload
	python $(BASE_DIR)/workloads/generator.py $^ > $@

run: FORCE $(CONTROLLER) $(KERNEL) $(APP_FSS) $(RUNTIME_FSS) $(BASE_DIR)/experiments/$(EXPERIMENT)/workload.json
	@echo Running $(EXPERIMENT) experiment...
	@echo Creating "firecracker" cgroup
	@$(SUDO) sudo cgcreate -g cpu,cpuset:/firecracker
	@mkdir -p $(BASE_DIR)/experiments/$(EXPERIMENT)/results
	@touch $(BASE_DIR)/experiments/$(EXPERIMENT)/results/$(MEMSIZE)-$(MODE).json
	@$(SUDO) $(CONTROLLER) -k $(KERNEL) --runtimefs_dir $(BASE_DIR)/store/runtimes --appfs_dir $(BASE_DIR)/store/apps \
		--fconfig $(BASE_DIR)/experiments/$(EXPERIMENT)/function_config.yaml --requests $(BASE_DIR)/experiments/$(EXPERIMENT)/workload.json \
		--output $(BASE_DIR)/experiments/$(EXPERIMENT)/results/$(MEMSIZE)-$(MODE).json $(SNAPSHOT_ARG) --mem_size $(MEMSIZE)

FORCE:

