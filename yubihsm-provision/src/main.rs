use dialoguer::{Confirmation, PasswordInput};
use yubihsm::asymmetric::Algorithm;
use yubihsm::attestation::Certificate;
use yubihsm::authentication::key::Key;
use yubihsm::authentication::{Algorithm as AuthAlg, DEFAULT_AUTHENTICATION_KEY_ID};
use yubihsm::capability::Capability;
use yubihsm::client::Client;
use yubihsm::connector::usb::{Devices, UsbTimeout};
use yubihsm::connector::Connector;
use yubihsm::domain::Domain;
use yubihsm::ecdsa::curve::NistP384;
use yubihsm::object::{Id, Label, Type};
use yubihsm::{Credentials, UsbConfig};

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process;
use std::{thread, time};

const TUF_ROOT_KEY_ID: Id = 3;
const TUF_TARGETS_KEY_ID: Id = 4;

// The suffix for the file that we'll write the YubiHSM's internal attestation
// certificate to. The ultimate path will be of the form XXXXXXXXXX_cert.der,
// where XXXXXXXXXX is the 0-padded serial number of the HSM.
const YUBIHSM_ATTESTATION_CERT_SUFFIX: &'static str = "cert.der";

// The suffix for the file that we'll write the root keypair's attestation certificate to.
// This will have the same ultimate path format as the internal attestation path.
const TUF_ROOT_KEY_ATTESTATION_FILE_SUFFIX: &'static str = "root_attestation.der";

// The suffix for the file that we'll write the root keypair's public key to.
// This will have the same ultimate path format as the internal attestation path.
const TUF_ROOT_KEY_PUBKEY_FILE_SUFFIX: &'static str = "root_pubkey.pub";

// The suffix for the file that we'll write the targets keypair's attestation certificate to.
// This will have the same ultimate path format as the internal attestation path.
const TUF_TARGETS_KEY_ATTESTATION_FILE_SUFFIX: &'static str = "targets_attestation.der";

// The suffix for the file that we'll write the targets keypair's public key to.
// This will have the same ultimate path format as the internal attestation path.
const TUF_TARGETS_KEY_PUBKEY_FILE_SUFFIX: &'static str = "targets_pubkey.pub";

const HSM_USB_TIMEOUT: u64 = 10;

const BIG_SCARY_BANNER: &'static str = r#"
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
"#;

const NEW_AUTH_KEY_MESSAGE: &'static str = r#"
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
"#;

fn confirm(msg: &str) -> Result<(), String> {
    match Confirmation::new().with_text(msg).default(false).interact() {
        Ok(true) => Ok(()),
        Ok(false) => Err(String::from("user interrupted provisioning")),
        Err(e) => Err(format!("prompt error: {}", e)),
    }
}

fn big_scary_banner() -> Result<(), String> {
    println!("{}", BIG_SCARY_BANNER);
    confirm("Continue?")
}

fn find_hsm() -> Result<UsbConfig, String> {
    let devices = match Devices::detect(UsbTimeout::from_secs(HSM_USB_TIMEOUT)) {
        Ok(ds) => ds,
        Err(e) => return Err(format!("HSM detection error: {}", e)),
    };

    let device = match devices.len() {
        1 => &devices.as_slice()[0],
        0 => return Err(String::from("no YubiHSMs detected")),
        _ => {
            return Err(format!(
                "more than one YubiHSM detected; refusing to continue"
            ))
        }
    };

    println!(
        "Discovered a {} with serial number {}",
        device.product_name, device.serial_number
    );

    Ok(UsbConfig {
        serial: Some(device.serial_number),
        timeout_ms: HSM_USB_TIMEOUT * 1000,
    })
}

fn file_presence_checks(serial_number: &str) -> Result<(), String> {
    for suffix in vec![
        TUF_ROOT_KEY_ATTESTATION_FILE_SUFFIX,
        TUF_TARGETS_KEY_ATTESTATION_FILE_SUFFIX,
    ] {
        let file = format!("{}_{}", serial_number, suffix);
        if Path::new(&file).exists() {
            return Err(format!(
                "Attestation file already exists: {}; aborting",
                file
            ));
        }
    }

    Ok(())
}

