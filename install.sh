#!/usr/bin/env bash
#
# install.sh — One-command NullBox installer
#
# Usage:
#   curl -fsSL https://nullbox.dev/install.sh | bash
#   # or
#   ./install.sh [--iso /path/to/nullbox.iso] [--memory 4096] [--cpus 4]
#
# Supports:
#   - Linux (QEMU/KVM via libvirt + virt-manager)
#   - macOS (UTM or QEMU via Homebrew)

set -euo pipefail

NULLBOX_VERSION="0.1.0"
ISO_URL="https://github.com/rohansx/nullbox/releases/download/v${NULLBOX_VERSION}/nullbox-x86_64.iso"
VM_NAME="nullbox"
MEMORY=4096
CPUS=4
ISO_PATH=""

# Parse args
for arg in "$@"; do
    case "${arg}" in
        --iso=*) ISO_PATH="${arg#*=}" ;;
        --memory=*) MEMORY="${arg#*=}" ;;
        --cpus=*) CPUS="${arg#*=}" ;;
    esac
done

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}"
echo "  _   _       _ _ ____            "
echo " | \ | |_   _| | | __ )  _____  __"
echo " |  \| | | | | | |  _ \ / _ \ \/ /"
echo " | |\  | |_| | | | |_) | (_) >  < "
echo " |_| \_|\__,_|_|_|____/ \___/_/\_\\"
echo -e "${NC}"
echo "  The Operating System for AI Agents"
echo "  v${NULLBOX_VERSION}"
echo ""

OS="$(uname -s)"

# ── Download ISO ──────────────────────────────────────────────────────────

download_iso() {
    local dest="$1"
    if [[ -n "${ISO_PATH}" && -f "${ISO_PATH}" ]]; then
        echo "  Using local ISO: ${ISO_PATH}"
        cp "${ISO_PATH}" "${dest}"
        return
    fi

    echo "  Downloading NullBox ISO (~64MB)..."
    if command -v curl &>/dev/null; then
        curl -fSL -o "${dest}" "${ISO_URL}"
    elif command -v wget &>/dev/null; then
        wget -q -O "${dest}" "${ISO_URL}"
    else
        echo -e "${RED}error: curl or wget required${NC}"
        exit 1
    fi
    echo -e "  ${GREEN}Downloaded${NC}"
}

# ── Linux Install ─────────────────────────────────────────────────────────

install_linux() {
    echo "  Platform: Linux"
    echo ""

    # Check KVM
    if [[ ! -e /dev/kvm ]]; then
        echo -e "${RED}error: KVM not available. Enable VT-x/AMD-V in BIOS.${NC}"
        exit 1
    fi

    # Install deps
    if command -v pacman &>/dev/null; then
        echo "  Installing virt-manager (pacman)..."
        sudo pacman -S --needed --noconfirm virt-manager qemu-desktop libvirt dnsmasq edk2-ovmf python-gobject 2>/dev/null || true
    elif command -v apt &>/dev/null; then
        echo "  Installing virt-manager (apt)..."
        sudo apt-get install -y virt-manager qemu-kvm libvirt-daemon-system 2>/dev/null || true
    elif command -v dnf &>/dev/null; then
        echo "  Installing virt-manager (dnf)..."
        sudo dnf install -y virt-manager qemu-kvm libvirt 2>/dev/null || true
    else
        echo "  Please install virt-manager manually for your distro"
    fi

    # Enable libvirt
    sudo systemctl enable --now libvirtd 2>/dev/null || true
    sudo virsh net-start default 2>/dev/null || true

    # Download ISO
    ISO_DEST="/var/lib/libvirt/images/nullbox.iso"
    sudo mkdir -p /var/lib/libvirt/images
    download_iso "/tmp/nullbox.iso"
    sudo mv /tmp/nullbox.iso "${ISO_DEST}"
    sudo chmod 644 "${ISO_DEST}"

    # Remove existing VM
    sudo virsh destroy "${VM_NAME}" 2>/dev/null || true
    sudo virsh undefine "${VM_NAME}" 2>/dev/null || true

    # Create VM
    echo "  Creating NullBox VM (${MEMORY}MB RAM, ${CPUS} CPUs)..."
    sudo virt-install \
        --name "${VM_NAME}" \
        --memory "${MEMORY}" \
        --vcpus "${CPUS}" \
        --cpu host-passthrough \
        --cdrom "${ISO_DEST}" \
        --os-variant linux2022 \
        --network network=default,model=virtio \
        --graphics spice \
        --video virtio \
        --disk none \
        --noautoconsole \
        --boot cdrom 2>/dev/null

    echo ""
    echo -e "${GREEN}  NullBox is running!${NC}"
    echo ""
    echo "  Open the console:"
    echo "    virt-manager"
    echo ""
    echo "  Or connect via serial:"
    echo "    sudo virsh console nullbox"
    echo ""
    echo "  Stop:"
    echo "    sudo virsh destroy nullbox"
    echo ""
    echo "  Start again:"
    echo "    sudo virsh start nullbox"
}

