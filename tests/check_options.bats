#!/usr/bin/env bats

@test "Check for help usage message" {
    run kvm-install-vm
    [[ "$output" =~ "NAME" ]]
}

@test "Install VM - batstestvm" {
    run bash -c "kvm-install-vm create batstestvm"
    [ "$status" -eq 0 ]
}

@test "Check running VM state" {
    run bash -c "virsh -q domstate batstestvm"
    [ "$output" = "running" ]
}

@test "Delete VM - batstestvm" {
    run bash -c "kvm-install-vm remove batstestvm"
    [ "$status" -eq 0 ]
}

@test "Check destroyed VM state" {
    run bash -c "virsh -q domstate batstestvm"
    [[ "$output" =~ "error: failed to get domain 'batstestvm'" ]]
}
