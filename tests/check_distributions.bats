#!/usr/bin/env bats

VMNAME=batstestvm

function create_test_vm ()
{
    local -r var="$1"
    run kvm-install-vm create -t ${var} ${VMNAME}-${var}
    [ "$status" -eq 0 ]
}

function remove_test_vm ()
{
    local -r var="$1"
    run kvm-install-vm remove ${VMNAME}-${var}
    [ "$status" -eq 0 ]
}

@test "Install VM (CentOS 7 Atomic) - $VMNAME-centos7-atomic" {
    create_test_vm centos7-atomic
}

@test "Delete VM (CentOS 7 Atomic) - $VMNAME-centos7-atomic" {
    remove_test_vm centos7-atomic
}

@test "Install VM (Fedora 26 Atomic) - $VMNAME-fedora26-atomic" {
    create_test_vm fedora26-atomic
}

@test "Delete VM (Fedora 26 Atomic) - $VMNAME-fedora26-atomic" {
    remove_test_vm fedora26-atomic
}

@test "Install VM (Fedora 26) - $VMNAME-fedora26" {
    create_test_vm fedora26
}

@test "Delete VM (Fedora 26) - $VMNAME-fedora26" {
    remove_test_vm fedora26
}

@test "Install VM (Ubuntu 16.04) - $VMNAME-ubuntu1604" {
    create_test_vm ubuntu1604
}

@test "Delete VM (Ubuntu 16.04) - $VMNAME-ubuntu1604" {
    remove_test_vm ubuntu1604
}

@test "Install VM (Debian 9) - $VMNAME-debian9" {
    create_test_vm debian9
}

@test "Delete VM (Debian 9) - $VMNAME-debian9" {
    remove_test_vm debian9
}
