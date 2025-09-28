#!/bin/bash

# Suite-level setup and teardown for kvm-install-vm tests
# This runs once before all tests and once after all tests complete

# Load shared test configuration
VMPREFIX=batstestvm
TIMEOUT=60

# VM directory configuration (from vmdir.bash)
VMDIR=${HOME}/virt/vms

# Load configuration if available
if [[ -f .kivrc ]]; then
    source .kivrc
fi

function cleanup_all_test_vms {
    echo "Cleaning up all test VMs..." >&2

    # Get all domains that start with our test prefix, filter empty lines
    local test_vms
    test_vms=$(virsh list --all --name 2>/dev/null | grep "^${VMPREFIX}" 2>/dev/null | grep -v "^$" || true)

    if [[ -n "$test_vms" ]]; then
        while IFS= read -r vm; do
            if [[ -n "$vm" && "$vm" =~ ^${VMPREFIX} ]]; then
                echo "Removing test VM: $vm" >&2
                # Use virsh directly to avoid potential DHCP release hangs in cleanup
                virsh destroy "$vm" >/dev/null 2>&1 || true
                virsh undefine "$vm" --managed-save --snapshots-metadata --nvram >/dev/null 2>&1 || true
                # Remove VM files
                rm -rf "$HOME/virt/vms/$vm" 2>/dev/null || true
                rm -rf "$HOME/virt/images/$vm" 2>/dev/null || true
            fi
        done <<< "$test_vms"
    fi

    # Clean up test directories and files
    rm -rf "$HOME/virt/.tests" 2>/dev/null || true
    rm -f "$HOME/virt/images/Rocky-8-GenericCloud-Base.latest.x86_64.qcow2" 2>/dev/null || true
}

setup_suite() {
    cleanup_all_test_vms

    # Generate unique VM name if not set
    if [ -z "${VMNAME}" ]; then
        TIMESTAMP=$(date '+%Y%m%d%H%M%S')
        VMNAME="batstestvm-${TIMESTAMP}"
    fi

    # Export variables for all test files
    export VMPREFIX
    export TIMEOUT
    export VMDIR
    export VMNAME

    # Create fresh test directory
    mkdir -p "$HOME/virt/.tests"
}

teardown_suite() {
    cleanup_all_test_vms
}
