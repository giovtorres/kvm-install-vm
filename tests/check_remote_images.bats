#!/usr/bin/env bats

TESTDIR=~/virt/.tests

teardown() {
    # Clean up specific VMs that THIS test might have created
    # (Global cleanup will catch anything we miss)
    ./kvm-install-vm remove "${VMPREFIX}"-remote-rocky 2>/dev/null || true
    ./kvm-install-vm remove "${VMPREFIX}"-remote-fresh 2>/dev/null || true
    ./kvm-install-vm remove "${VMPREFIX}"-remote-invalid 2>/dev/null || true
}

@test "Remote Rocky Linux image download and VM creation" {
    # Use the real Rocky Linux 8 image URL for testing
    ROCKY_URL="https://dl.rockylinux.org/pub/rocky/8/images/x86_64/Rocky-8-GenericCloud-Base.latest.x86_64.qcow2"

    # Use short timeout to test download start, not completion
    run timeout 10 ./kvm-install-vm create -i "${ROCKY_URL}" "${VMPREFIX}"-remote-rocky

    # Should show remote image detection
    [[ "${output}" =~ "Using remote image" ]]
    [[ "${output}" =~ "format: qcow2" ]]

    # Should show download activity or timeout (both prove download functionality works)
    if [ "$status" -eq 124 ]; then
        # Timeout occurred - check if download actually started
        IMAGE_FILE="$HOME/virt/images/Rocky-8-GenericCloud-Base.latest.x86_64.qcow2"
        [[ -f "$IMAGE_FILE.part" ]] || [[ -f "$IMAGE_FILE" ]]
    else
        # Command completed - should show download activity or existing image
        [[ "${output}" =~ "Downloading image from" ]] || [[ "${output}" =~ "Image already exists" ]]
    fi
}

@test "Remote image fresh download" {
    ROCKY_URL="https://dl.rockylinux.org/pub/rocky/8/images/x86_64/Rocky-8-GenericCloud-Base.latest.x86_64.qcow2"
    IMAGE_FILE="$HOME/virt/images/Rocky-8-GenericCloud-Base.latest.x86_64.qcow2"

    # Clean up any existing image to force fresh download
    rm -f "$IMAGE_FILE" 2>/dev/null || true
    rm -f "$IMAGE_FILE.part" 2>/dev/null || true

    # Use short timeout to test download start, not completion
    run timeout 10 ./kvm-install-vm create -i "${ROCKY_URL}" "${VMPREFIX}"-remote-fresh

    # Should show download activity or timeout (both prove download functionality works)
    if [ "$status" -eq 124 ]; then
        # Timeout occurred - check if download actually started
        [[ -f "$IMAGE_FILE.part" ]] || [[ -f "$IMAGE_FILE" ]]
    else
        # Command completed - should show download activity
        [[ "${output}" =~ "Using remote image" ]]
        [[ "${output}" =~ "Downloading image from" ]]
    fi
}

@test "Remote image URL validation - unsupported format" {
    INVALID_URL="https://example.com/invalid-file.txt"

    run timeout $TIMEOUT ./kvm-install-vm create -i "${INVALID_URL}" "${VMPREFIX}"-remote-invalid

    [ "$status" -eq 2 ]
    [[ "${output}" =~ "Unsupported remote image format" ]]
}

@test "Remote image format detection - qcow2" {
    QCOW2_URL="https://example.com/test.qcow2"

    # This should pass format validation but fail on download (which is expected)
    run timeout $TIMEOUT ./kvm-install-vm create -i "${QCOW2_URL}" "${VMPREFIX}"-format-test

    [[ "${output}" =~ "format: qcow2" ]]
}

@test "Remote image format detection - raw" {
    RAW_URL="https://example.com/test.raw"

    run timeout $TIMEOUT ./kvm-install-vm create -i "${RAW_URL}" "${VMPREFIX}"-format-test

    [[ "${output}" =~ "format: raw" ]]
}

@test "Remote image format detection - vhd" {
    VHD_URL="https://example.com/test.vhd"

    run timeout $TIMEOUT ./kvm-install-vm create -i "${VHD_URL}" "${VMPREFIX}"-format-test

    [[ "${output}" =~ "format: vpc" ]]
}

@test "URL detection function" {
    HTTPS_URL="https://example.com/test.qcow2"

    run timeout $TIMEOUT ./kvm-install-vm create -i "${HTTPS_URL}" "${VMPREFIX}"-url-test

    [[ "${output}" =~ "Using remote image" ]]
}

@test "Non-URL path handling" {
    LOCAL_PATH="/nonexistent/local/file.qcow2"

    run timeout $TIMEOUT ./kvm-install-vm create -i "${LOCAL_PATH}" "${VMPREFIX}"-local-test

    [[ "${output}" =~ "Custom image file not found" ]]
}
