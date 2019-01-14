## kvm-install-vm

A bash wrapper around virt-install to build virtual machines on a local KVM
hypervisor.  You can run it as a normal user which will use `qemu:///session` to
connect locally to your KVM domains.

Tested on the latest Fedora.

### Prerequisites

You need to have the KVM hypervisor installed, along with a few other packages:

- genisoimage or mkisofs
- virt-install
- libguestfs-tools-c
- qemu-img
- libvirt-client
- jq

To install the dependencies, run:

```
sudo dnf -y install genisoimage virt-install libguestfs-tools-c qemu-img libvirt-client wget jq
```

If you want to resolve guests by their hostnames, install the `libvirt-nss` package:

```
sudo dnf -y install libvirt-nss
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
    help        - show this help or help for a subcommand
    attach-disk - create and attach a disk device to guest domain
    create      - create a new guest domain
    detach-disk - detach a disk device from a guest domain
    list        - list all domains, running and stopped
    remove      - delete a guest domain
    net-create  - create and activate a network
    net-remove  - remove and undefine a network
    net-list    - list all networks
    deploy      - deploy virtual networks and machine from a configuration file input
```

#### Creating Guest VMs

```
$ kvm-install-vm help create
NAME
    kvm-install-vm create [OPTIONS] VMNAME

DESCRIPTION
    Create a new guest domain.

OPTIONS
    -a          Autostart           (default: false)
    -b          Bridge              (default: virbr0)
    -c          Number of vCPUs     (default: 1)
    -d          Disk Size (GB)      (default: 10)
    -D          DNS Domain          (default: example.local)
    -f          CPU Model / Feature (default: host)
    -g          Graphics type       (default: spice)
    -h          Display help
    -i          Custom QCOW2 Image
    -k          SSH Public Key      (default: $HOME/.ssh/id_rsa.pub)
    -l          Location of Images  (default: $HOME/virt/images)
    -L          Location of VMs     (default: $HOME/virt/vms)
    -m          Memory Size (MB)    (default: 1024)
    -M          Mac address         (default: auto-assigned)
    -p          Console port        (default: auto)
    -s          Custom shell script
    -t          Linux Distribution  (default: centos7)
    -T          Timezone            (default: US/Eastern)
    -u          Custom user         (defualt: $USER)
    -v          Be verbose

DISTRIBUTIONS
    NAME            DESCRIPTION                         LOGIN
    amazon2         Amazon Linux 2                      ec2-user
    centos7         CentOS 7                            centos
    centos7-atomic  CentOS 7 Atomic Host                centos
    centos6         CentOS 6                            centos
    debian9         Debian 9 (Stretch)                  debian
    fedora27        Fedora 27                           fedora
    fedora27-atomic Fedora 27 Atomic Host               fedora
    fedora28        Fedora 28                           fedora
    fedora28-atomic Fedora 28 Atomic Host               fedora
    ubuntu1604      Ubuntu 16.04 LTS (Xenial Xerus)     ubuntu
    ubuntu1804      Ubuntu 18.04 LTS (Bionic Beaver)    ubuntu

EXAMPLES
    kvm-install-vm create foo
        Create VM with the default parameters: CentOS 7, 1 vCPU, 1GB RAM, 10GB
        disk capacity.

    kvm-install-vm create -c 2 -m 2048 -d 20 foo
        Create VM with custom parameters: 2 vCPUs, 2GB RAM, and 20GB disk
        capacity.

    kvm-install-vm create -t debian9 foo
        Create a Debian 9 VM with the default parameters.

    kvm-install-vm create -T UTC foo
        Create a default VM with UTC timezone.

    kvm-install-vm create -s ~/script.sh -g vnc -u bar foo
        Create a VM with a custom script included in user-data, a graphical
        console accessible over VNC, and a user named 'bar'.
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

### Create a virtual network
```
$ ./kvm-install-vm net-create help
NAME
    kvm-install-vm net-create

DESCRIPTION
    Create a new network.

COMMANDS
    help - show this help

OPTIONS
    -a          Autostart           (default: false)
    -b          Bridge              (default: virbr0)
    -n          Network             (default: default)
    -s          DHCP start address  (default: 10.0.0.2)
    -e          DHCP end address    (default: 10.0.0.254)
    -M          Mac address         (default: auto-assigned)
    -A          IP address          (default: 10.0.0.1)
    -m          Netmask             (default: 255.255.255.0)
    -f          Forwarding mode     (default: nat)
    -d          Interface device    (default: none)

FORWARDING MODES
    nat
    routed
    isolated

EXAMPLE
    kvm-install-vm net-create
        Creates a default network
```

### Removing a virtual network
```
$ ./kvm-install-vm net-remove help
NAME
    kvm-install-vm net-remove

DESCRIPTION
    Removes and undefines a network.

COMMANDS
    help - show this help

EXAMPLE
    kvm-install-vm net-remove default
        Removes and undefines network default
```

### Deploying virtual networks and guest domains from a configuration file
```
$ ./kvm-install-vm deploy help
NAME
    kvm-install-vm deploy

DESCRIPTION
    Deploy virtual networks and machine from a configuration file.

COMMANDS
    help - show this help

OPTIONS
    -f          filename (default deployment.json)

EXAMPLE
    kvm-install-vm deploy -f deployment.json
        Deploys all virtual networks and machines defined in the configuration file.
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
