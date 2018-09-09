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

@test "Install VM (Amazon Linux 2) - $VMNAME-amazon2" {
    create_test_vm amazon2
}

@test "Delete VM (Amazon Linux 2) - $VMNAME-amazon2" {
    remove_test_vm amazon2
}

@test "Install VM (CentOS 7 Atomic) - $VMNAME-centos7-atomic" {
    create_test_vm centos7-atomic
}

@test "Delete VM (CentOS 7 Atomic) - $VMNAME-centos7-atomic" {
    remove_test_vm centos7-atomic
}

@test "Install VM (Fedora 27) - $VMNAME-fedora27" {
    create_test_vm fedora27
}

@test "Delete VM (Fedora 27) - $VMNAME-fedora27" {
    remove_test_vm fedora27
}

@test "Install VM (Fedora 27 Atomic) - $VMNAME-fedora27-atomic" {
    create_test_vm fedora27-atomic
}

@test "Delete VM (Fedora 27 Atomic) - $VMNAME-fedora27-atomic" {
    remove_test_vm fedora27-atomic
}

@test "Install VM (Fedora 28) - $VMNAME-fedora28" {
    create_test_vm fedora28
}

@test "Delete VM (Fedora 28) - $VMNAME-fedora28" {
    remove_test_vm fedora28
}

@test "Install VM (Fedora 28 Atomic) - $VMNAME-fedora28-atomic" {
    create_test_vm fedora28-atomic
}

@test "Delete VM (Fedora 28 Atomic) - $VMNAME-fedora28-atomic" {
    remove_test_vm fedora28-atomic
}

@test "Install VM (Ubuntu 16.04) - $VMNAME-ubuntu1604" {
    create_test_vm ubuntu1604
}

@test "Delete VM (Ubuntu 16.04) - $VMNAME-ubuntu1604" {
    remove_test_vm ubuntu1604
}

@test "Install VM (Ubuntu 18.04) - $VMNAME-ubuntu1804" {
    create_test_vm ubuntu1804
}

@test "Delete VM (Ubuntu 18.04) - $VMNAME-ubuntu1804" {
    remove_test_vm ubuntu1804
}

@test "Install VM (Debian 9) - $VMNAME-debian9" {
    create_test_vm debian9
}

@test "Delete VM (Debian 9) - $VMNAME-debian9" {
    remove_test_vm debian9
}
