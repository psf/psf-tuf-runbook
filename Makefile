.PHONY: all
all: bin/yubihsm-provision bin/nitrohsm-provision

.PHONY: clean
clean:
	rm -rf bin/yubihsm-provision bin/nitrohsm-provision

bin/yubihsm-provision: bin
	cp yubihsm-provision/target/release/yubihsm-provision ./bin

bin/nitrohsm-provision: bin
	cp nitrohsm-provision/target/release/nitrohsm-provision ./bin

bin:
	mkdir -p bin
