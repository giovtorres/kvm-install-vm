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
OPTIONS
  -c          Number of vCPUs     (default: 1)
  -m          Memory Size (MB)    (default: 1024)
  -d          Disk Size (GB)      (default: 10)
  -t          Linux Distribution  (default: centos7)
  -l          Location of Images  (default: $HOME/virt/images)
  -k          SSH Public Key      (default: $HOME/.ssh/id_rsa.pub)
  -b          Bridge              (default: virbr0)
  -h          Display help
  -i          Custom QCOW2 Image
  -n vmname   Name of VM to create
  -r vmname   Name of VM to delete

DISTRIBUTIONS
 - centos7
 - centos6

EXAMPLES

Create VM with default params:
  ./kvm-install-vm -n foo

Create VM with custom params (2 vCPUs, 2048MB RAM and 20GB disk):
  ./kvm-install-vm -c 2 -m 2048 -d 20 -n foo

Remove (destroy and undefine) a VM:
  ./kvm-install-vm -r foo
```

### Notes

- This will download a cloud image from the CentOS site if the default QCOW2
  image doesn't exist.

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
