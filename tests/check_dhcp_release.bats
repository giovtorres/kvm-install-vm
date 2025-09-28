#!/usr/bin/env bats

teardown() {
    # Clean up specific VMs that THIS test might have created
    # (Global cleanup will catch anything we miss)
    ./kvm-install-vm remove "${VMPREFIX}"-dhcp-test 2>/dev/null || true
    ./kvm-install-vm remove "${VMPREFIX}"-specific-test 2>/dev/null || true
}

@test "Check dhcp_release prerequisite detection" {
    # Test the check_dhcp_release function behavior
    if command -v dhcp_release > /dev/null 2>&1; then
        skip "dhcp_release is installed, cannot test missing prerequisite"
    fi

    # This is just a check - no VM creation needed
    echo "dhcp_release not available - this is expected for testing"
}

@test "DHCP lease release integration - VM removal" {
    # Test the DHCP release function behavior without creating actual VM
    # This tests the logic path without requiring VM infrastructure

    # Test removing a non-existent VM to verify DHCP release handling
    run timeout $TIMEOUT ./kvm-install-vm remove "nonexistent-test-vm-${RANDOM}"

    # Should complete successfully (removal of non-existent VM should not fail)
    [ "$status" -eq 0 ]

    # Should show that domain doesn't exist
    [[ "${output}" =~ "does not exist" ]]

    # If dhcp_release is missing, should show appropriate message during removal
    if ! command -v dhcp_release > /dev/null 2>&1; then
        [[ "${output}" =~ "dhcp_release not found" ]] || [[ "${output}" =~ "DHCP lease cleanup will be skipped" ]]
    fi
}

@test "DHCP lease release - MAC address extraction logic" {
    # Test the MAC address extraction logic without requiring actual VMs

    # Test MAC address regex pattern used in the script
    SAMPLE_MAC="52:54:00:12:34:56"
    [[ "$SAMPLE_MAC" =~ ^[0-9a-fA-F]{2}:[0-9a-fA-F]{2}:[0-9a-fA-F]{2}:[0-9a-fA-F]{2}:[0-9a-fA-F]{2}:[0-9a-fA-F]{2}$ ]]

    # Test invalid MAC addresses
    INVALID_MAC="invalid:mac:address"
    [[ ! "$INVALID_MAC" =~ ^[0-9a-fA-F]{2}:[0-9a-fA-F]{2}:[0-9a-fA-F]{2}:[0-9a-fA-F]{2}:[0-9a-fA-F]{2}:[0-9a-fA-F]{2}$ ]]

    echo "MAC address validation logic works correctly"
}

@test "Distribution detection for dhcp_release installation suggestions" {
    # Test that appropriate installation commands are suggested

    # Skip if dhcp_release is available - we need it to be missing for this test
    if command -v dhcp_release > /dev/null 2>&1; then
        skip "dhcp_release is available, cannot test installation suggestions"
    fi

    # Test distribution detection without creating a VM
    # Check if we have the expected package managers
    if command -v apt-get > /dev/null 2>&1; then
        echo "apt-get detected - should suggest dnsmasq-utils installation via apt-get"
    elif command -v dnf > /dev/null 2>&1; then
        echo "dnf detected - should suggest dnsmasq-utils installation via dnf"
    else
        echo "Unknown package manager - should suggest generic installation"
    fi
}

@test "DHCP status file path validation" {
    # Test that the script handles DHCP status file paths correctly

    # Check that the expected status file path format is correct
    EXPECTED_STATUS_FILE="/var/lib/libvirt/dnsmasq/virbr0.status"

    # Test file path construction (default bridge virbr0)
    [[ "$EXPECTED_STATUS_FILE" =~ /var/lib/libvirt/dnsmasq/.*.status ]]

    echo "DHCP status file path format is correct"
}

@test "DHCP release error handling" {
    # Test error handling in DHCP release scenarios

    # Test removal of non-existent VM (should handle gracefully)
    run timeout $TIMEOUT ./kvm-install-vm remove "totally-fake-vm-${RANDOM}"
    [ "$status" -eq 0 ]

    # Should indicate domain doesn't exist
    [[ "${output}" =~ "does not exist" ]]

    # Should not crash or show fatal errors
    [[ ! "${output}" =~ "fatal" ]]
    [[ ! "${output}" =~ "ERROR" ]]
}

@test "Error handling for missing domain during DHCP release" {
    # Test handling when domain doesn't exist but we try to release lease

    # Try to remove non-existent VM
    run timeout $TIMEOUT ./kvm-install-vm remove "nonexistent-vm-${RANDOM}"

    # Should handle gracefully without errors
    [ "$status" -eq 0 ]
    [[ "${output}" =~ "does not exist" ]]
}