## kvm-install-vm

A bash wrapper around virt-install to build virtual machines on a local KVM
hypervisor.

### Usage

```
OPTIONS
  -c          Number of vCPUs     (default: 1)
  -m          Memory Size (MB)    (default: 1024)
  -d          Disk Size (GB)      (default: 10)
  -t          Linux Distribution  (default: centos7)
  -l          Location of Images  (default: /home/giovanni/virt/images)
  -k          SSH Public Key      (default: /home/giovanni/.ssh/id_rsa.pub)
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

Create VM with custom params:
  ./kvm-install-vm -c 2 -m 2048 -d 20 -n foo

Remove (destroy and undefine) a VM:
  ./kvm-install-vm -r foo
```

### Notes

- This will download a cloud image from the CentOS site if the default QCOW2
  image doesn't exist.

### Use Cases

If Docker or Vagrant don't do it for you and you just want to spin up one or
VMs quickly and locally to test things like:

- high availability
- clustering
- package installs
- preparing for exams
- checking for system defaults
- anything else you would do with a VM

then this wrapper could be useful for you.
