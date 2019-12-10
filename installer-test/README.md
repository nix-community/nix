# Usage

For now all that we have a is a Ubuntu VM to test the installer manually.
Automated tests come next.

Run ./wootbuntu to start the VM. It will automatically download the ISO and
setup QEMU. Make sure to have KVM enabled on your machine for it to be fast
(/dev/kvm should exist on the host).

You should see the following error[1]:

    error: no such device: root.

Wait 30 seconds and it should continue to boot. Sometimes you need to type
Enter to continue.

# Login

Next comes the login session. Use these credentials:

username: ubuntu
password: ubuntu

# Run the installer

The nix folder is mounted read-only under /mnt. Build the installer on the
host and then run the installer on the guest.

# Shutdown

Go to another terminal and run `pkill qemu`. This is not ideal..

# Reset the VM

Delete the `disk.qcow2` file and re-start the VM.

# TODO:

* Figure out how to shutdown the VM nicely.


[1]: https://bugs.launchpad.net/cloud-images/+bug/1726476
