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

@test "Install VM (CentOS 8) - $VMNAME-centos8" {
    create_test_vm centos8
}

@test "Delete VM (CentOS 8) - $VMNAME-centos8" {
    remove_test_vm centos8
}

@test "Install VM (Fedora 29) - $VMNAME-fedora29" {
    create_test_vm fedora27
}

@test "Delete VM (Fedora 29) - $VMNAME-fedora29" {
    remove_test_vm fedora27
}

@test "Install VM (Fedora 29 Atomic) - $VMNAME-fedora29-atomic" {
    create_test_vm fedora27-atomic
}

@test "Delete VM (Fedora 29 Atomic) - $VMNAME-fedora29-atomic" {
    remove_test_vm fedora27-atomic
}

@test "Install VM (Fedora 30) - $VMNAME-fedora30" {
    create_test_vm fedora28
}

@test "Delete VM (Fedora 30) - $VMNAME-fedora30" {
    remove_test_vm fedora28
}

@test "Install VM (Fedora 31) - $VMNAME-fedora31" {
    create_test_vm fedora31
}

@test "Delete VM (Fedora 31) - $VMNAME-fedora31" {
    remove_test_vm fedora31
}

@test "Install VM (Fedora 32) - $VMNAME-fedora32" {
    create_test_vm fedora32
}

@test "Delete VM (Fedora 32) - $VMNAME-fedora32" {
    remove_test_vm fedora32
}

@test "Install VM (Fedora 33) - $VMNAME-fedora33" {
    create_test_vm fedora33
}

@test "Delete VM (Fedora 33) - $VMNAME-fedora33" {
    remove_test_vm fedora33
}

@test "Install VM (Fedora 34) - $VMNAME-fedora34" {
    create_test_vm fedora34
}

@test "Delete VM (Fedora 34) - $VMNAME-fedora34" {
    remove_test_vm fedora34
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

@test "Install VM (Ubuntu 20.04) - $VMNAME-ubuntu2004" {
    create_test_vm ubuntu2004
}

@test "Delete VM (Ubuntu 20.04) - $VMNAME-ubuntu2004" {
    remove_test_vm ubuntu2004
}

@test "Install VM (Ubuntu 24.04) - $VMNAME-ubuntu2404" {
    create_test_vm ubuntu2404
}

@test "Delete VM (Ubuntu 24.04) - $VMNAME-ubuntu2404" {
    remove_test_vm ubuntu2404
}

@test "Install VM (Debian 9) - $VMNAME-debian9" {
    create_test_vm debian9
}

@test "Delete VM (Debian 9) - $VMNAME-debian9" {
    remove_test_vm debian9
}

@test "Install VM (Debian 10) - $VMNAME-debian10" {
    create_test_vm debian10
}

@test "Delete VM (Debian 10) - $VMNAME-debian10" {
    remove_test_vm debian10
}

@test "Install VM (openSUSE Leap 15) - $VMNAME-opensuse15" {
    create_test_vm opensuse15
}

@test "Delete VM (openSUSE Leap 15) - $VMNAME-opensuse15" {
    remove_test_vm opensuse15
}
