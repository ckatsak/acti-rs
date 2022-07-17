#
# ckatsak, Wed 06 Jul 2022 03:58:53 AM EEST
#
OWNER ?= $(USER)
IMAGE_NAME := $(OWNER)/acti-registrant
IMAGE_TAG := $(shell sed -En \
		's/version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' \
		../../crates/registrant-rs/Cargo.toml | head -1)
PUBLIC_REGISTRY ?= docker.io
#LOCAL_REGISTRY ?= localhost:5000
LOCAL_REGISTRY ?= 192.168.44.1:4999
DOCKER ?= docker


.PHONY: image push-local push-public

image:
	$(DOCKER) build --progress=plain --no-cache --pull -f Dockerfile \
		-t $(IMAGE_NAME):$(IMAGE_TAG) ../..
	$(DOCKER) image prune --force --filter label=stage=builder

push-local:
	$(DOCKER) tag $(IMAGE_NAME):$(IMAGE_TAG) \
		$(LOCAL_REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)
	$(DOCKER) push $(LOCAL_REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)

push-public:
	$(DOCKER) tag $(IMAGE_NAME):$(IMAGE_TAG) \
		$(PUBLIC_REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)
	$(DOCKER) push $(PUBLIC_REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)

