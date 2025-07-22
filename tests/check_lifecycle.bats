#!/usr/bin/env bats

VMPREFIX=batstestvm

@test "Install VM - $VMPREFIX-rocky9" {
    run ./kvm-install-vm create ${VMPREFIX}-rocky9
    [ "$status" -eq 0 ]
}

@test "Shutdown/Destroy VM - $VMPREFIX-rocky9" {
    run virsh destroy $VMPREFIX-rocky9
    [ "$status" -eq 0 ]
}

@test "Delete VM - $VMPREFIX-rocky9" {
    run ./kvm-install-vm remove ${VMPREFIX}-rocky9
    [[ "${lines[0]}" =~ "Domain is not running" ]]
    [ "$status" -eq 0 ]
}
