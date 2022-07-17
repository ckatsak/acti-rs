#
# Root Makefile
#
# ckatsak, Wed 06 Jul 2022 09:16:46 PM EEST
#
PROJECT_ROOT := $(strip $(dir $(abspath $(lastword $(MAKEFILE_LIST)))))

DOCKER ?= docker
CARGO ?= cargo
RUST_DEBIAN_IMG := rust:1.62-bullseye  # slim lacks libssl-dev

CRDGEN := $(PROJECT_ROOT)/target/release/crdgen
CRD_YAML_OUT := $(PROJECT_ROOT)/deploy/crds.yml


.PHONY: noop
noop:
	$(info Please specify a target.)


.PHONY: generate-yaml-crds crdgen
generate-yaml-crds: $(CRDGEN)
	$(CRDGEN) >$(CRD_YAML_OUT)

crdgen: $(CRDGEN)
$(CRDGEN):
	$(DOCKER) run \
		--rm \
		--user "$(shell id -u):$(shell id -g)" \
		--volume "$(PROJECT_ROOT):/src" \
		$(RUST_DEBIAN_IMG) \
		bash -c 'rustup component add rustfmt \
			&& cd /src/ \
			&& cargo build --release --bin crdgen \
			&& strip -s /src/target/release/crdgen'


.PHONY: image image-registrant
image: image-registrant
image-registrant:
	$(MAKE) -C $(PROJECT_ROOT)/deploy/registrant image

.PHONY: push-local push-local-registrant
push-local: push-local-registrant
push-local-registrant:
	$(MAKE) -C $(PROJECT_ROOT)/deploy/registrant push-local

.PHONY: push-public push-public-registrant
push-public: push-public-registrant
push-public-registrant:
	$(MAKE) -C $(PROJECT_ROOT)/deploy/registrant push-public


.PHONY: lint-local
lint-local:
	$(CARGO) clippy --all-features -- --D warnings


.PHONY: test-local tarpaulin
test-local:
	$(CARGO) test --workspace -- --nocapture
tarpaulin:
	$(CARGO) tarpaulin --all-features --ignore-tests --timeout 3600 -o html


.PHONY: docs-local
docs-local:
	$(CARGO) doc --all-features --workspace --no-deps


.PHONY: clean-local
clean-local:
	$(CARGO) clean