fn open_hsm_default_creds(connector: Connector) -> Result<Client, String> {
    // NOTE(ww): We assume here that the YubiHSM being provisioned still
    // has its default authentication key. If this isn't the case,
    // the user can physically perform a reset by pressing the metal contact
    // of the HSM for 10 seconds while inserting and then continue with
    // provisioning.

    let credentials = Credentials::default();
    match Client::open(connector, credentials, true) {
        Ok(c) => Ok(c),
        Err(e) => Err(format!(
            "unable to open a client connection with the HSM: {}; try a physical reset",
            e
        )),
    }
}

fn open_hsm(connector: Connector, credentials: Credentials) -> Result<Client, String> {
    match Client::open(connector, credentials, true) {
        Ok(c) => Ok(c),
        Err(e) => Err(format!(
            "unable to open a client connection with the HSM: {}; try a physical reset",
            e
        )),
    }
}

fn perform_factory_reset(usb_config: &UsbConfig) -> Result<(), String> {
    let connector = Connector::usb(usb_config);
    let client = open_hsm_default_creds(connector)?;

    println!("We've successfully authenticated with the HSM!");
    confirm("Continue with factory reset? This step is IRREVERSIBLE!")?;
    match client.reset_device() {
        Ok(()) => Ok(()),
        Err(e) => return Err(format!("reset failed: {}; try a physical reset", e)),
    }
}

fn new_auth_key(usb_config: &UsbConfig) -> Result<Id, String> {
    let connector = Connector::usb(usb_config);
    let mut client = open_hsm_default_creds(connector.clone())?;

    println!("{}", NEW_AUTH_KEY_MESSAGE);
    confirm("Continue?")?;

    let password = match PasswordInput::new()
        .with_prompt("Authentication key password")
        .interact()
    {
        Ok(password) => password,
        Err(e) => return Err(format!("prompt failed: {}", e)),
    };

    let confirm_password = match PasswordInput::new()
        .with_prompt("Authentication key password")
        .interact()
    {
        Ok(password) => password,
        Err(e) => return Err(format!("prompt failed: {}", e)),
    };

    if password != confirm_password {
        return Err(String::from("supplied passwords don't match!"));
    }

    // These are the permissions that our new auth key will be given.
    // In detail:
    //   * GET_OPAQUE: Allows sessions under this key to retrieve opaque
    //     HSM-stored data. In particular, this allows us to retrieve the opaque
    //     built-in attestation key for attestation signing.
    //   * GENERATE_ASYMMETRIC_KEY: Allows sessions under this key to generate
    //     asymmetric keypairs, which we'll need to generate our keys.
    //   * SIGN_ECDSA: Allows sessions under this key to create digital signatures with
    //     available (EC) keys.
    //   * SIGN_ATTESTATION_CERTIFICATE: Allows sessions under this key to
    //     generate x509 certificates that attest to the HSM's possession
    //     of a private key.
    //   * DELETE_AUTHENTICATION_KEY: Allows sessions under this key to
    //     delete authentication keys, which we'll use to delete the
    //     factory default authentication key.
    let auth_key_caps = Capability::GET_OPAQUE
        | Capability::GENERATE_ASYMMETRIC_KEY
        // NOTE: This needs to be replaced with SIGN_EDDSA once attestation
        // of Ed25519 keys is figured out.
        | Capability::SIGN_ECDSA
        | Capability::SIGN_ATTESTATION_CERTIFICATE
        | Capability::DELETE_AUTHENTICATION_KEY;

    let key_id = match client.put_authentication_key(
        // This is the object ID of the authentication key being created.
        // Since we're performing this operation right after a factory reset,
        // ID #2 should be available for use. ID #1 is currently in use as the
        // default authentication key.
        2_u16,
        // This is the label associated with our authentication key.
        // NOTE: This unwrap is safe, since the "tuf-authkey" literal is under 40 bytes.
        Label::from_bytes(b"tuf-authkey").unwrap(),
        // This is the set of domains associated with our authentication key.
        // We don't make use of the YubiHSM's domain feature, so we always set this
        // to DOM1.
        Domain::DOM1,
        // The set of capabilities specified above.
        auth_key_caps,
        // The set of delegated capabilities, i.e. the capabilities needed by
        // the keys that we create under this authentication key. We'll be using
        // those keys to perform EC signatures, so it's the only delegated
        // capability required.
        // NOTE: This needs to be changed to SIGN_EDDSA once attestation of
        // Ed25519 keys is figured out.
        Capability::SIGN_ECDSA,
        // The authentication key's algorithm. This is the only available option.
        AuthAlg::YubicoAes,
        // The password-derived key used to protect this authentication key.
        // NOTE: The YubiHSM family uses PBKDF2 with a static salt for key
        // derivation, so a long, random password should be used.
        Key::derive_from_password(password.as_bytes()),
    ) {
        Ok(id) => id,
        Err(e) => return Err(format!("failed to insert new auth key: {}; reprovision", e)),
    };

    let credentials = Credentials::from_password(key_id, password.as_bytes());
    client = open_hsm(connector, credentials)?;

    // Remove the original, default authentication key.
    if let Err(e) = client.delete_object(DEFAULT_AUTHENTICATION_KEY_ID, Type::AuthenticationKey) {
        return Err(format!(
            "failed to delete default auth key: {}; reprovision",
            e
        ));
    }

    println!(
        "Success! Provisioned a new authentication key as object {} and deleted the default key",
        key_id
    );

    return Ok(key_id);
}

