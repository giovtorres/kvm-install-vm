[0;1;32m●[0m systemd-zram-setup@zram1.service - Create swap on /dev/zram1
     Loaded: loaded (]8;;file://fedora/usr/lib/systemd/system/systemd-zram-setup@.service/usr/lib/systemd/system/systemd-zram-setup@.service]8;;; static)
    Drop-In: /run/systemd/generator/systemd-zram-setup@zram1.service.d
             └─]8;;file://fedora/run/systemd/generator/systemd-zram-setup@zram1.service.d/bindings.confbindings.conf]8;;
     Active: [0;1;32mactive (exited)[0m since Sat 2022-10-29 11:32:43 WITA; 2h 31min ago
       Docs: ]8;;man:zram-generator(8)man:zram-generator(8)]8;;
             ]8;;man:zram-generator.conf(5)man:zram-generator.conf(5)]8;;
    Process: 669 ExecStart=/usr/lib/systemd/system-generators/zram-generator --setup-device zram1 (code=exited, status=0/SUCCESS)
   Main PID: 669 (code=exited, status=0/SUCCESS)
        CPU: 153ms

10月 29 11:32:43 fedora systemd[1]: Starting systemd-zram-setup@zram1.service - Create swap on /dev/zram1...
10月 29 11:32:43 fedora systemd-makefs[672]: /dev/zram1 successfully formatted as ext2 (label "zram1", uuid cc24b691-90ca-488d-a9d7-aaf8138fcaab)
10月 29 11:32:43 fedora systemd[1]: Finished systemd-zram-setup@zram1.service - Create swap on /dev/zram1.
