# kvm-install-vm

A Bash wrapper around `virt-install` to quickly spin up and manage local KVM virtual machines.

## Table of Contents

- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
   - [Create a VM](#create-a-vm)
   - [Delete a VM](#delete-a-vm)
   - [Attach a Disk](#attach-a-disk)
- [Configuration](#configuration)
- [Boot Mode](#boot-mode)
- [Hostname Resolution (optional)](#hostname-resolution-optional)
- [Troubleshooting](#troubleshooting)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

## ✨ Features

- 🚀 One-command VM provisioning with sensible defaults
- 🌐 Support for multiple distro cloud-images (AlmaLinux, Debian, Rocky, Ubuntu, and more)
- 🔧 Customize CPU, RAM, disk size, architecture, and timezone
- 💾 Attach additional disks on the fly
- 🖥️ Boot using BIOS or UEFI (SecureBoot enabled or disabled)
- ⚙️ Persist your favorite defaults via `~/.kivrc`

## 🔧 Prerequisites

You’ll need a working KVM/libvirt setup and these utilities:

<details>
<summary>Install on Fedora</summary>

```bash
sudo dnf install -y \
  virt-install \
  libguestfs-tools-c \
  qemu-img \
  libvirt-client \
  libosinfo \
  wget
```

</details>

<details>
<summary>Install on Ubuntu/Debian</summary>

```bash
sudo apt update && sudo apt install -y \
  virtinst \
  libguestfs-tools \
  qemu-utils \
  libvirt-clients \
  libosinfo-bin \
  wget
```

</details>

## 🚀 Quick Start

**Create your first VM**

```bash
# Rocky Linux 9 VM named "myvm" with defaults
./kvm-install-vm create myvm
```

Connect via `ssh myvm` once cloud-init finishes. Run the help command to see all options:

```bash
kvm-install-vm help
```

## Installation

Copy the script to a folder in your PATH, and the config file to your HOME directory:

```bash
cp kvm-install-vm $HOME/local/bin/
cp .kivrc $HOME
```

Optionally, you may instead copy the script globally:

```bash
sudo cp kvm-install-vm /usr/local/bin
```

## 💻 Usage

### Create a VM
```bash
kvm-install-vm create [OPTIONS] <VM_NAME>
```

```bash
# Defaults: Rocky Linux 9, 1 vCPU, 1536 MB RAM, 10 GB disk, x86_64
kvm-install-vm create myvm
```

**Common flags:**
- `-c, --cpus <N>`       Set vCPUs (default: 1)
- `-m, --memory <MB>`    RAM in MB (default: 1536)
- `-d, --disk <GB>`      Disk size in GB (default: 10)
- `-A, --arch <ARCH>`    Architecture (x86_64 or aarch64)
- `-t, --distro <NAME>`  Distro key (see BUILTIN_VMS)
- `-T, --tz <ZONE>`      Timezone (default: host timezone)

### Delete a VM

```bash
# Destroys domain and deletes its storage!
kvm-install-vm remove myvm
```

### Attach a Disk

```bash
kvm-install-vm attach-disk \
  -d <GB> \
  -s <FILENAME>.qcow2 \
  -t <DEVICE_NAME> \
  <VM_NAME>
```

## ⚙️ Configuration

You can override defaults by creating a `~/.kivrc` file:

```bash
# Example ~/.kivrc
# Default vCPUs and RAM
VCPUS=2
MEMORY=2048
```

Order of precedence:

1. Built‑in script defaults
2. `.kivrc` settings
3. Command‑line flags

### Using other images

Use the BUILTIN_VMS array in `.kivrc` to add new images:

```bash
BUILTIN_VMS+=("almalinux9:AlmaLinux 9 cloud image:x86_64:https://repo.almalinux.org/almalinux/9/cloud/images/AlmaLinux-9-cloud.qcow2|almalinux")
```

You may add multiple lines to include multiple VMs. See `.kivrc` for more details.

## 🖥️ Boot mode

By default, if the [EDK2 OVMF](https://github.com/tianocore/tianocore.github.io/wiki/OVMF) package is detected on the host, `virt-install` will boot using UEFI with secure boot disabled.

With the `-S` flag, you can enable secure boot when creating a virtual machine.

To confirm secure boot is enabled, use `mokutil` inside the virtual machine:

```bash
[root@rocky ~]# mokutil --sb-state
SecureBoot enabled
```

## 🌐 Hostname Resolution (optional)

To resolve your VMs by hostname, install the `libvirt-nss` plugin and update `/etc/nsswitch.conf`:

```bash
# Fedora
sudo dnf install -y libvirt-nss
# Ubuntu/Debian
sudo apt install -y libnss-libvirt
```

Then add `libvirt` and `libvirt_guest` to the hosts line in `/etc/nsswitch.conf`:

```
hosts: files mdns4_minimal [NOTFOUND=return] libvirt libvirt_guest dns myhostname
```

See the [libvirt NSS docs](https://libvirt.org/nss.html) for details.

## 🐞 Troubleshooting

- **Unknown OS variant**:

  ```
  ERR: Unknown OS variant '<name>'. Please update your osinfo-db.
  ```

  Download the latest database:

  ```bash
  wget -O /tmp/osinfo-db.tar.xz https://releases.pagure.org/libosinfo/osinfo-db-$(date +%Y%m%d).tar.xz
  osinfo-db-import --local /tmp/osinfo-db.tar.xz
  ```

- **DHCP hostname collisions**:\
  If you recreate a VM with the same name quickly, an old lease may linger—wait for it to expire or clear `/var/lib/libvirt/dnsmasq/*.leases`.

## 🧪 Testing

Tests are powered by [Bats](https://github.com/bats-core/bats-core).\
Run:

```bash
./test.sh
```

## 🤝 Contributing

1. Fork the repo
2. Create a feature branch
3. Open a pull request

Please follow the existing code style and add tests where possible.

## 📄 License

This project is licensed under the [MIT License](LICENSE).
