# Project variables.

REGISTRY ?= ghcr.io
USERNAME ?= cosi-spec
TAG ?= $(shell git describe --tag --always --dirty)
IMAGE := $(REGISTRY)/$(USERNAME)/engine:$(TAG)

# Build variables.

BUILD := docker buildx build
PLATFORM ?= linux/amd64
PROGRESS ?= auto
PUSH ?= false
ARG_RUST_IMAGE ?= docker.io/library/rust:1.50.0-alpine3.13
ARG_PROTO_FILE ?= https://raw.githubusercontent.com/andrewrynhard/specification/87d89fabe2be5cb818e652181749e418afe7a454/spec.proto
COMMON_ARGS := --file=Dockerfile
COMMON_ARGS += --progress=$(PROGRESS)
COMMON_ARGS += --platform=$(PLATFORM)
COMMON_ARGS += --push=$(PUSH)
COMMON_ARGS += --build-arg=RUST_IMAGE=$(ARG_RUST_IMAGE)
COMMON_ARGS += --build-arg=PROTO_FILE=$(ARG_PROTO_FILE)

# Misc.

# Note: this list should be present in .gitignore.
ARTIFACTS := binaries proto src/spec target

all: lint test artifacts image ## Runs the complete set of targets.

lint: ## Lints the source code.
	$(BUILD) $(COMMON_ARGS) --target=$@ .

test: ## Runs tests.
	$(BUILD) $(COMMON_ARGS) --target=$@ .

artifacts: ## Builds the engine, runtime,  client, plugins, and generated code.
	$(BUILD) $(COMMON_ARGS) --output=type=local,dest=. --target=$@ .

image: ## Builds the engine image.
	$(BUILD) $(COMMON_ARGS) --output=type=image,name=$(IMAGE) --target=$@ .

define HELP_MENU_HEADER
\033[0;31mGetting Started\033[0m

To build this project, you must have the following installed:

- git
- make
- docker (19.03 or higher)
- buildx (https://github.com/docker/buildx)

The build process makes use of features not currently supported by the default
builder instance (docker driver). To create a compatible builder instance, run:

```
docker buildx create --driver docker-container --name local --use
```

If you already have a compatible builder instance, you may use that instead.

The artifacts (i.e. $(ARTIFACTS)) will be output to the root of the project.

endef

export HELP_MENU_HEADER

help: ## This help menu.
	@echo -e "$$HELP_MENU_HEADER"
	@grep -E '^[a-zA-Z%_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

run: artifacts ## Builds and runs the project.
	docker run --rm -it -v $(PWD)/binaries:/system -v $(PWD)/examples:/examples --entrypoint=/system/engine --name cosi busybox

example-%: ## Applies the supplied example resource in the container created by the `run` target.
	docker exec -it cosi /system/client /examples/$*.yaml

cargo-%: ## Executes cargo commands.
	docker run --rm -v $(PWD):/src -w /src --entrypoint=/usr/local/cargo/bin/cargo $(ARG_RUST_IMAGE) $*

clean: ## Removes the asset directory.
	-rm -rfv $(ARTIFACTS)
