use clap::{App, Arg};
use dialoguer::{Confirmation, PasswordInput};
use lazy_static::lazy_static;
use pkcs11::{types, Ctx};
use rand::Rng;
use regex::Regex;

use std::path::Path;
use std::process;

#[cfg(target_os = "macos")]
const OPENSC_PKCS11_SO: &'static str = "/usr/local/lib/opensc-pkcs11.so";

#[cfg(all(target_arch = "arm", target_os = "linux"))]
const OPENSC_PKCS11_SO: &'static str = "/usr/lib/arm-linux-gnueabihf/opensc-pkcs11.so";

const BIG_SCARY_BANNER: &'static str = r#"
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
"#;

// The Nitrokey HSM uses a Smartcard-HSM internally; the latter
// uses a 16-character hex string as its SO PIN.
const SO_PIN_ALPHABET: &'static [u8] = b"0123456789abcdef";
const SO_PIN_LENGTH: usize = 16;

// There's conflicting information available online about the valid character
// set and maximum length for a normal user PIN.
// We use 6 characters chosen from the lowercase alphabet + numbers.
const USER_PIN_ALPHABET: &'static [u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
const USER_PIN_LENGTH: usize = 6;

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

fn is_valid_so_pin(val: String) -> Result<(), String> {
    lazy_static! {
        static ref SO_PIN_PATTERN: Regex = Regex::new("^[[:xdigit:]]{16}$").unwrap();
    }

    if SO_PIN_PATTERN.is_match(&val) {
        Ok(())
    } else {
        Err(format!("invalid SO pin (expected 16 hex digits): {}", val))
    }
}

fn find_hsm() -> Result<(Ctx, types::CK_SLOT_ID, String), String> {
    let pkcs11_so_path = Path::new(OPENSC_PKCS11_SO);
    if !pkcs11_so_path.exists() {
        return Err(format!(
            "No OpenSC PKCS#11 shared object: {}",
            OPENSC_PKCS11_SO
        ));
    }

    // Open up our PKCS#11 context, using the OpenSC PKCS#11 shared object.
    // A failure here indicates something fundamentally wrong with either
    // the shared object or these bindings and *NOT* the HSM itself.
    let ctx = match Ctx::new_and_initialize(pkcs11_so_path) {
        Ok(ctx) => ctx,
        Err(e) => {
            return Err(format!(
                "Couldn't load and initialize the OpenSC PKCS#11 interface: {}",
                e
            ))
        }
    };

    // Grab the list of available token slots. A token can have more than
    // one slot, but (experimentally) the Nitrokey HSM only has one.
    let slots = match ctx.get_slot_list(true) {
        Ok(slots) => slots,
        Err(e) => return Err(format!("Couldn't get slot list: {}", e)),
    };

    // Sanity checks: we expect to be run with exactly one HSM plugged in,
    // so having no slots *OR* more than one available indicates a user error.
    let slot = match slots.len() {
        1 => slots.as_slice()[0],
        0 => return Err(String::from("no HSMs detected")),
        _ => {
            return Err(String::from(
                "more than one HSM or token detected; refusing to continue",
            ))
        }
    };

    // Sanity-check the token that's backing our single slot.
    // Don't allow a non-Nitrokey HSM to progress beyond this point.
    let manufacturer_id = match ctx.get_slot_info(slot) {
        Ok(slot_info) => String::from(slot_info.manufacturerID),
        Err(e) => {
            return Err(format!(
                "unable to get token info for slot #{}: {}",
                slot, e
            ))
        }
    };

    if manufacturer_id != "Nitrokey" {
        return Err(format!("unknown HSM: {}", manufacturer_id));
    }

    // Finally, grab our Nitrokey HSM's serial number, so that we can write
    // unique files to disk.
    let serial_number = match ctx.get_token_info(slot) {
        Ok(token) => String::from(token.serialNumber),
        Err(e) => {
            return Err(format!(
                "couldn't get info for token with slot #{}: {}",
                slot, e
            ))
        }
    };

    println!("Successfully discovered a Nitrokey HSM with Slot #{}", slot);

    Ok((ctx, slot, serial_number))
}

fn token_in_deadly_state(token: &types::CK_TOKEN_INFO) -> bool {
    // Our HSM is said to be in a "deadly" state if it is either
    // one PIN attempt away from locking out a user, has already
    // locked out a user, or failed its own self-check.
    (token.flags
        & (types::CKF_USER_PIN_FINAL_TRY
            | types::CKF_USER_PIN_LOCKED
            | types::CKF_SO_PIN_FINAL_TRY
            | types::CKF_SO_PIN_LOCKED
            | types::CKF_ERROR_STATE))
        >= 1
}

fn new_so_pin() -> String {
    let mut rng = rand::thread_rng();

    (0..SO_PIN_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0, SO_PIN_ALPHABET.len());
            SO_PIN_ALPHABET[idx] as char
        })
        .collect()
}

fn new_user_pin() -> String {
    let mut rng = rand::thread_rng();

    (0..USER_PIN_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0, USER_PIN_ALPHABET.len());
            USER_PIN_ALPHABET[idx] as char
        })
        .collect()
}

