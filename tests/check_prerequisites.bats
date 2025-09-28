#!/usr/bin/env bats

@test "Check that virt-install is available" {
    run timeout $TIMEOUT command -v virt-install
    [ "$status" -eq 0 ]
}

@test "Check that virt-resize is available" {
    run timeout $TIMEOUT command -v virt-resize
    [ "$status" -eq 0 ]
}

@test "Check that qemu-img is available" {
    run timeout $TIMEOUT command -v qemu-img
    [ "$status" -eq 0 ]
}

@test "Check that virsh is available" {
    run timeout $TIMEOUT command -v virsh
    [ "$status" -eq 0 ]
}

@test "Check dhcp_release availability (optional)" {
    # This is an optional prerequisite - test should not fail if missing
    run timeout $TIMEOUT command -v dhcp_release
    if [ "$status" -eq 0 ]; then
        echo "dhcp_release is available for automatic DHCP lease cleanup"
    else
        echo "dhcp_release not found - DHCP lease cleanup will be manual"
        echo "Install dnsmasq-utils package to enable automatic DHCP lease cleanup"
    fi
}
