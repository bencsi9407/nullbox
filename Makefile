# NullBox Build System
#
# Targets:
#   make test         — Run all Rust unit tests
#   make build        — Build all Rust crates (debug)
#   make release      — Build all crates for release (static musl)
#   make kernel       — Build KSPP-hardened Linux kernel
#   make initramfs    — Build initramfs (needs kernel + nulld)
#   make squashfs     — Build SquashFS rootfs (needs release binaries)
#   make iso          — Build bootable x86_64 ISO
#   make qemu-kernel  — Boot kernel only in QEMU (validation)
#   make qemu         — Boot kernel+initramfs in QEMU
#   make qemu-iso     — Boot full ISO in QEMU
#   make all          — Build everything
#   make clean        — Remove build artifacts

SHELL := /bin/bash
NULLBOX_ROOT := $(shell pwd)
TARGET := x86_64-unknown-linux-musl
ARCH := x86_64

.PHONY: test build release kernel initramfs squashfs iso all clean \
        qemu-kernel qemu qemu-iso deps-check

# === Development ===

test:
	cargo test --workspace

build:
	cargo build --workspace

release:
	cargo build --workspace --exclude cage --target $(TARGET) --release
	cargo build -p cage --release

# === Kernel ===

kernel:
	@chmod +x kernel/scripts/build-kernel.sh
	ARCH=$(ARCH) kernel/scripts/build-kernel.sh

# === Image Pipeline ===

initramfs: release
	@chmod +x image/scripts/build-initramfs.sh
	image/scripts/build-initramfs.sh

squashfs: release
	@chmod +x image/scripts/build-squashfs.sh
	TARGET=$(TARGET) image/scripts/build-squashfs.sh

iso: kernel initramfs
	@chmod +x image/scripts/build-iso.sh
	image/scripts/build-iso.sh

# === QEMU Testing ===

qemu-kernel:
	@chmod +x image/scripts/test-qemu.sh
	image/scripts/test-qemu.sh kernel

qemu: kernel initramfs
	@chmod +x image/scripts/test-qemu.sh
	image/scripts/test-qemu.sh initramfs

qemu-iso: iso
	@chmod +x image/scripts/test-qemu.sh
	image/scripts/test-qemu.sh iso

# === Full Build ===

all: test release kernel squashfs initramfs iso

# === Dependencies Check ===

deps-check:
	@echo "Checking NullBox build dependencies..."
	@command -v cargo   >/dev/null && echo "  ✓ cargo"   || echo "  ✗ cargo (pacman -S rust)"
	@command -v clang   >/dev/null && echo "  ✓ clang"   || echo "  ✗ clang (pacman -S clang)"
	@command -v lld     >/dev/null && echo "  ✓ lld"     || echo "  ✗ lld (pacman -S lld)"
	@command -v llvm-ar >/dev/null && echo "  ✓ llvm"    || echo "  ✗ llvm (pacman -S llvm)"
	@command -v bc      >/dev/null && echo "  ✓ bc"      || echo "  ✗ bc (pacman -S bc)"
	@command -v flex    >/dev/null && echo "  ✓ flex"    || echo "  ✗ flex (pacman -S flex)"
	@command -v bison   >/dev/null && echo "  ✓ bison"   || echo "  ✗ bison (pacman -S bison)"
	@command -v mksquashfs >/dev/null && echo "  ✓ mksquashfs" || echo "  ✗ mksquashfs (pacman -S squashfs-tools)"
	@command -v xorriso >/dev/null && echo "  ✓ xorriso" || echo "  ✗ xorriso (pacman -S xorriso)"
	@command -v grub-mkrescue >/dev/null && echo "  ✓ grub-mkrescue" || echo "  ✗ grub-mkrescue (pacman -S grub)"
	@command -v qemu-system-x86_64 >/dev/null && echo "  ✓ qemu" || echo "  ✗ qemu (pacman -S qemu-full)"
	@[[ -e /dev/kvm ]] && echo "  ✓ KVM" || echo "  ✗ KVM (enable in BIOS)"
	@echo ""
	@echo "For musl target: rustup target add $(TARGET)"
	@echo "  (or: pacman -S rust-musl)"

# === Clean ===

clean:
	cargo clean
	rm -rf build/