fn new_ed25519_keypair_with_attestation(
    label_str: &str,
    key_id: Id,
    client: &Client,
) -> Result<Certificate, String> {
    let label = match Label::from_bytes(label_str.as_bytes()) {
        Ok(label) => label,
        Err(e) => return Err(format!("user error: key label invalid: {}; reprovision", e)),
    };

    if let Err(e) = client.generate_asymmetric_key(
        key_id,
        label,
        Domain::DOM1,
        Capability::SIGN_EDDSA,
        Algorithm::Ed25519,
    ) {
        return Err(format!("failed to create keypair: {}; reprovision", e));
    }

    // NOTE: The None parameter here indicates that we're using the default
    // attestation key (object ID 0) to generate our attestation certificate.
    // The default attestation key is a natural choice, since it's signed
    // by an intermediate CA which in turn is signed by the well-known,
    // public Yubico CA. Yubico publishes the intermediate's public cert here:
    // https://developers.yubico.com/YubiHSM2/Concepts/E45DA5F361B091B30D8F2C6FA040DB6FEF57918E.pem
    match client.sign_attestation_certificate(key_id, None) {
        Ok(cert) => Ok(cert),
        Err(e) => Err(format!(
            "failed to create attestation certificate for {} ({}): {}; reprovision",
            label_str, key_id, e
        )),
    }
}

fn new_ecp384_keypair_with_attestation(
    label_str: &str,
    key_id: Id,
    client: &Client,
) -> Result<(Vec<u8>, Certificate), String> {
    let label = match Label::from_bytes(label_str.as_bytes()) {
        Ok(label) => label,
        Err(e) => return Err(format!("user error: key label invalid: {}; reprovision", e)),
    };

    if let Err(e) = client.generate_asymmetric_key(
        key_id,
        label,
        Domain::DOM1,
        Capability::SIGN_ECDSA,
        Algorithm::EcP384,
    ) {
        return Err(format!("failed to create keypair: {}; reprovision", e));
    }

    let pubkey = match client.get_public_key(key_id) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            return Err(format!(
                "failed to retrieve public key for {} ({}): {}; reprovision",
                label_str, key_id, e
            ))
        }
    };

    // NOTE: get_public_key returns the public key as raw bytes, meaning
    // that it isn't in a format that most libraries can consume.
    // We ask it nicely to convert itself into a common format.
    // The unwrap here is safe, since
    // NistP384::asymmetric_algorithm() == pubkey.algorithm.
    let pubkey = pubkey.ecdsa::<NistP384>().unwrap().as_bytes().to_vec();

    // NOTE: The None parameter here indicates that we're using the default
    // attestation key (object ID 0) to generate our attestation certificate.
    // The default attestation key is a natural choice, since it's signed
    // by an intermediate CA which in turn is signed by the well-known,
    // public Yubico CA. Yubico publishes the intermediate's public cert here:
    // https://developers.yubico.com/YubiHSM2/Concepts/E45DA5F361B091B30D8F2C6FA040DB6FEF57918E.pem
    let cert = match client.sign_attestation_certificate(key_id, None) {
        Ok(cert) => cert,
        Err(e) => {
            return Err(format!(
                "failed to create attestation certificate for {} ({}): {}; reprovision",
                label_str, key_id, e
            ))
        }
    };

    Ok((pubkey, cert))
}

