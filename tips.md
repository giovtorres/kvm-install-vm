# tips
nmcli device

## fedora 35
./kvm-install-vm create -c 4 -m 5120 -d 60 -T "Asia/Shanghai" -t fedora35 -D fedora35.my -v fedora35.my
./kvm-install-vm create -c 4 -m 5120 -d 50 -T "Asia/Shanghai" -t fedora35 -D fedora35.my -v fedora35.my

## fedora 34
./kvm-install-vm create -c 4 -m 5120 -d 80 -T "Asia/Shanghai" -t fedora34 -D fedora34.my -v fedora34.my
./kvm-install-vm create -c 4 -m 6144 -d 80 -T "Asia/Shanghai" -t fedora34 -D fedora34.my -v fedora34.my

virsh guestinfo fedora34.my
ssh-keygen -R fedora34.my
ssh fedora@fedora34.my

sudo dnf install -y langpacks-zh_CN
sudo localectl set-locale LANG=zh_CN.UTF-8
sudo dnf install -y cinnamon-desktop
sudo dnf -y groupinstall "Development Tools"

### activate sshd(启用sshd)
sudo rpm -qa | grep openssh-server
sudo systemctl status sshd
sudo systemctl enable sshd.service
sudo systemctl start sshd
sudo systemctl status sshd

---
#### no password login ( 无密码 ssh 密钥)
ssh-copy-id -i ~/.ssh/id_rsa.pub fedora@fedora34

### delete vps(删除vps)
./kvm-install-vm remove fedora35.my
