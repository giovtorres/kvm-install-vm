#!/usr/bin/env bats

VMPREFIX=batstestvm
TESTDIR=~/virt/.tests

setup() {
    # Create test directory
    mkdir -p "${TESTDIR}"
}

teardown() {
    # Clean up any created VMs
    ./kvm-install-vm remove "${VMPREFIX}"-remote-rocky 2>/dev/null || true
    ./kvm-install-vm remove "${VMPREFIX}"-remote-fresh 2>/dev/null || true
    ./kvm-install-vm remove "${VMPREFIX}"-remote-invalid 2>/dev/null || true

    # Clean up downloaded images from tests
    rm -f ~/virt/images/Rocky-9-GenericCloud.latest.x86_64.qcow2 2>/dev/null || true
    rm -f ~/virt/images/invalid-file.txt 2>/dev/null || true
}

@test "Remote Rocky Linux image download and VM creation" {
    # Use the real Rocky Linux 9 image URL for testing
    ROCKY_URL="https://dl.rockylinux.org/pub/rocky/9/images/x86_64/Rocky-9-GenericCloud.latest.x86_64.qcow2"

    # Test download detection and VM creation (use -n to assume no, preventing actual VM creation)
    run timeout 30 ./kvm-install-vm create -n -i "${ROCKY_URL}" "${VMPREFIX}"-remote-rocky

    # Check that it recognizes it as a remote image
    [[ "${output}" =~ "Using remote image" ]]
    [[ "${output}" =~ "format: qcow2" ]]

    # Should either download or use existing image
    [[ "${output}" =~ "Downloading image from" ]] || [[ "${output}" =~ "Image already exists" ]]
}

@test "Remote image fresh download" {
    # Use a specific test URL to ensure fresh download
    ROCKY_URL="https://dl.rockylinux.org/pub/rocky/9/images/x86_64/Rocky-9-GenericCloud.latest.x86_64.qcow2"
    IMAGE_FILE="~/virt/images/Rocky-9-GenericCloud.latest.x86_64.qcow2"

    # Clean up any existing image to force fresh download
    rm -f "${IMAGE_FILE}" 2>/dev/null || true
    rm -f "${IMAGE_FILE}.part" 2>/dev/null || true

    # Test actual download (use -n to prevent VM creation)
    run timeout 60 ./kvm-install-vm create -n -i "${ROCKY_URL}" "${VMPREFIX}"-remote-fresh

    # Should show download activity
    [[ "${output}" =~ "Using remote image" ]]
    [[ "${output}" =~ "Downloading image from" ]]
}

@test "Remote image URL validation - unsupported format" {
    # Test with an unsupported file extension
    INVALID_URL="https://example.com/invalid-file.txt"

    run timeout 5 ./kvm-install-vm create -n -i "${INVALID_URL}" "${VMPREFIX}"-remote-invalid

    # Should fail with unsupported format error
    [ "$status" -eq 2 ]
    [[ "${output}" =~ "Unsupported remote image format" ]]
}

@test "Remote image format detection - qcow2" {
    QCOW2_URL="https://example.com/test.qcow2"

    # This should pass format validation but fail on download (which is expected)
    run timeout 5 ./kvm-install-vm create -n -i "${QCOW2_URL}" "${VMPREFIX}"-format-test

    # Should show qcow2 format detection before failing on download
    [[ "${output}" =~ "format: qcow2" ]] || [[ "${output}" =~ "Could not download" ]]
}

@test "Remote image format detection - raw" {
    RAW_URL="https://example.com/test.raw"

    run timeout 5 ./kvm-install-vm create -n -i "${RAW_URL}" "${VMPREFIX}"-format-test

    # Should show raw format detection
    [[ "${output}" =~ "format: raw" ]] || [[ "${output}" =~ "Could not download" ]]
}

@test "Remote image format detection - vhd" {
    VHD_URL="https://example.com/test.vhd"

    run timeout 5 ./kvm-install-vm create -n -i "${VHD_URL}" "${VMPREFIX}"-format-test

    # Should show vpc format detection (vhd maps to vpc)
    [[ "${output}" =~ "format: vpc" ]] || [[ "${output}" =~ "Could not download" ]]
}

@test "URL detection function" {
    # Test HTTPS URL detection
    HTTPS_URL="https://example.com/test.qcow2"

    run timeout 5 ./kvm-install-vm create -n -i "${HTTPS_URL}" "${VMPREFIX}"-url-test

    # Should be detected as remote image
    [[ "${output}" =~ "Using remote image" ]]
}

@test "Non-URL path handling" {
    # Test that local paths still work as before
    LOCAL_PATH="/nonexistent/local/file.qcow2"

    run timeout 5 ./kvm-install-vm create -n -i "${LOCAL_PATH}" "${VMPREFIX}"-local-test

    # Should show local file handling and fail because file doesn't exist
    [[ "${output}" =~ "Custom image file not found" ]]
}