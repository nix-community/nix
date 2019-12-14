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

# copy the images to work on them
subprocess.check_call(["cp", "--reflink=auto", image, "disk.qcow2"])
subprocess.check_call(["cp", "--reflink=auto", userdata, "userdata.qcow2"])
subprocess.check_call(["chmod", "+w", "disk.qcow2", "userdata.qcow2"])

# Make some room on the root image
subprocess.check_call(["qemu-img", "resize", "disk.qcow2", "+64G"])

log("booting VM")

qemu = pexpect.spawn(
        "qemu-system-x86_64",
        [
            "-drive", "file=disk.qcow2,format=qcow2",
            "-drive", "file=userdata.qcow2,format=qcow2",
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

log("AAA")

qemu.expect(u"cloud-init.*finished at ")
qemu.sendcontrol("a")
qemu.send("c")

log("BBB")

qemu.expect(u"\(qemu\)")
qemu.sendline("savevm prepare")

log("CCC")

qemu.expect(u"\(qemu\)")
qemu.sendline("quit")

log("DDD")

qemu.wait()

log("FINISHED")
