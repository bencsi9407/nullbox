#!/usr/bin/env bash
#
# build-kernel.sh — Build KSPP-hardened Linux kernel for NullBox
#
# Usage:
#   ./kernel/scripts/build-kernel.sh [--arch x86_64|aarch64] [--kernel-version 6.18.20]
#
# Requirements: clang, lld, llvm, llvm-ar, llvm-nm, llvm-strip, llvm-objcopy
#               bc, flex, bison, libelf, openssl (headers)

set -euo pipefail

NULLBOX_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
KERNEL_VERSION="${KERNEL_VERSION:-6.18.20}"
KERNEL_MAJOR="${KERNEL_VERSION%%.*}"
ARCH="${ARCH:-x86_64}"
JOBS="${JOBS:-$(nproc)}"

BUILD_DIR="${NULLBOX_ROOT}/build/kernel"
DOWNLOAD_DIR="${NULLBOX_ROOT}/build/downloads"
KERNEL_SRC="${BUILD_DIR}/linux-${KERNEL_VERSION}"
OUTPUT_DIR="${NULLBOX_ROOT}/build/output/kernel/${ARCH}"

# Map arch names
case "${ARCH}" in
    x86_64)
        KERNEL_ARCH="x86_64"
        LINUX_ARCH="x86"
        DEFCONFIG="${NULLBOX_ROOT}/kernel/config/x86_64_defconfig"
        ;;
    aarch64|arm64)
        KERNEL_ARCH="aarch64"
        LINUX_ARCH="arm64"
        DEFCONFIG="${NULLBOX_ROOT}/kernel/config/aarch64_defconfig"
        ;;
    *)
        echo "error: unsupported arch '${ARCH}'. Use x86_64 or aarch64."
        exit 1
        ;;
esac

echo "=== NullBox Kernel Build ==="
echo "  Kernel:  Linux ${KERNEL_VERSION}"
echo "  Arch:    ${ARCH}"
echo "  Config:  ${DEFCONFIG}"
echo "  Jobs:    ${JOBS}"
echo "  Output:  ${OUTPUT_DIR}"
echo ""

# Check required tools
for tool in clang lld llvm-ar llvm-nm llvm-strip llvm-objcopy bc flex bison; do
    if ! command -v "${tool}" &>/dev/null; then
        echo "error: required tool '${tool}' not found"
        echo "install: pacman -S clang lld llvm bc flex bison"
        exit 1
    fi
done

# Download kernel source if not present
mkdir -p "${DOWNLOAD_DIR}" "${BUILD_DIR}" "${OUTPUT_DIR}"

TARBALL="${DOWNLOAD_DIR}/linux-${KERNEL_VERSION}.tar.xz"
if [[ ! -f "${TARBALL}" ]]; then
    KERNEL_URL="https://cdn.kernel.org/pub/linux/kernel/v${KERNEL_MAJOR}.x/linux-${KERNEL_VERSION}.tar.xz"
    echo ">>> Downloading Linux ${KERNEL_VERSION}..."
    curl -L -o "${TARBALL}" "${KERNEL_URL}"
fi

# Extract if not already extracted
if [[ ! -d "${KERNEL_SRC}" ]]; then
    echo ">>> Extracting kernel source..."
    tar -xf "${TARBALL}" -C "${BUILD_DIR}"
fi

# Copy defconfig
echo ">>> Applying NullBox defconfig..."
cp "${DEFCONFIG}" "${KERNEL_SRC}/.config"

# Common make flags for Clang/LLVM build with ThinLTO
MAKE_FLAGS=(
    -C "${KERNEL_SRC}"
    "ARCH=${LINUX_ARCH}"
    "LLVM=1"
    "LLVM_IAS=1"
    "-j${JOBS}"
)

# Merge defconfig (resolve dependencies, set defaults for unset options)
echo ">>> Resolving kernel config..."
make "${MAKE_FLAGS[@]}" olddefconfig

# Build kernel
echo ">>> Building kernel (this takes a while)..."
make "${MAKE_FLAGS[@]}"

# Copy outputs
echo ">>> Copying build artifacts..."
case "${LINUX_ARCH}" in
    x86)
        cp "${KERNEL_SRC}/arch/x86/boot/bzImage" "${OUTPUT_DIR}/vmlinuz"
        ;;
    arm64)
        cp "${KERNEL_SRC}/arch/arm64/boot/Image" "${OUTPUT_DIR}/vmlinuz"
        ;;
esac

cp "${KERNEL_SRC}/System.map" "${OUTPUT_DIR}/System.map"
cp "${KERNEL_SRC}/.config" "${OUTPUT_DIR}/config"

echo ""
echo "=== Kernel build complete ==="
echo "  vmlinuz:    ${OUTPUT_DIR}/vmlinuz"
echo "  System.map: ${OUTPUT_DIR}/System.map"
echo "  config:     ${OUTPUT_DIR}/config"
echo ""
echo "  Size: $(du -h "${OUTPUT_DIR}/vmlinuz" | cut -f1)"
