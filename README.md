## kvm-install-vm

A bash wrapper around virt-install to build virtual machines on a local KVM
hypervisor.  You can run it as a normal user which will use `qemu:///session` to
connect locally to your KVM domains.

Tested on Fedora 25/26.

### Prerequisites

You need to have the KVM hypervisor installed, along with a few other packages:

- genisoimage
- virt-install
- libguestfs-tools-c
- qemu-img
- libvirt-client

To install the dependencies, run:

```
sudo dnf -y install genisoimage virt-install libguestfs-tools-c qemu-img libvirt-client wget
```

### Usage

```
NAME
    kvm-install-vm - Install virtual guests using cloud-init on a local KVM
    hypervisor.

SYNOPSIS
    ./kvm-install-vm [OPTIONS] -n|-r vmname

DESCRIPTION
    A bash wrapper around virt-install to build virtual machines on a local KVM
    hypervisor. You can run it as a normal user which will use qemu:///session
    to connect locally to your KVM domains.

MANDATORY ARGUMENTS
    You must specify one of the following arguments to either create or delete
    a VM:
        -n vmname   Name of VM to create
        -r vmname   Name of VM to delete

OPTIONS
    -b          Bridge              (default: virbr0)
    -c          Number of vCPUs     (default: 1)
    -d          Disk Size (GB)      (default: 10)
    -f          CPU Model / Feature (default: host)
    -h          Display help
    -i          Custom QCOW2 Image
    -k          SSH Public Key      (default: $HOME/.ssh/id_rsa.pub)
    -l          Location of Images  (default: $HOME/virt/images)
    -m          Memory Size (MB)    (default: 1024)
    -M mac      Mac address         (default: auto-assigned)
    -t          Linux Distribution  (default: centos7)
    -T          Timezone            (default: US/Eastern)


DISTRIBUTIONS
    NAME            DESCRIPTION                         LOGIN
    centos7         CentOS 7                            centos
    centos7-atomic  CentOS 7 Atomic Host                centos
    centos6         CentOS 6                            centos
    debian9         Debian 9 (Stretch)                  debian
    fedora26        Fedora 26                           fedora
    fedora26-atomic Fedora 26 Atomic Host               fedora
    ubuntu1604      Ubuntu 16.04 LTS (Xenial Xerus)     ubuntu

EXAMPLES
    ./kvm-install-vm -n foo
        Create VM with the default parameters: CentOS 7, 1 vCPU, 1GB RAM, 10GB
        disk capacity.

    ./kvm-install-vm -c 2 -m 2048 -d 20 -n foo
        Create VM with custom parameters: 2 vCPUs, 2GB RAM, and 20GB disk
        capacity.

    ./kvm-install-vm -t debian9 -n foo
        Create a Debian 9 VM with the default parameters.

    ./kvm-install-vm -r foo
        Remove (destroy and undefine) a VM.  WARNING: This will delete all
        customizations in the VM!
```

### Example

[![asciicast](https://asciinema.org/a/bVgjJ3SHgvROX90iRuCCF1h4d.png)](https://asciinema.org/a/bVgjJ3SHgvROX90iRuCCF1h4d)

### Notes

1. This script will download a qcow2 cloud image from the respective
   distribution's download site.  See script for URLs.

2. To resolve guest's by their hostnames, install the `libvirt-nss` package and
   configure `nsswitch.conf`.  See [here](https://libvirt.org/nss.html) for
   more information.

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
