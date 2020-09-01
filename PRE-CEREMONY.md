Pre-ceremony action items
=========================

The following is a checklist of action items that **must** be completed
before the key signing and generation ceremonies may be attempted.

## Inventory

Confirm the presence of **each** of the following:

* **Exactly one (1)** preparation computer
  * The preparation steps below assume that the computer is running macOS
* **Exactly one (1)** communication computer
  * This *MAY* be the same as the preparation computer
* **Exactly one (1)** digital camera
  * This *MAY* be a smartphone in airplane mode.
* **Exactly one (1)** external monitor, HDMI compatible
* **Exactly one (1)** HDMI cable
* **Exactly one (1)** USB keyboard
* **Exactly one (1)** microUSB to USB-A cable
* **Exactly one (1)** wall socket to USB-A power source, minimum 5V/1A
* **Exactly one (1)** Raspberry Pi, Model 3
* **Exactly one (1)** microSD flash storage card, minimum 32GB
* **At least one (1)** microSD reader with USB interface
* **Exactly six (6)** HSMs:
  * **Exactly three (3)** YubiHSMs
  * **Exactly three (3)** Nitrokey HSMs
* **Exactly one (1)** USB flash storage stick, minimum 8GB
* **At least seven (7)** tamper-evident bags, including evidence labels
* **At least 12 (12)** sheets of loose-leaf paper
* **At least one (1)** permanent marker
* **At least one (1)** pair of scissors

Optionally, allow for **each** of the following:

* A USB-A extender

## Preparation

### Inspect the tamper-evident bags and markers

* Ensure that the tamper-evident bags are intact.

* Ensure that the markers are functional and can correctly mark the tamper-evident bags.

### Format the flash storage stick

On the preparation computer:

* Insert the flash storage stick.

* Open "Disk Utility" and identify the flash storage stick.

* Click "Unmount" if the flash storage stick is mounted.

* Click "Erase".

* Name the stick "TUF" and select `MS-DOS (FAT)` as the format.

* Click "Erase".

* Once formatting is complete, click "Unmount".

* Remove the flash storage stick from the preparation computer.

### Image and test the Raspberry Pi and peripherals

On the preparation computer:

* Download the prepared ceremony image, hosted [here](https://drive.google.com/file/d/1YJyOGuhXkOVXxXk4p70_s5u4plOBAAC8/view?usp=sharing).

* Confirm the integrity of the ceremony image archive:

    * SHA2-256: `24981cb828ffbdf804f4cf177a6179d64527570e9909299e269c3f9083d21fc6`

    ```bash
    $ shasum -a 256 runbook.img.zip
    ```

* Decompress the ceremony image:

    ```bash
    $ unzip runbook.img.zip
    ```

* Confirm the integrity of the ceremony image:

    * SHA2-256: `0bba7b046ef50df390af646b564b571077b5ceef9c9b03bb41089d9ed6751c32`

    ```bash
    $ shasum -a 256 runbook.img
    ```

* Insert the microSD card into the microSD reader, and insert the microSD reader into the
desktop.

* Identify the block device associated with the microSD card.

    ```bash
    $ diskutil list
    ```

The block device should have a path in the format `/dev/diskN`, where `N` is the device index.

We will use `/dev/rdiskN` below for the "raw" device, for performance reasons.

* Ensure that the block device is fully unmounted.

    ```bash
    $ diskutil unmountDisk /dev/rdiskN
    ```

* Copy the ceremony image onto the block device.

    ```bash
    sudo dd bs=4m if=runbook.img of=/dev/rdiskN
    ```

    where `/dev/rdiskN` is the "raw" block device identified above.

* Safely sync and eject the microSD card.

    ```bash
    $ sync
    $ diskutil eject /dev/rdiskN
    ```

* Insert the microSD card into the Raspberry Pi.

* Connect the Raspberry Pi to all peripherals **except** power and the flask stick.

* Connect the Raspberry Pi to power, and confirm boot on the monitor.

* Log into the Raspberry Pi on the prompt with the following credentials:

    Username: `pi`

    Password: `raspberry`

* Insert the flash storage stick into the Raspberry Pi.

* Identify the flash storage stick's device and confirm that it mounts:

    ```bash
    $ sudo mount -t vfat /dev/sda1 /media/ceremony-products -o umask=000
    $ sudo umount /media/ceremony-products
    ```

* Confirm the presence of the following programs, using `which`:

    ```bash
    $ which pkcs11-tool
    /usr/bin/pkcs11-tool
    $ which yubihsm-provision
    /home/pi/psf-tuf-runbook/bin/yubihsm-provision
    $ which nitrohsm-provision
    /home/pi/psf-tuf-runbook/bin/nitrohsm-provision
    ```

1. Confirm the hash of the `yubihsm-provision` binary against the following checksum:

    * SHA2-256: `f47e247f0739cf919baaef4079a1a8023adeb6c95957cfc8f3ca094dd4c80edf`

    ```bash
    $ shasum -a 256 $(which yubihsm-provision)
    ```

1. Confirm the hash of the `nitrohsm-provision` binary against the following checksum:

    * SHA2-256: `9d89fd46fb4ce71a9eb21b743a0ad68112c80f26e51f959120bc92a6f1d3806f`

    ```bash
    $ shasum -a 256 $(which nitrohsm-provision)
    ```

* Power the Raspberry Pi off and disconnect all peripherals **except** for the microSD card
and flash stick.

    ```bash
    $ sudo shutdown
    ```

* Store the Raspberry Pi and attached microSD card and flash stick in a tamper-evident bag.

### Test the ceremony communication computer

* Ensure that the communication computer's camera is functional.

* Ensure that the communication computer has internet access.

* Ensure that the communication computer's browser is up-to-date.
