Pre-ceremony action items
=========================

The following is a checklist of action items that **must** be completed
before the key signing and generation ceremonies may be attempted.

## Inventory

Confirm the presence of **each** of the following:

* **Exactly one (1)** preparation desktop
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
* **At least seven (7)** tamper-evident bags, including evidence labels
* **At least one (1)** permanent marker

## Preparation

### Inspect the tamper-evident bags and markets

* Ensure that the tamper-evident bags are intact.

* Ensure that the markers are functional and can correctly mark the tamper-evident bags.

### Image and test the Raspberry Pi and peripherals

On a desktop:

* Download the prepared ceremony image:

```bash
$ curl -o ceremony.iso TODO-URL
```

* Confirm the integrity of the ceremony image against this checksum: `TODO-CHECKSUM`:

```bash
$ shasum -a 256 ceremony.iso
```

* Insert the microSD card into the microSD reader, and insert the microSD reader into the
desktop.

* Identify the block device associated with the microSD card.

On Linux, use `lsblk`:

On macOS, use `diskutil list`:

* Copy the ceremony image onto the block device.

```bash
sudo dd bs=1M if=ceremony.iso of=BLOCK-DEVICE-PATH
```

where `BLOCK-DEVICE-PATH` is the block device identified above.

<!-- TODO(ww): Eject/sync. -->

* Safely sync and eject the microSD card.

* Insert the microSD card into the Raspberry Pi.

* Connect the Raspberry Pi to all peripherals **except** power.

* Connect the Raspberry Pi to power, and confirm POST.

* Log into the Raspberry Pi on the prompt with the following credentials:

Username: `pi`

Password: `raspberry`

* Confirm the presence of the following binaries:

* Power the Raspberry Pi off and disconnect all peripherals **except** for the microSD card.

* Store the Raspberry Pi and attached microSD card in a tamper-evident bag.
