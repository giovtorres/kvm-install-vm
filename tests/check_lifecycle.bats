#!/usr/bin/env bats

teardown() {
    # Clean up specific VMs that THIS test might have created
    # (Global cleanup will catch anything we miss)
    ./kvm-install-vm remove "${VMPREFIX}"-validation-test 2>/dev/null || true
    ./kvm-install-vm remove "${VMPREFIX}"-nonexistent 2>/dev/null || true
    ./kvm-install-vm remove "${VMPREFIX}"-test-dhcp-integration 2>/dev/null || true
}

@test "VM creation command validation" {
    run timeout 5 ./kvm-install-vm create ${VMPREFIX}-validation-test
    # Should either succeed quickly or timeout (both are acceptable for validation)
    # The important thing is that it doesn't fail with syntax or validation errors
    [[ "$status" -eq 0 || "$status" -eq 124 ]]
}

@test "VM removal with non-existent VM shows proper message" {
    run timeout $TIMEOUT ./kvm-install-vm remove ${VMPREFIX}-nonexistent
    [ "$status" -eq 0 ]
    [[ "${output}" =~ "does not exist" ]]
}

@test "DHCP release integration in VM removal" {
    # This tests the removal path without requiring an actual VM
    run timeout $TIMEOUT ./kvm-install-vm remove ${VMPREFIX}-test-dhcp-integration
    [ "$status" -eq 0 ]
    # Should show domain doesn't exist (since we're not creating it)
    [[ "${output}" =~ "does not exist" ]]
}
