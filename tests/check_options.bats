#!/usr/bin/env bats

@test "Check for help usage message" {
    run kvm-install-vm
    [ "$output" = "You must specify a name for the VM with -n. Use -h to see usage." ]
}

@test "Install VM - batstestvm" {
    run bash -c "kvm-install-vm -n batstestvm"
    [ "$status" -eq 0 ]
}

@test "Check running VM state" {
    run bash -c "virsh -q domstate batstestvm"
    [ "$output" = "running" ]
}

@test "Delete VM - batstestvm" {
    run bash -c "kvm-install-vm -r batstestvm"
    [ "$status" -eq 0 ]
}

@test "Check destroyed VM state" {
    run bash -c "virsh -q domstate batstestvm"
    [[ "$output" =~ "error: failed to get domain 'batstestvm'" ]]
}