fn perform_factory_reset(
    pkcs11_ctx: &Ctx,
    slot: types::CK_SLOT_ID,
    so_pin: &str,
) -> Result<String, String> {
    confirm("Continue with factory reset? This step is IRREVERSIBLE!")?;

    let token = match pkcs11_ctx.get_token_info(slot) {
        Ok(token) => token,
        Err(e) => {
            return Err(format!(
                "couldn't get info for token with slot #{}: {}",
                slot, e
            ))
        }
    };

    // First, check to see if we're in a dead or deadly state.
    // Don't attempt to perform any automatic steps if we are.
    if token_in_deadly_state(&token) {
        return Err(format!(
            "HSM is either locked or one step away from locking; requires manual intervention"
        ));
    }

    // Next, initialize (or reinitialize) the HSM with the current SO PIN.
    if let Err(e) = pkcs11_ctx.init_token(slot, Some(so_pin), "Nitrokey HSM - TUF") {
        return Err(format!("failed to (re)initialize HSM: {}", e));
    }
    println!("Success! Reinitialized the HSM.");

    // Next, authenticate using the current SO PIN and change the SO PIN
    // to a new random PIN. Display this new SO PIN to the user for storage.
    let session = match pkcs11_ctx.open_session(
        slot,
        types::CKF_SERIAL_SESSION | types::CKF_RW_SESSION,
        None,
        None,
    ) {
        Ok(session) => session,
        Err(e) => return Err(format!("failed to open session with HSM: {}", e)),
    };

    if let Err(e) = pkcs11_ctx.login(session, types::CKU_SO, Some(so_pin)) {
        pkcs11_ctx
            .close_session(session)
            .unwrap_or_else(|e| eprintln!("Error while closing session: {}", e));
        return Err(format!("failed to login as Security Officer: {}", e));
    }

    // Generate our new, random SO PIN.
    let new_so_pin = new_so_pin();

    println!("This is your NEW Security Officer PIN: {}", new_so_pin);
    println!("You MUST write this SO PIN down before continuing.");

    let confirm_new_so_pin = match PasswordInput::new()
        .with_prompt("Re-enter your NEW Security Officer PIN")
        .interact()
    {
        Ok(password) => password,
        Err(e) => {
            pkcs11_ctx
                .close_session(session)
                .unwrap_or_else(|e| eprintln!("Error while closing session: {}", e));
            return Err(format!("prompt failed: {}", e));
        }
    };

    if new_so_pin != confirm_new_so_pin {
        pkcs11_ctx
            .close_session(session)
            .unwrap_or_else(|e| eprintln!("Error while closing session: {}", e));
        return Err(String::from("SO PIN does not match!"));
    }

    // Set our new SO PIN.
    if let Err(e) = pkcs11_ctx.set_pin(session, Some(so_pin), Some(&new_so_pin)) {
        pkcs11_ctx
            .close_session(session)
            .unwrap_or_else(|e| eprintln!("Error while closing session: {}", e));
        return Err(format!("Failed to set new SO PIN: {}", e));
    }

    // Re-login with our new SO PIN, so that we can set a new user PIN.
    if let Err(e) = pkcs11_ctx.logout(session) {
        pkcs11_ctx
            .close_session(session)
            .unwrap_or_else(|e| eprintln!("Error while closing session: {}", e));
        return Err(format!("Failed to cycle SO session (logout): {}", e));
    }

    if let Err(e) = pkcs11_ctx.login(session, types::CKU_SO, Some(&new_so_pin)) {
        pkcs11_ctx
            .close_session(session)
            .unwrap_or_else(|e| eprintln!("Error while closing session: {}", e));
        return Err(format!("Failed to cycle SO session (login): {}", e));
    }

    // Generate our new, random user PIN.
    let new_user_pin = new_user_pin();

    println!("This is your NEW user PIN: {}", new_user_pin);
    println!("You MUST write this user PIN down before continuing.");

    let confirm_new_user_pin = match PasswordInput::new()
        .with_prompt("Re-enter your NEW user PIN")
        .interact()
    {
        Ok(password) => password,
        Err(e) => {
            pkcs11_ctx
                .close_session(session)
                .unwrap_or_else(|e| eprintln!("Error while closing session: {}", e));
            return Err(format!("prompt failed: {}", e));
        }
    };

    if new_user_pin != confirm_new_user_pin {
        pkcs11_ctx
            .close_session(session)
            .unwrap_or_else(|e| eprintln!("Error while closing session: {}", e));
        return Err(String::from("User PIN does not match!"));
    }

    if let Err(e) = pkcs11_ctx.init_pin(session, Some(&new_user_pin)) {
        pkcs11_ctx
            .close_session(session)
            .unwrap_or_else(|e| eprintln!("Error while closing session: {}", e));
        return Err(format!("Failed to set new user PIN: {}", e));
    }

    if let Err(e) = pkcs11_ctx.close_session(session) {
        return Err(format!("Failed to close session: {}", e));
    }

    println!("Success! We've reinitialized the Nitrokey with a new SO PIN and user PIN.");

    // Return the new user PIN so that we can use it for key generation later.
    Ok(new_user_pin)
}

fn run() -> Result<(), String> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("so-pin")
                .help("the current Security Officer PIN")
                .short("p")
                .long("so-pin")
                .multiple(false)
                .takes_value(true)
                .required(true)
                .validator(is_valid_so_pin),
        )
        .get_matches();

    let so_pin = matches.value_of("so-pin").unwrap();

    big_scary_banner()?;

    let (pkcs11_ctx, slot, serial_number) = find_hsm()?;

    // Step 1: Ensure that the Nitrokey is in an acceptable state. This includes:
    //  1. Reinitializing the HSM using the current SO PIN.
    //  2. Setting a new SO PIN.
    //  3. Creating the normal user account and PIN.
    perform_factory_reset(&pkcs11_ctx, slot, &so_pin)?;

    println!("Use this serial number when doing key generation: {}", serial_number);

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