fn run() -> Result<(), String> {
    big_scary_banner()?;

    // Step 0: Find the attached YubiHSM and return a suitable USB config
    // for connecting to it. We use this config through the other steps,
    // to avoid rediscovery.
    let usb_config = find_hsm()?;
    let serial_number = match usb_config.serial {
        Some(serial) => serial.to_string(),
        None => return Err(String::from("no serial number for USB config?")),
    };

    file_presence_checks(&serial_number)?;

    // Step 1: Reset the device to a factory state.
    perform_factory_reset(&usb_config)?;
    println!("Success! Giving the HSM 10 seconds to come back online...");
    thread::sleep(time::Duration::from_secs(HSM_USB_TIMEOUT));

    // Stage 2: Create a new authentication key, remove the default one.
    // Returns a object ID suitable for connecting to the HSM via the new
    // authentication key, as long as the user supplies the correct password.
    let auth_key_id = new_auth_key(&usb_config)?;
    println!("Success!");

    // Stage 3: Using the new authentication key, generate two keypairs
    // suitable for signing operations. Generate an x509 attestation cert for
    // each keypair, and extract the HSM's attestation certificate for
    // verifying each attestation later.
    println!("We're creating our TUF keys and attestation certificates now.");
    let password = match PasswordInput::new()
        .with_prompt("Authentication key password")
        .interact()
    {
        Ok(password) => password,
        Err(e) => return Err(format!("prompt failed: {}", e)),
    };
    let client = open_hsm(
        Connector::usb(&usb_config),
        Credentials::from_password(auth_key_id, password.as_bytes()),
    )?;

    let attestation_cert = match client.get_opaque(0) {
        Ok(cert) => cert,
        Err(e) => return Err(format!("couldn't get the HSM's attestation cert: {}", e)),
    };

    // let root_key_attestation =
    //     new_ed25519_keypair_with_attestation("tuf-root", TUF_ROOT_KEY_ID, &client)?;
    // let targets_key_attestation =
    //     new_ed25519_keypair_with_attestation("tuf-targets", TUF_TARGETS_KEY_ID, &client)?;

    let (root_pubkey, root_attestation) =
        new_ecp384_keypair_with_attestation("tuf-root", TUF_ROOT_KEY_ID, &client)?;
    let (targets_pubkey, targets_attestation) =
        new_ecp384_keypair_with_attestation("tuf-targets", TUF_TARGETS_KEY_ID, &client)?;

    // Write our public keys and attestation data to disk.
    for tup in vec![
        (YUBIHSM_ATTESTATION_CERT_SUFFIX, attestation_cert),
        (
            TUF_ROOT_KEY_ATTESTATION_FILE_SUFFIX,
            root_attestation.into_vec(),
        ),
        (TUF_ROOT_KEY_PUBKEY_FILE_SUFFIX, root_pubkey),
        (
            TUF_TARGETS_KEY_ATTESTATION_FILE_SUFFIX,
            targets_attestation.into_vec(),
        ),
        (TUF_TARGETS_KEY_PUBKEY_FILE_SUFFIX, targets_pubkey),
    ] {
        let path = format!("{}_{}", serial_number, tup.0);
        let mut file = match File::create(Path::new(&path)) {
            Ok(file) => file,
            Err(e) => {
                return Err(format!(
                    "attestation file creation failed: {}: {}",
                    tup.0, e
                ))
            }
        };

        if let Err(e) = file.write_all(&tup.1) {
            return Err(format!("attestation file I/O failed: {}: {}", tup.0, e));
        }
    }

    Ok(())
}

fn main() {
    process::exit(match run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Fatal: {}", e);
            1
        }
    });
}
