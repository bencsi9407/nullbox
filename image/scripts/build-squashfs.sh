#!/usr/bin/env bash
#
# build-squashfs.sh — Build the NullBox SquashFS root filesystem
#
# Creates a read-only SquashFS image containing:
#   - /system/bin/  — all NullBox binaries (statically linked)
#   - /system/config/ — nulld.toml service configuration
#   - /etc/ — minimal system config (hostname, resolv.conf)
#   - Empty mount points for tmpfs, overlay, etc.
#
# Usage:
#   ./image/scripts/build-squashfs.sh [--target x86_64-unknown-linux-musl]

set -euo pipefail

NULLBOX_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TARGET="${TARGET:-x86_64-unknown-linux-musl}"
BUILD_DIR="${NULLBOX_ROOT}/build/squashfs-staging"
OUTPUT_DIR="${NULLBOX_ROOT}/build/output/squashfs"

echo "=== NullBox SquashFS Build ==="
echo "  Target: ${TARGET}"

# Check for mksquashfs
if ! command -v mksquashfs &>/dev/null; then
    echo "error: mksquashfs not found. Install: pacman -S squashfs-tools"
    exit 1
fi

mkdir -p "${OUTPUT_DIR}"

# Clean previous build
rm -rf "${BUILD_DIR:?}"

# Create filesystem layout (from ARCHITECTURE.md)
mkdir -p "${BUILD_DIR}"/{system/bin,system/config,etc,tmp,var,agent,vault,snapshots,proc,sys,dev,run}
mkdir -p "${BUILD_DIR}/dev"/{pts,shm}

# Copy binaries
RELEASE_DIR="${NULLBOX_ROOT}/target/${TARGET}/release"
FALLBACK_DIR="${NULLBOX_ROOT}/target/release"

copy_binary() {
    local name="$1"
    if [[ -f "${RELEASE_DIR}/${name}" ]]; then
        cp "${RELEASE_DIR}/${name}" "${BUILD_DIR}/system/bin/${name}"
        echo "  Copied ${name} (musl static)"
    elif [[ -f "${FALLBACK_DIR}/${name}" ]]; then
        cp "${FALLBACK_DIR}/${name}" "${BUILD_DIR}/system/bin/${name}"
        echo "  Copied ${name} (release — not static, dev only)"
    else
        echo "  WARNING: ${name} binary not found, skipping"
    fi
}

echo ">>> Copying binaries..."
copy_binary nulld
copy_binary nullctl

# For v0.1, cage/egress/ctxgraph are libraries linked into nulld or
# run as separate binaries. Create placeholder scripts for now.
# These will be replaced with actual binaries as we implement the daemons.
for svc in cage egress ctxgraph; do
    if [[ ! -f "${BUILD_DIR}/system/bin/${svc}" ]]; then
        cat > "${BUILD_DIR}/system/bin/${svc}" << EOF
#!/bin/sh
echo "${svc}: placeholder service running"
while true; do sleep 3600; done
EOF
        chmod +x "${BUILD_DIR}/system/bin/${svc}"
        echo "  Created ${svc} placeholder"
    fi
done

# Create nulld.toml service configuration
cat > "${BUILD_DIR}/system/config/nulld.toml" << 'EOF'
# NullBox v0.1 service configuration
# Services are started in dependency order by nulld.
#
# NOTE: Service daemons (egress, ctxgraph, cage) will be added
# as they are implemented as standalone binaries.
# For now, nulld boots to idle — verifying the init chain works.
EOF
echo "  Created nulld.toml"

# Create minimal /etc
echo "nullbox" > "${BUILD_DIR}/etc/hostname"
echo "nameserver 1.1.1.1" > "${BUILD_DIR}/etc/resolv.conf"
cat > "${BUILD_DIR}/etc/os-release" << 'EOF'
NAME="NullBox"
VERSION="0.1.0"
ID=nullbox
PRETTY_NAME="NullBox v0.1.0"
HOME_URL="https://github.com/nullbox-os/nullbox"
EOF
echo "  Created /etc files"

# Create example AGENT.toml for testing
mkdir -p "${BUILD_DIR}/agent"
cat > "${BUILD_DIR}/agent/test-agent.toml" << 'EOF'
[agent]
name = "test-agent"
version = "0.1.0"

[capabilities]
max_cpu_percent = 25
max_memory_mb = 256

[capabilities.network]
allow = ["httpbin.org"]

[capabilities.filesystem]
read = ["/data"]
write = ["/data/output"]
EOF
echo "  Created test AGENT.toml"

# Build SquashFS image with zstd compression
echo ">>> Building SquashFS image..."
mksquashfs "${BUILD_DIR}" "${OUTPUT_DIR}/nullbox.squashfs" \
    -comp zstd \
    -Xcompression-level 19 \
    -all-root \
    -noappend \
    -quiet

SQUASHFS_SIZE=$(du -h "${OUTPUT_DIR}/nullbox.squashfs" | cut -f1)

echo ""
echo "=== SquashFS build complete ==="
echo "  Output:  ${OUTPUT_DIR}/nullbox.squashfs"
echo "  Size:    ${SQUASHFS_SIZE}"
echo "  Target:  <100MB"
echo ""
echo "  Contents:"
ls -la "${BUILD_DIR}/system/bin/"
