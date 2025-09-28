#!/usr/bin/env bats

@test "Check for help usage message" {
    run ./kvm-install-vm
    [[ "${lines[0]}" =~ "NAME" ]]
}

@test "Test create with no hostname" {
    run ./kvm-install-vm create
    [[ "${lines[0]}" =~ "NAME" ]]
}

@test "Test create with options and no VM name" {
    run ./kvm-install-vm create -t debian12 -d 20
    [[ "${lines[0]}" =~ "Please specify a single host to create." ]]
}

@test "Test remove with no hostname" {
    run ./kvm-install-vm remove
    [[ "${lines[0]}" =~ "NAME" ]]
}

@test "Install VM (Rocky9) - $VMNAME" {
    run ./kvm-install-vm create "$VMNAME"
    [ "$status" -eq 0 ]
}

@test "Check running VM state" {
    run virsh -q domstate $VMNAME
    [ "$output" = "running" ]
}

#@test "Check libvirt-nss hostname resolution" {
#    run sleep 45
#    run ping -c 1 "$VMNAME"
#    [ "$status" -eq 0 ]
#    [[ "${lines[-2]}" =~ "1 packets transmitted, 1 received," ]]
#}

@test "Attach disk to VM without specifying target" {
    run ./kvm-install-vm attach-disk -d 1 "$VMNAME"
    [ "$status" -eq 2 ]
    [[ "${lines[0]}" =~ "ERR: You must specify a target device" ]]
}

@test "Attach disk to VM without specifying disk size" {
    run ./kvm-install-vm attach-disk -t vdb "$VMNAME"
    [ "$status" -eq 2 ]
    [[ "${lines[0]}" =~ "You must specify a size" ]]
}

@test "Attach disk to VM" {
    run ./kvm-install-vm attach-disk -d 1 -t vdb "$VMNAME"
    [ "$status" -eq 0 ]
}

@test "Check block list for VM" {
    run grep " vdb " <(virsh domblklist "$VMNAME")
    [ "$status" -eq 0 ]
}

@test "Delete VM - $VMNAME" {
    run ./kvm-install-vm remove "$VMNAME"
    [ "$status" -eq 0 ]
}

@test "Check destroyed VM state" {
    run virsh -q domstate "$VMNAME"
    [[ "$output" =~ "error: failed to get domain '$VMNAME'" ]]
}

@test "Check destroyed VM files" {
    run ls "${VMDIR}/${VMNAME}"
    [[ "$output" =~ "No such file or directory" ]]
}
