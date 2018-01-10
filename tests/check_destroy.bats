#!/usr/bin/env bats

VMPREFIX=batstestvm

@test "Install VM - $VMPREFIX-centos7-destroy" {
    run kvm-install-vm create ${VMPREFIX}-centos7-destroy
    [ "$status" -eq 0 ]
}

@test "Shutdown/Destroy VM - $VMPREFIX-centos7-destroy" {
    run virsh destroy $VMPREFIX-centos7-destroy
    [ "$status" -eq 0 ]
}

@test "Delete VM - $VMPREFIX-centos7-destroy" {
    run kvm-install-vm remove ${VMPREFIX}-centos7-destroy
    [[ "${lines[0]}" =~ "Domain is not running" ]]
    [ "$status" -eq 0 ]
}
