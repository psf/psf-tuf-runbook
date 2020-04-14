.PHONY: all
all: bin/yubihsm-provision bin/nitrohsm-provision

.PHONY: dist
dist:
	@echo TODO

.PHONY: clean
clean:
	rm -rf bin/yubihsm-provision bin/nitrohsm-provision

bin/yubihsm-provision: bin
	$(MAKE) -C yubihsm-provision
	cp yubihsm-provision/target/release/yubihsm-provision ./bin

bin/nitrohsm-provision: bin
	$(MAKE) -C nitrohsm-provision
	cp nitrohsm-provision/target/release/nitrohsm-provision ./bin

bin:
	mkdir -p bin
