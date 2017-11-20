#!/usr/bin/env bats

VMNAME=batstestvm

@test "Check for help usage message" {
    run kvm-install-vm
    [ "$status" -eq 1 ]
    [[ "${lines[0]}" =~ "NAME" ]]
}

@test "Test create with no hostname" {
    run kvm-install-vm create
    [ "$status" -eq 1 ]
    [[ "${lines[0]}" =~ "NAME" ]]
}

@test "Test create with options and no hostname" {
    run kvm-install-vm create -t debian9 -d 20
    [ "$status" -eq 1 ]
    [[ "${lines[0]}" =~ "Please specify a single host to create." ]]
}

@test "Test remove with no hostname" {
    run kvm-install-vm remove
    [ "$status" -eq 1 ]
    [[ "${lines[0]}" =~ "NAME" ]]
}

@test "Install VM - $VMNAME" {
    run kvm-install-vm create $VMNAME
    [ "$status" -eq 0 ]
}

@test "Check running VM state" {
    run bash -c "virsh -q domstate $VMNAME"
    [ "$output" = "running" ]
}

@test "Check libvirt-nss hostname resolution" {
    run sleep 10
    run ping -c 1 $VMNAME
    [ "$status" -eq 0 ]
    [[ "${lines[-2]}" =~ "1 packets transmitted, 1 received," ]]
}

@test "Check cloud-init package is removed" {
    run ssh -o StrictHostKeyChecking=no $VMNAME rpm -q cloud-init
    [[ "$output" =~ "package cloud-init is not installed" ]]
}

@test "Delete VM - $VMNAME" {
    run bash -c "kvm-install-vm remove $VMNAME"
    [ "$status" -eq 0 ]
}

@test "Check destroyed VM state" {
    run bash -c "virsh -q domstate $VMNAME"
    [[ "$output" =~ "error: failed to get domain '$VMNAME'" ]]
}
