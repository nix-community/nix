#!/usr/bin/env python

import pexpect
import subprocess
import sys

def log(msg):
    print("[prepare] " + msg)

# disk image and userdata images are passed as arguments
image = sys.argv[1]
userdata = sys.argv[2]

log("image={} userdata={}".format(image, userdata))

log("booting VM")

qemu = pexpect.spawn(
        "qemu-system-x86_64",
        [
            "-drive", "file={},format=qcow2".format(image),
            "-drive", "file={},format=qcow2".format(userdata),
            "-enable-kvm",
            "-m", "2G",
            "-nographic",
            "-serial", "mon:stdio",
            "-smp", "2",
            "-device", "rtl8139,netdev=net0",
            "-netdev", "user,id=net0,hostfwd=tcp:127.0.0.1:10022-:22",
        ],
        logfile = sys.stdout,
        encoding = 'utf8',
        timeout = 1000,
        )

# work around a bug in the image
qemu.expect(u"error: no such device: root.")
qemu.sendline("")

log("waiting on boot to finish")

qemu.expect(u"cloud-init.*finished at ")

log("logging in")

qemu.sendline("ubuntu")
qemu.expect(u"Password:")
qemu.sendline("ubuntu")
qemu.expect(u"ubuntu@ubuntu")

log("entering qemu menu")

qemu.sendcontrol("a")
qemu.send("c")

log("creating snapshot")

qemu.expect(u"\(qemu\)")
qemu.sendline("savevm prepare")

log("exiting")

qemu.expect(u"\(qemu\)")
qemu.sendline("quit")
qemu.wait()

log("FINISHED")