# ── macOS Install ─────────────────────────────────────────────────────────

install_macos() {
    echo "  Platform: macOS"
    echo ""

    ARCH="$(uname -m)"

    # Check for UTM first (preferred on macOS)
    if [[ -d "/Applications/UTM.app" ]]; then
        echo "  Found UTM — using it for NullBox VM"
        install_macos_utm
        return
    fi

    # Check for QEMU via Homebrew
    if command -v qemu-system-x86_64 &>/dev/null; then
        echo "  Found QEMU — using it for NullBox VM"
        install_macos_qemu
        return
    fi

    # Neither found — install UTM
    echo "  Installing UTM (VM manager for macOS)..."
    if command -v brew &>/dev/null; then
        brew install --cask utm
        install_macos_utm
    else
        echo ""
        echo "  Install UTM from: https://mac.getutm.app/"
        echo "  Or install Homebrew: https://brew.sh"
        echo ""
        echo "  Then run this script again."
        exit 1
    fi
}

install_macos_utm() {
    local iso_dest="${HOME}/Library/Containers/com.utmapp.UTM/Data/Documents/nullbox.iso"
    mkdir -p "$(dirname "${iso_dest}")" 2>/dev/null || true

    # For UTM, just download the ISO and tell the user to import it
    download_iso "${HOME}/Downloads/nullbox.iso"

    echo ""
    echo -e "${GREEN}  ISO downloaded to ~/Downloads/nullbox.iso${NC}"
    echo ""
    echo "  To create the VM in UTM:"
    echo "  1. Open UTM"
    echo "  2. Click '+' → Virtualize (Intel Mac) or Emulate (Apple Silicon)"
    echo "  3. Select 'Linux'"
    echo "  4. Browse to ~/Downloads/nullbox.iso"
    echo "  5. Set RAM to ${MEMORY}MB, CPUs to ${CPUS}"
    echo "  6. Uncheck 'Enable hardware OpenGL acceleration'"
    echo "  7. Click 'Save' then 'Play'"
    echo ""
    echo "  Note: On Apple Silicon, NullBox runs in emulation mode (slower)."
    echo "  For native ARM64 support, build with: ./kernel/scripts/build-kernel.sh --arch aarch64"
}

install_macos_qemu() {
    download_iso "/tmp/nullbox.iso"

    echo ""
    echo -e "${GREEN}  Starting NullBox in QEMU...${NC}"
    echo ""

    local accel="tcg"
    if [[ "$(uname -m)" == "x86_64" ]]; then
        # Intel Mac — can use HVF
        accel="hvf"
    fi

    echo "  Run:"
    echo "    qemu-system-x86_64 \\"
    echo "      -accel ${accel} -m ${MEMORY} -smp ${CPUS} \\"
    echo "      -nographic -serial mon:stdio \\"
    echo "      -cdrom /tmp/nullbox.iso \\"
    echo "      -nic user,model=virtio,hostfwd=tcp::19100-:9100"
}

# ── Dispatch ──────────────────────────────────────────────────────────────

case "${OS}" in
    Linux)  install_linux ;;
    Darwin) install_macos ;;
    *)
        echo -e "${RED}Unsupported platform: ${OS}${NC}"
        echo "NullBox supports Linux and macOS."
        exit 1
        ;;
esac
