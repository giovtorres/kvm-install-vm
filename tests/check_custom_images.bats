#!/usr/bin/env bats

VMPREFIX=batstestvm
TESTDIR=~/virt/.tests

setup() {
    # Create test images for custom image tests
    mkdir -p ~/virt/.tests/
    qemu-img create -f qcow2 "${TESTDIR}"/test-image.qcow2 1M
    qemu-img create -f raw "${TESTDIR}"/test-image.raw 1M
    qemu-img create -f vpc "${TESTDIR}"/test-image.vhd 1M
}

teardown() {
    # Clean up test images
    rm -f "${TESTDIR}"/test-image.*
    rm -f relative-test.qcow2

    # Clean up any partially created VMs
    ./kvm-install-vm remove "${VMPREFIX}"-custom-qcow2 2>/dev/null || true
    ./kvm-install-vm remove "${VMPREFIX}"-custom-raw 2>/dev/null || true
    ./kvm-install-vm remove "${VMPREFIX}"-custom-vhd 2>/dev/null || true
    ./kvm-install-vm remove "${VMPREFIX}"-relative 2>/dev/null || true
}

@test "Custom qcow2 image format detection" {
    run timeout 1 ./kvm-install-vm create -n -i "${TESTDIR}"/test-image.qcow2 "${VMPREFIX}"-custom-qcow2
    [[ "${output}" =~ "format: qcow2" ]]
}

@test "Custom raw image format detection" {
    run timeout 1 ./kvm-install-vm create -n -i "${TESTDIR}"/test-image.raw "${VMPREFIX}"-custom-raw
    [[ "${output}" =~ "format: raw" ]]
}

@test "Custom vhd image format detection" {
    run timeout 1 ./kvm-install-vm create -n -i "${TESTDIR}"/test-image.vhd "${VMPREFIX}"-custom-vhd
    [[ "${output}" =~ "format: vpc" ]]
}

@test "Relative path support with custom image" {
    cp "${TESTDIR}"/test-image.qcow2 ./relative-test.qcow2

    run timeout 1 ./kvm-install-vm create -n -i ./relative-test.qcow2 "${VMPREFIX}"-relative.qcow2
    [[ "${output}" =~ "format: qcow2" ]]
    [[ "${output}" =~ "/relative-test.qcow2" ]]
}

@test "Nonexistent custom image fails gracefully" {
    run ./kvm-install-vm create -i /nonexistent/image.qcow2 "${VMPREFIX}"-nonexistent
    [ "$status" -ne 0 ]
}
