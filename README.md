## kvm-install-vm

A bash wrapper around virt-install to build virtual machines on a local KVM
hypervisor.  You can run it as a normal user which will use `qemu:///session` to
connect locally to your KVM domains.

Tested on the latest Fedora.

### Prerequisites

You need to have the KVM hypervisor installed, along with a few other packages (naming of packages can differ on other distributions):

- virt-install >= 3.0.0
- libguestfs-tools-c
- qemu-img
- libvirt-client
- libosinfo

To install the dependencies, run:

- Fedora example:

```
sudo dnf -y install virt-install libguestfs-tools-c qemu-img libvirt-client wget libosinfo
```

- Ubuntu example:

```
sudo apt install -y virtinst libguestfs-tools qemu-utils libvirt-clients wget libosinfo-bin
```

If you want to resolve guests by their hostnames, install the `libvirt-nss` package:

- Fedora example:

```
sudo dnf -y install libvirt-nss
```

- Ubuntu example:

```bash
sudo apt install -y libnss-libvirt
```

Then, add `libvirt` and `libvirt_guest` to list of **hosts** databases in
`/etc/nsswitch.conf`.  See [here](https://libvirt.org/nss.html) for more
information.

### Usage

```bash
./kvm-install-vm help
```

#### Creating Guest VMs

```bash
# Create VM with the default parameters: Rocky Linux 9, 1 vCPU, 1.5GB RAM, 10GB disk capacity, x86_64 arch
kvm-install-vm create myvm

# Create VM with custom parameters: 2 vCPUs, 2GB RAM, and 20GB disk capacity.
kvm-install-vm create -c 2 -m 2048 -d 20 myvm

# Create a Debian 12 VM with the default parameters.
kvm-install-vm create -t debian12 myvm

# Create a default VM with UTC timezone.
kvm-install-vm create -T UTC myvm
```

#### Deleting a Guest Domain

```bash
# Remove (destroy and undefine) a guest domain. WARNING: This will delete the guest domain and any changes made inside it!
kvm-install-vm remove myvm
```

#### Attaching a new disk

```bash
# Attach a 10GB disk device named example-5g.qcow2 to the myvm guest domain.
kvm-install-vm attach-disk -d 10 -s example-5g.qcow2 -t vdb myvm
```

### Setting Custom Defaults

Copy the `.kivrc` file to your $HOME directory to set custom defaults.  This is
convenient if you find yourself repeatedly setting the same options on the
command line, like the distribution or the number of vCPUs.

Options are evaluated in the following order:

- Default options set in the script
- Custom options set in `.kivrc`
- Option flags set on the command line

### Notes

1. This script will download a qcow2 cloud image from the respective
   distribution's download site.  See script for URLs.

2. If using libvirt-nss, keep in mind that DHCP leases take some time to
   expire, so if you create a VM, delete it, and recreate another VM with the
   same name in a short period of time, there will be two DHCP leases for the
   same host and its hostname will likely not resolve until the old lease
   expires.

3. The Operating System information database (osinfo-db) provides Operating
   System specific information needed to create guests for the various systems
   supported by `kvm-install-vm`.  The database files provided by your package
   manager may be out of date and not provide definitions for recent Operating
   System versions. If you encounter the following error message, you may need
   to update the database files:
   `ERR: Unknown OS variant '<name>'. Please update your osinfo-db.`
   If you have already updated your system, and the osinfo-db is still to old,
   then you can use the `osinfo-db-import` tool with the `--local` option, to
   install an up-to-date database in your home directory which will not
   conflict with your package manager files. The `osinfo-db-import` tool is
   provided by the rpm/deb packages `osinfo-db-tools`.
   See https://libosinfo.org/download for more information.

### Testing

Tests are written using [Bats](https://github.com/sstephenson/bats).  To
execute the tests, run `./test.sh` in the root directory of the project.

### Use Cases

If you don't need to use Docker or Vagrant, don't want to make changes to a
production machine, want to tinker with the kernel or systemd, or just want to spin up one or more VMs locally to test things like:

- high availability
- clustering
- package installs
- preparing for exams
- checking for system defaults
- anything else you would do with a VM

...then this wrapper could be useful for you.

### Troubleshooting

If you will encounter something similar:

```
ERR: Unknown OS variant 'fedora31'. Please update your osinfo-db.  See https://libosinfo.org/download for more information.
```

Then you need to update the DB in libosinfo.
Check the url and select the latest date ( https://releases.pagure.org/libosinfo/ )

```
wget -O "/tmp/osinfo-db.tar.xz" https://releases.pagure.org/libosinfo/osinfo-db-20200515.tar.xz
sudo osinfo-db-import --local "/tmp/osinfo-db.tar.xz"
```
