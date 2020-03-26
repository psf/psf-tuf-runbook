.PHONY: all
all: bin/yubihsm-provision

.PHONY: dist
dist:
	@echo TODO

bin/yubihsm-provision: bin
	$(MAKE) -C yubihsm-provision
	cp yubihsm-provision/target/release/yubihsm-provision ./bin

bin:
	mkdir -p bin
