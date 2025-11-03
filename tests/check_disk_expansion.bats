#!/usr/bin/env bats

teardown() {
    # Clean up specific VMs that THIS test might have created
    ./kvm-install-vm remove "${VMPREFIX}"-expand-15g 2>/dev/null || true
}

@test "VM creation with -d 15 expands disk to 15GB" {
    # Create VM with 15GB disk
    run timeout $TIMEOUT ./kvm-install-vm create -d 15 "${VMPREFIX}"-expand-15g
    [ "$status" -eq 0 ]
    [[ "${output}" =~ "Resizing the disk to 15G" ]]

    # Shut down VM to release disk lock for qemu-img
    virsh shutdown "${VMPREFIX}"-expand-15g
    sleep 10

    # Verify disk size using qemu-img info
    DISK_PATH="${VMDIR}/${VMPREFIX}-expand-15g/${VMPREFIX}-expand-15g.qcow2"
    run qemu-img info "$DISK_PATH"
    [ "$status" -eq 0 ]
    [[ "${output}" =~ "virtual size: 15 GiB" ]]
}
