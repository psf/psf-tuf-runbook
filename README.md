psf-tuf-runbook
===============

This repository contains a runbook and supporting program for the Python Software Foundation's
TUF key generation and signing ceremonies.

**Due to COVID-19, this document has been modified for a two person, remote ceremony.**

## Notation

This document is designed to be read as a *runbook* -- a collection of discrete instructions
with remediation steps that, if followed correctly, should result in the intended effects.

We use the following notation:

* **DO** *actions*: Perform the following actions.
* **IF** *condition* **THEN** *actions*: If *condition* is met, then perform the following *actions*.
* **GO TO** *heading*: Go to the referenced heading in the runbook and perform the stated actions
thereon.
* **END**: You've reached an end state.

1. **DO GO TO** [Start](#start)

## Start

1. **DO GO TO** [Prepare the environment](#prepare-the-environment).

## Prepare the environment

1. **DO** perform the [pre-ceremony](PRE-CEREMONY.md).

1. **IF** you have a phone or other personal devices, **THEN** set them on airplane mode.

1. **DO** boot the trusted offline machine, and log into it using the credentials provided
during the pre-ceremony.

1. **DO** mount the flash storage stick:

```bash
$ sudo mount -t vfat /dev/sda1 /media/ceremony-products -o umask=000
```

1. **DO** take pictures of each HSM, in their tamper-evident bags.

1. **DO** remove `YubiHSM2-1` from its tamper-evident bag and **GO TO**
[Provisioning the Provisioning the YubiHSM 2](#provisioning-the-yubihsm-2)

1. **DO** remove `YubiHSM2-2` from its tamper-evident bag and **GO TO**
[Provisioning the Provisioning the YubiHSM 2](#provisioning-the-yubihsm-2)

1. **DO** remove `YubiHSM2-3` from its tamper-evident bag and **GO TO**
[Provisioning the Provisioning the YubiHSM 2](#provisioning-the-yubihsm-2)

1. **DO** remove `Nitrokey HSM-4` from its tamper-evident bag and **GO TO**
[Provisioning the Provisioning the Nitrokey HSM](#provisioning-the-nitrokey-hsm)

1. **DO** remove `Nitrokey HSM-5` from its tamper-evident bag and **GO TO**
[Provisioning the Provisioning the Nitrokey HSM](#provisioning-the-nitrokey-hsm)

1. **DO** remove `Nitrokey HSM-6` from its tamper-evident bag and **GO TO**
[Provisioning the Provisioning the Nitrokey HSM](#provisioning-the-nitrokey-hsm)

1. **DO** unmount the flash storage stick:

```bash
$ sync
$ sudo umount /media/ceremony-products
```

1. **END**

## Provisioning the YubiHSM 2

1. **DO** locate and write down the serial number printed on the YubiHSM 2. Refer to the picture below:

    ![A YubiHSM 2](./assets/yubihsm2.jpg)

    In this picture, the serial number is `7550054`. Note that in later steps the serial number will
    be 0-padded to 10 digits, like `0007550054`.

1. **DO** confirm the hash of the `./bin/yubihsm-provision` binary against the following checksums with
the following commands:

    * SHA1: **TODO**
    * SHA2-256: **TODO**

    ```bash
    $ shasum -a 1 ./bin/yubihsm-provision
    $ shasum -a 256 ./bin/yubihsm-provision
    ```

1. **IF** the YubiHSM 2 is being reprovisioned due to a compromise or failed ceremony, **THEN** you
must perform a physical reset.

    1. **DO** touch and hold the metal contact of the YubiHSM 2 for ten (10) seconds as you insert
    it into the trusted offline computer.

1. **IF** the YubiHSM 2 is being provisioned for the first time, **THEN** insert it into the trusted
offline computer.

1. **DO** ensure that exactly 1 (one) YubiHSM 2 is inserted into the trusted offline computer.

1. **DO** run the `./bin/yubihsm-provision` binary, using your key type according to the following rules:

    * **IF** your keytype is "P-256", **THEN** pass `--type p256`
    * **IF** your keytype is "P-384", **THEN** pass `--type p384`

    ```bash
    $ ./bin/yubihsm-provision --type KEY-TYPE
    ```

1. **DO** wait for this prompt:

    ```
    !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    !!!                    DANGER!                    !!!
    !!!                                               !!!
    !!!   This program will reset and reprovision     !!!
    !!!   your YubiHSM 2 for TUF purposes.            !!!
    !!!                                               !!!
    !!!   Make sure to read the runbook before        !!!
    !!!   using this program. Failure to do so        !!!
    !!!   will cause PERMANENT key loss.              !!!
    !!!                                               !!!
    !!!   Hit "y" (case insensitive) to continue.     !!!
    !!!                                               !!!
    !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    ```

1. **DO** hit `y` once ready to continue.

1. **DO** wait for the following output and prompt:

    ```
    Discovered a Yubico YubiHSM with serial number XXXXXXXXXX
    We've successfully authenticated with the HSM!
    Continue with factory reset? This step is IRREVERSIBLE! [y/N]
    ```

1. **DO** confirm that the serial number in the output matches the serial number written down.

1. **DO** hit `y` once ready to continue.

1. **DO** wait for the following output and prompt:

    ```
    Success! Giving the HSM 10 seconds to come back online...

    #####################################################
    ###                                               ###
    ###   We're going to create a new "auth key"      ###
    ###   on your YubiHSM.                            ###
    ###                                               ###
    ###   This "auth key" will                        ###
    ###   have a password that you MUST remember      ###
    ###   OR store securely and will protect the      ###
    ###   TUF keys that are going to be created.      ###
    ###                                               ###
    ###   Hit "y" (case insensitive) to continue.     ###
    ###                                               ###
    #####################################################
    ```

1. **DO** hit `y` once ready to continue.

1. **DO** enter your new authentication key password. This password should be long, random, and
unique.

1. **DO** re-enter your authentication key password.

1. **DO** wait for the following output:

    ```
    Success! Provisioned a new authentication key as object 2 and deleted the default key
    Success!
    We're creating our TUF keys and attestation certificates now.
    ```

1. **DO** re-enter your authentication key password.

1. **DO** wait for the program to exit.

1. **DO** write down your authentication key password on a *separate* piece of loose-leaf, and fold it.

1. **DO** check for the following files in the runbook directory:

    ```
    XXXXXXXXXX_cert.der
    XXXXXXXXXX_root_attestation.der
    XXXXXXXXXX_root_pubkey.pub
    XXXXXXXXXX_targets_attestation.der
    XXXXXXXXXX_targets_pubkey.pub
    ```

    Where `XXXXXXXXXX` is the 0-prefixed serial number.

1. **DO** run the `./bin/raw-ec-points-to-pem` script with each public key generated above, using your key type according to the following rules:

    * **IF** your keytype is "P-256", **THEN** pass `--type p256`
    * **IF** your keytype is "P-384", **THEN** pass `--type p384`

    ```bash
    $ ./bin/raw-ec-points-to-pem --type KEY-TYPE XXXXXXXXXX_root_pubkey.pub
    $ ./bin/raw-ec-points-to-pem --type KEY-TYPE XXXXXXXXXX_targets_pubkey.pub
    ```

1. **DO** remove the HSM.

1. **DO** seal the provisioned HSM and folded authentication key password in a tamper-evident bag.

1. **DO** label the bag with the HSM's signing body ID and 0-prefixed serial number.

1. **DO** run the following to copy the public ceremony products:

    ```bash
    $ cp XXXXXXXXXX_* /media/ceremony-products
    $ sync
    ```

    Where `XXXXXXXXXX` is the 0-prefixed serial number.

## Provisioning the Nitrokey HSM

1. **DO** determine the current Security Officer PIN ("SO-PIN"):

    1. **IF** the Nitrokey has not been provisioned before, **THEN** the SO-PIN is `3537363231383830`.

    1. **IF** the Nitrokey has been previously provisioned, **THEN** the SO-PIN should have been retained from the previous provisoning.

1. **DO** insert the Nitrokey HSM into the trusted offline computer.

1. **DO** ensure that exactly one (1) Nitrokey HSM is inserted into the trusted offline computer.

1. **DO** run the `./bin/nitrohsm-provision` script, using your SO-PIN:

    ```bash
    $ ./bin/nitrohsm-provision --so-pin SO-PIN
    ```

1. **DO** wait for this prompt:

    ```
    !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    !!!                    DANGER!                    !!!
    !!!                                               !!!
    !!!   This program will reset and reprovision     !!!
    !!!   your Nitrokey HSM for TUF purposes.         !!!
    !!!                                               !!!
    !!!   Make sure to read the runbook before        !!!
    !!!   using this program. Failure to do so        !!!
    !!!   will cause PERMANENT key loss and MAY       !!!
    !!!   leave your HSM in an unusable state.        !!!
    !!!                                               !!!
    !!!   Hit "y" (case insensitive) to continue.     !!!
    !!!                                               !!!
    !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    ```

1. **DO** hit `y` once ready to continue.

1. **DO** wait for the following output and prompt:

    ```
    Successfully discovered a Nitrokey HSM with Slot #0
    Continue with factory reset? This step is IRREVERSIBLE! [y/N]
    ```

1. **DO** hit `y` once ready to continue.

1. **DO** wait for the following output and prompt:

    ```
    Success! Reinitialized the HSM.
    This is your NEW Security Officer PIN: XXXXXXXXXXXXXXXX
    You MUST write this SO PIN down before continuing.
    Re-enter your NEW Security Officer PIN:
    ```

1. **DO** write down the *new* Security Officer PIN on a *separate* piece of loose-leaf, and fold it.

1. **DO** re-enter the *new* Security Officer PIN.

1. **DO** wait for the following output and prompt:

    ```
    This is your NEW user PIN: XXXXXX
    You MUST write this user PIN down before continuing.
    Re-enter your NEW user PIN:
    ```

1. **DO** write down the *new* user PIN on a *separate* piece of loose-leaf, and fold it.

1. **DO** re-enter the *new* user PIN.

1. **DO** wait for the following output:

    ```
    Success! We've reinitialized the Nitrokey with a new SO PIN and user PIN.
    Use this serial number when doing key generation: XXXXXXXXXXX
    ```

1. **DO** write down the serial number printed above on a *separate* piece of loose-leaf.

1. **DO** run the `./bin/generate-nitrohsm-keys` script, using your key type according to the following rules:

    * **IF** your keytype is "P-256", **THEN** pass `--type p256`
    * **IF** your keytype is "P-384", **THEN** pass `--type p384`

    ```bash
    $ ./bin/generate-nitrohsm-keys --type KEY-TYPE --pin USER-PIN --serial XXXXXXXXXXX
    ```

    where `USER-PIN` is your *new* user PIN and `XXXXXXXXXXX` is the Nitrokey HSM's serial
    number, as printed.

1. **DO** check for the following files in the runbook directory:

    ```
    XXXXXXXXXXX_root_pubkey.pub
    XXXXXXXXXXX_root_pubkey.pem
    XXXXXXXXXXX_targets_pubkey.pub
    XXXXXXXXXXX_targets_pubkey.pem
    ```

1. **DO** remove the HSM.

1. **DO** seal the provisioned HSM and folded Security Officer and user PINs in a tamper-evident bag.

1. **DO** label the bag with the HSM's signing body ID and serial number.

1. **DO** run the following to copy the ceremony products:

    ```bash
    $ cp XXXXXXXXXXX_* /media/ceremony-products
    $ sync
    ```

    Where `XXXXXXXXXXX` is the Nitrokey HSM's serial number.
