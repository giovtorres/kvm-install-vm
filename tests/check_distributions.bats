#!/usr/bin/env bats

VMNAME=batstestvm

function create_test_vm ()
{
    local -r var="$1"
    run ./kvm-install-vm create -t ${var} ${VMNAME}-${var}
    [ "$status" -eq 0 ]
}

function remove_test_vm ()
{
    local -r var="$1"
    run ./kvm-install-vm remove ${VMNAME}-${var}
    [ "$status" -eq 0 ]
}

@test "Install VM (Ubuntu 24.04) - $VMNAME-ubuntu24.04" {
    create_test_vm ubuntu24.04
}

@test "Delete VM (Ubuntu 24.04) - $VMNAME-ubuntu24.04" {
    remove_test_vm ubuntu24.04
}
