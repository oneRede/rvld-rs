VERSION = 0.1.0
COMMIT_ID = $(shell git rev-list -1 HEAD)
TESTS := $(wildcard tests/*.sh)

build:
	@cargo build
	@ln -sf target/debug/rvld-rs ld

test: build
	@CC="riscv64-linux-gnu-gcc" \
	$(MAKE) $(TESTS)
	@printf '\e[32mPassed all tests\e[0m\n'

$(TESTS):
	@echo 'Testing' $@
	@./$@
	@printf '\e[32mOK\e[0m\n'

clean:
	cargo clean
	rm -rf out/
	rm -rf ld

.PHONY: build clean test $(TESTS)