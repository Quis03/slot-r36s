#!/bin/sh
# Build the R36S card tree into dist-r36s/ inside the arm64 build box.
# Usage: r36s/build.sh            build everything
#        r36s/build.sh test       run the test suite in the box instead
set -eu
cd "$(dirname "$0")/.."

# Colima users without Docker Desktop's credential helper.
if [ -z "${DOCKER_HOST:-}" ] && [ -S "$HOME/.colima/default/docker.sock" ]; then
	export DOCKER_HOST="unix://$HOME/.colima/default/docker.sock"
fi
if grep -q '"credsStore"' "${DOCKER_CONFIG:-$HOME/.docker}/config.json" 2>/dev/null \
	&& ! command -v docker-credential-desktop >/dev/null 2>&1; then
	export DOCKER_CONFIG="$PWD/target/docker-config"
	mkdir -p "$DOCKER_CONFIG"
	[ -f "$DOCKER_CONFIG/config.json" ] || echo '{}' > "$DOCKER_CONFIG/config.json"
fi

docker build -q --platform linux/arm64 -t slot-r36s-build r36s > /dev/null

box() {
	docker run --rm --platform linux/arm64 \
		-v "$PWD":/src -w /src \
		-v slot-cargo:/usr/local/cargo/registry \
		-v slot-rustup:/usr/local/rustup \
		-e SLOT_SILENT=1 \
		slot-r36s-build sh -c "$1"
}

if [ "${1:-}" = "test" ]; then
	box 'cargo test --target-dir target-linux --workspace --no-fail-fast'
	exit
fi

OUT=dist-r36s/slot
box "task dist:device OUT=$OUT"
cp -R r36s/card/System/. "$OUT/System/"
mkdir -p dist-r36s/tools
cp r36s/card/tools/*.sh dist-r36s/tools/
chmod +x "$OUT"/System/*.sh dist-r36s/tools/*.sh
echo "built dist-r36s: copy slot/ and tools/ to the root of EASYROMS"
