CARGO ?= cargo
DOCKER ?= docker

all: lint

lint:
	$(CARGO) clippy --all-features

doc:
	$(CARGO) doc --all-features --workspace --no-deps

test:
	$(CARGO) t --workspace -- --nocapture

tarpaulin:
	$(CARGO) tarpaulin -v --all-features --ignore-tests --timeout 3600 \
		-o html

example:
	$(CARGO) r --release --example print_all

ctr-example:
	$(DOCKER) build --progress=plain --no-cache --pull \
		-f Dockerfile_example -t ckatsak/hwloc:example .
	$(DOCKER) run --rm -it ckatsak/hwloc:example

clean:
	$(CARGO) clean

distclean: clean
	-$(DOCKER) rmi ckatsak/hwloc:example

.PHONY: all lint doc test tarpaulin example ctr-example clean distclean

