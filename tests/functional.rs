use assert_cmd::prelude::*;
use color_eyre::eyre::{eyre, Result};
use rand::Rng;
use scopeguard::defer;
use std::{fs, process::Command, thread, time::Duration};

#[test]
fn empty_config() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let cfg_path = tmp.path().join("config.toml");
    fs::write(&cfg_path, "")?;
    escargot::CargoBuild::new()
        .bin("portproxy")
        .current_release()
        .current_target()
        .run()?
        .command()
        .env("RUST_LOG", "debug")
        .env("PORTPROXY_CONFIG", &cfg_path)
        .assert()
        .failure()
        .stderr(predicates::str::contains("No proxies defined in config"));

    Ok(())
}

#[test]
fn functional() -> Result<()> {
    let tmp = tempfile::tempdir()?;

    let config = "\
    [[proxy]]
    listen = \"127.0.0.1:8001\"
    connect = \"127.0.0.1:8000\"
    ";

    let config_path = tmp.path().join("config.toml");
    fs::write(tmp.path().join(&config_path), config)?;

    let data: String = rand::thread_rng()
        .sample_iter::<char, _>(rand::distributions::Standard)
        .take(1024)
        .collect();

    fs::write(tmp.path().join("data"), &data)?;

    let mut server = Command::new("python3")
        .current_dir(&tmp)
        .args(["-m", "http.server", "--bind", "127.0.0.1"])
        .spawn()?;
    defer!(server.kill().expect("could not kill server"));

    let mut portproxy = escargot::CargoBuild::new()
        .bin("portproxy")
        .current_release()
        .current_target()
        .run()?
        .command()
        .env("RUST_LOG", "debug")
        .env("PORTPROXY_CONFIG", &config_path)
        .spawn()?;
    defer!(portproxy.kill().expect("could not kill portproxy"));

    // Wait for the server to start
    thread::sleep(Duration::from_secs(1));

    // Read the data via HTTP
    let read = reqwest::blocking::get("http://127.0.0.1:8001/data")?.text()?;

    if read != data {
        return Err(eyre!("Read data does not match the generated one"));
    }

    Ok(())
}
