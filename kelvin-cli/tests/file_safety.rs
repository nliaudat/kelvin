//! End-to-end checks that file decryption rejects cut or reordered ciphertext,
//! and that the CLI never destroys its input or leaves partial files behind.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `STREAMING_CHUNK_SIZE`, also passed as `--bytes-per-step`.
const CHUNK: usize = 65_536;
const AUTH_OVERHEAD: usize = 33;
const SECURE_TAG: usize = 16;

struct Workspace {
    dir: PathBuf,
}

impl Workspace {
    fn new(name: &str) -> Self {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let workspace = Workspace { dir };
        let config = workspace.path("config.json");
        assert!(kelvin(&["keygen", "--fast", "--output", &config]));
        workspace
    }

    fn path(&self, name: &str) -> String {
        self.dir.join(name).to_str().unwrap().to_owned()
    }

    fn run(&self, command: &str, mode: &str, input: &str, output: &str) -> bool {
        let bytes_per_step = CHUNK.to_string();
        let config = self.path("config.json");
        let (input, output) = (self.path(input), self.path(output));
        let mut args = vec![command, "--mode", mode, "--nonce", "00112233"];
        args.extend(["--config", &config, "--input", &input, "--output", &output]);
        if mode != "secure" {
            args.extend(["--bytes-per-step", &bytes_per_step]);
        }
        kelvin(&args)
    }

    fn assert_no_temporary_files(&self) {
        for entry in fs::read_dir(&self.dir).unwrap() {
            let name = entry.unwrap().file_name().into_string().unwrap();
            assert!(!name.ends_with(".tmp"), "left behind: {name}");
        }
    }
}

fn kelvin(args: &[&str]) -> bool {
    Command::new(env!("CARGO_BIN_EXE_kelvin")).args(args).output().unwrap().status.success()
}

fn plaintext(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 251) as u8).collect()
}

fn check_mode(mode: &str, overhead: usize) {
    let workspace = Workspace::new(mode);
    let record = CHUNK + overhead;

    // Empty, short, exact-multiple and uneven lengths all round-trip.
    for len in [0, 100, 2 * CHUNK, 2 * CHUNK + 100] {
        fs::write(workspace.path("plain"), plaintext(len)).unwrap();
        assert!(workspace.run("encrypt", mode, "plain", "cipher"), "{mode}: encrypt {len}");
        assert!(workspace.run("decrypt", mode, "cipher", "round"), "{mode}: decrypt {len}");
        assert_eq!(fs::read(workspace.path("round")).unwrap(), plaintext(len), "{mode}: {len}");
    }

    // `cipher` now holds two full records and a short final one.
    let cipher = fs::read(workspace.path("cipher")).unwrap();
    let mut swapped = cipher.clone();
    swapped[..record].copy_from_slice(&cipher[record..2 * record]);
    swapped[record..2 * record].copy_from_slice(&cipher[..record]);
    let mut tampered = cipher.clone();
    tampered[record + 10] ^= 1;
    let damaged = [
        ("an empty file", Vec::new()),
        ("a cut after one chunk", cipher[..record].to_vec()),
        ("a cut after two chunks", cipher[..2 * record].to_vec()),
        ("a cut inside a chunk", cipher[..record + 100].to_vec()),
        ("reordered chunks", swapped),
        ("a tampered chunk", tampered),
        ("appended data", [cipher.clone(), cipher.clone()].concat()),
    ];
    for (what, bytes) in damaged {
        fs::write(workspace.path("damaged"), bytes).unwrap();
        fs::write(workspace.path("kept"), b"previous contents").unwrap();

        assert!(!workspace.run("decrypt", mode, "damaged", "fresh"), "{mode}: accepted {what}");
        assert!(!Path::new(&workspace.path("fresh")).exists(), "{mode}: output for {what}");
        assert!(!workspace.run("decrypt", mode, "damaged", "kept"), "{mode}: accepted {what}");
        assert_eq!(fs::read(workspace.path("kept")).unwrap(), b"previous contents");
    }

    // Writing onto the input is refused and leaves it intact.
    assert!(!workspace.run("encrypt", mode, "plain", "plain"), "{mode}: input == output");
    assert_eq!(fs::read(workspace.path("plain")).unwrap(), plaintext(2 * CHUNK + 100));

    workspace.assert_no_temporary_files();
}

#[test]
fn secure_mode_files() {
    check_mode("secure", SECURE_TAG);
}

#[test]
fn chaos_mode_files() {
    check_mode("chaos", AUTH_OVERHEAD);
}

#[test]
fn photon_mode_files() {
    check_mode("photon", AUTH_OVERHEAD);
}

#[test]
fn quantum_mode_files() {
    check_mode("quantum", AUTH_OVERHEAD);
}

#[test]
fn zero_bytes_per_step_is_rejected() {
    let workspace = Workspace::new("zero_step");
    fs::write(workspace.path("plain"), plaintext(100)).unwrap();
    let config = workspace.path("config.json");
    let (input, output) = (workspace.path("plain"), workspace.path("cipher"));

    for command in ["encrypt", "decrypt"] {
        let mut args = vec![command, "--mode", "photon", "--nonce", "00"];
        args.extend(["--config", &config, "--input", &input, "--output", &output]);
        args.extend(["--bytes-per-step", "0"]);
        assert!(!kelvin(&args), "{command} accepted a zero step size");
    }
    assert!(!Path::new(&output).exists());
}
