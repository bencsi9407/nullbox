#!/usr/bin/env bash
#
# run.sh — Run NullBox in QEMU
#
# Usage:
#   ./run.sh              # boot with serial console in terminal
#   ./run.sh --ports      # boot with TCP port forwarding for interaction
#   ./run.sh --test       # boot, run e2e tests, exit
#
# Ctrl-A X to quit QEMU (or Ctrl-C)

set -euo pipefail

NULLBOX_ROOT="$(cd "$(dirname "$0")" && pwd)"
VMLINUZ="${NULLBOX_ROOT}/build/output/kernel/x86_64/vmlinuz"
INITRAMFS="${NULLBOX_ROOT}/build/output/initramfs/initramfs.cpio.gz"
ISO="${NULLBOX_ROOT}/build/output/nullbox-x86_64.iso"

MODE="${1:-}"

if [[ ! -f "${VMLINUZ}" || ! -f "${INITRAMFS}" ]]; then
    echo "Build NullBox first:"
    echo "  cargo build --workspace --target x86_64-unknown-linux-musl --release"
    echo "  ./image/scripts/build-squashfs.sh"
    echo "  ./image/scripts/build-initramfs.sh"
    exit 1
fi

case "${MODE}" in
    --test)
        exec bash "${NULLBOX_ROOT}/image/scripts/e2e-test.sh"
        ;;
    --ports)
        echo "NullBox running with port forwarding:"
        echo "  TCP 19100 → ctxgraph (9100)"
        echo "  TCP 19200 → test-harness (9200)"
        echo ""
        echo "Interact from another terminal:"
        echo "  echo '{\"method\":\"write\",\"agent_id\":\"me\",\"key\":\"hi\",\"value\":\"world\"}' | bash -c 'exec 3<>/dev/tcp/127.0.0.1/19100; cat >&3; read -t3 l <&3; echo \$l; exec 3>&-'"
        echo ""
        echo "Press Ctrl-A X to quit."
        echo ""
        exec qemu-system-x86_64 \
            -enable-kvm -m 4G -cpu host -smp 4 \
            -nographic -serial mon:stdio \
            -kernel "${VMLINUZ}" -initrd "${INITRAMFS}" \
            -append "console=ttyS0,115200 loglevel=4" \
            -nic user,model=virtio,hostfwd=tcp::19100-:9100,hostfwd=tcp::19200-:9200
        ;;
    *)
        echo "NullBox — The Operating System for AI Agents"
        echo ""
        echo "Press Ctrl-A X to quit."
        echo ""
        exec qemu-system-x86_64 \
            -enable-kvm -m 4G -cpu host -smp 4 \
            -nographic -serial mon:stdio \
            -kernel "${VMLINUZ}" -initrd "${INITRAMFS}" \
            -append "console=ttyS0,115200 loglevel=7"
        ;;
esac
