# Convenience wrapper around cargo. Run `make` (or `make help`) for the list.

.DEFAULT_GOAL := help
.PHONY: help build run test web serve deploy clean

PORT ?= 8000
# Where `make deploy` puts the browser version. Your AWS CLI credentials are
# used; pick a profile with AWS_PROFILE=..., e.g. `make deploy AWS_PROFILE=rock`.
WEB_BUCKET ?= s3://rock-do-id-au/web/static/

# The browser version is the same program built for WebAssembly: macroquad
# runs there as it is, loaded by its own small JavaScript loader (no
# wasm-bindgen). The loader is fetched once into web/ (commit it to pin it);
# delete it to fetch it again.
WASM_TARGET := wasm32-unknown-unknown
MQ_JS_BUNDLE_URL := https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js

help:
	@echo "rock — common tasks"
	@echo
	@echo "  make build    cargo build --release"
	@echo "  make run      cargo run --release — the game in a window"
	@echo "  make test     cargo test"
	@echo "  make web      build the browser version (WebAssembly) into web/dist"
	@echo "  make serve    make web, then serve it at http://localhost:$(PORT)"
	@echo "  make deploy   make web, then upload web/dist to $(WEB_BUCKET)"
	@echo "  make clean    remove the builds"

build:
	cargo build --release
run:
	cargo run --release
test:
	cargo test

web/mq_js_bundle.js:
	curl -fsSL -o $@ $(MQ_JS_BUNDLE_URL)

# Its own target directory, whatever CARGO_TARGET_DIR or ~/.cargo/config.toml
# say, so the wasm is where this looks for it.
web: web/mq_js_bundle.js
	@rustup target list --installed | grep -qx $(WASM_TARGET) || rustup target add $(WASM_TARGET)
	cargo build --release --target $(WASM_TARGET) --target-dir target
	rm -rf web/dist
	mkdir -p web/dist
	cp target/$(WASM_TARGET)/release/rock.wasm web/index.html web/mq_js_bundle.js web/dist/
	@echo "Built web/dist. Serve it (make serve) and open it in a browser; it won't load from file://."

serve: web
	@echo "Rock at http://localhost:$(PORT)  (Ctrl-C to stop)"
	cd web/dist && python3 -m http.server $(PORT)

# The files keep their names from one build to the next, so browsers are
# told to check for a newer copy each time (unchanged ones come back 304).
# The wasm goes up on its own as application/wasm, which browsers need to
# compile it as it streams in.
deploy: web
	@command -v aws >/dev/null || { echo "make deploy needs the AWS CLI: brew install awscli"; exit 1; }
	aws s3 sync web/dist/ $(WEB_BUCKET) --exclude "*.wasm" --cache-control no-cache
	for wasm in web/dist/*.wasm; do \
		aws s3 cp "$$wasm" "$(WEB_BUCKET)$$(basename "$$wasm")" \
			--content-type application/wasm --cache-control no-cache; \
	done
	@echo "Deployed web/dist to $(WEB_BUCKET)"

clean:
	cargo clean
	cargo clean --target-dir target
	rm -rf web/dist
