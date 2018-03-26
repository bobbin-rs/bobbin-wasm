.PHONY: test test-dump install

BINDIR=$(HOME)/.cargo/bin
TEST_ARGS=--bindir $(BINDIR)

install:
	cargo install --force --debug

test: test-dump test-interp

test-dump:
	cat test-dump.txt | grep -v \# | xargs test/run-tests.py $(TEST_ARGS)

test-interp:
	cat test-interp.txt | grep -v \# | xargs test/run-tests.py $(TEST_ARGS)
