## kvm-install-vm

A bash wrapper around virt-install to build virtual machines on a local KVM
hypervisor.  You can run it as a normal user which will use `qemu:///session` to
connect locally to your KVM domains.

Tested on the latest Fedora.

### Prerequisites

You need to have the KVM hypervisor installed, along with a few other packages (naming of packages can differ on other distributions):

- genisoimage or mkisofs
- virt-install
- libguestfs-tools-c
- qemu-img
- libvirt-client
- libosinfo

To install the dependencies, run:

- Fedora example:

```
sudo dnf -y install genisoimage virt-install libguestfs-tools-c qemu-img libvirt-client wget libosinfo
```

- Ubuntu example:

```
sudo apt install -y genisoimage virtinst libguestfs-tools qemu-utils libvirt-clients wget libosinfo-bin
```

If you want to resolve guests by their hostnames, install the `libvirt-nss` package:

- Fedora example:

```
sudo dnf -y install libvirt-nss
```

- Ubuntu example:

```
sudo apt install -y libnss-libvirt
```

Then, add `libvirt` and `libvirt_guest` to list of **hosts** databases in
`/etc/nsswitch.conf`.  See [here](https://libvirt.org/nss.html) for more
information.

### Usage

```
$ kvm-install-vm help
NAME
    kvm-install-vm - Install virtual guests using cloud-init on a local KVM
    hypervisor.

SYNOPSIS
    kvm-install-vm COMMAND [OPTIONS]

DESCRIPTION
    A bash wrapper around virt-install to build virtual machines on a local KVM
    hypervisor. You can run it as a normal user which will use qemu:///session
    to connect locally to your KVM domains.

COMMANDS
    help    - show this help or help for a subcommand
    create  - create a new guest domain
    list    - list all domains, running and stopped
    remove  - delete a guest domain
```

#### Creating Guest VMs

```
$ kvm-install-vm help create
NAME
    kvm-install-vm create [COMMANDS] [OPTIONS] VMNAME

DESCRIPTION
    Create a new guest domain.

COMMANDS
    help - show this help

OPTIONS
    -a          Autostart             (default: false)
    -b          Bridge                (default: virbr0)
    -c          Number of vCPUs       (default: 1)
    -d          Disk Size (GB)        (default: 10)
    -D          DNS Domain            (default: example.local)
    -f          CPU Model / Feature   (default: host)
    -g          Graphics type         (default: spice)
    -h          Display help
    -i          Custom QCOW2 Image
    -k          SSH Public Key        (default: $HOME/.ssh/id_rsa.pub)
    -l          Location of Images    (default: $HOME/virt/images)
    -L          Location of VMs       (default: $HOME/virt/vms)
    -m          Memory Size (MB)      (default: 1024)
    -M          Mac address           (default: auto-assigned)
    -p          Console port          (default: auto)
    -s          Custom shell script
    -t          Linux Distribution    (default: centos8)
    -T          Timezone              (default: US/Eastern)
    -u          Custom user           (default: $USER)
    -y          Assume yes to prompts (default: false)
    -n          Assume no to prompts  (default: false)
    -v          Be verbose

DISTRIBUTIONS
    NAME            DESCRIPTION                         LOGIN
    amazon2         Amazon Linux 2                      ec2-user
    centos8         CentOS 8                            centos
    centos7         CentOS 7                            centos
    centos7-atomic  CentOS 7 Atomic Host                centos
    centos6         CentOS 6                            centos
    debian9         Debian 9 (Stretch)                  debian
    debian10        Debian 10 (Buster)                  debian
    fedora29        Fedora 29                           fedora
    fedora29-atomic Fedora 29 Atomic Host               fedora
    fedora30        Fedora 30                           fedora
    fedora31        Fedora 31                           fedora
    fedora32        Fedora 32                           fedora
    opensuse15      OpenSUSE Leap 15.2                  opensuse
    ubuntu1604      Ubuntu 16.04 LTS (Xenial Xerus)     ubuntu
    ubuntu1804      Ubuntu 18.04 LTS (Bionic Beaver)    ubuntu
    ubuntu2004      Ubuntu 20.04 LTS (Focal Fossa)      ubuntu

EXAMPLES
    kvm-install-vm create foo
        Create VM with the default parameters: CentOS 8, 1 vCPU, 1GB RAM, 10GB
        disk capacity.

    kvm-install-vm create -c 2 -m 2048 -d 20 foo
        Create VM with custom parameters: 2 vCPUs, 2GB RAM, and 20GB disk
        capacity.

    kvm-install-vm create -t debian9 foo
        Create a Debian 9 VM with the default parameters.

    kvm-install-vm create -T UTC foo
        Create a default VM with UTC timezone.
```

#### Deleting a Guest Domain

```
$ kvm-install-vm help remove
NAME
    kvm-install-vm remove [COMMANDS] VMNAME

DESCRIPTION
    Destroys (stops) and undefines a guest domain.  This also remove the
    associated storage pool.

COMMANDS
    help - show this help

EXAMPLE
    kvm-install-vm remove foo
        Remove (destroy and undefine) a guest domain.  WARNING: This will
        delete the guest domain and any changes made inside it!
```

#### Attaching a new disk

```
$ kvm-install-vm help attach-disk
NAME
    kvm-install-vm attach-disk [OPTIONS] [COMMANDS] VMNAME

DESCRIPTION
    Attaches a new disk to a guest domain.

COMMANDS
    help - show this help

OPTIONS
    -d SIZE     Disk size (GB)
    -f FORMAT   Disk image format       (default: qcow2)
    -s IMAGE    Source of disk device
    -t TARGET   Disk device target

EXAMPLE
    kvm-install-vm attach-disk -d 10 -s example-5g.qcow2 -t vdb foo
        Attach a 10GB disk device named example-5g.qcow2 to the foo guest
        domain.
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
production machine, or just want to spin up one or more VMs locally to test
things like:

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
