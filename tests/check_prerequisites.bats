#!/usr/bin/env bats

@test "Check that genisoimage is available" {
    command -v genisoimage
}

@test "Check that virt-install is available" {
    command -v virt-install
}

@test "Check that virt-resize is available" {
    command -v virt-resize
}

@test "Check that qemu-img is available" {
    command -v qemu-img
}

@test "Check that virsh is available" {
    command -v virsh
}
