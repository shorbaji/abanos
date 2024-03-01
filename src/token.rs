/// This module gets an ID token for the CLI from the server.
/// usage:
///
/// get_token("<host>".to_string());
///
use std::{io::{Read, Write}, path::PathBuf};

pub fn get_token(host: &String) -> Result<String, String> {
    // get or create the path to ~/.abanos
    let path = config_path()?;

    let path = path.join("token");
    get_token_from_file(&path).or_else(|_| {
        login(host).inspect(|token| {
            let _ = std::fs::write(path, token);
        })
    })
}

fn get_token_from_file(path: &PathBuf) -> Result<String, String> {
    // if ~/abanos/token exists read it and return the token
    if path.exists() {
        std::fs::read_to_string(path)
            .map_err(|_| "Failed to read token file".to_string())
    } else {
        Err("Token file not found".to_string())
    }
}

fn config_path() -> Result<PathBuf, String> {
    // get or create the path to ~/.abanos
    if let Some(mut path) = home::home_dir() {
        path.push(".abanos");
        if !path.clone().is_dir() {
            println!("Creating the ~/.abanos directory");
            std::fs::create_dir_all(path.clone())
                .map_err(|_| "unable to create ~/.abanos".to_string())?;
            Ok(path)
        } else {
            Ok(path)
        }
    } else {
        Err("Failed to get the home directory".to_string())
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DeviceCode {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u32,
    interval: u32,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Token {
    access_token: String,
    token_type: String,
    scope: String,
}

fn request_device_code() -> Result<String, String> {
    let base = "https://oauth2.googleapis.com/device/code";
    let client_id = "Iv1.54d037600fdd3058";
    let scope = "read:user";
    let uri = format!("{base}?client_id={client_id}&scope={scope})");
    
    // request a device code
    let mut response = idcurl::Request::post(uri)
        .header("Accept", "application/json")
        .send()
        .expect("error sending/receive device code request");

    let device_code = serde_json::from_reader::<_, DeviceCode>(response).expect("error parsing device code response");

    // print the user code and verification uri to the user
    println!("Please go to {} and enter the code {}", device_code.verification_uri, device_code.user_code);

    // poll the server for the token
    let mut wait = 0;
    
    while wait < 300 {
        wait += device_code.interval;
        std::thread::sleep(std::time::Duration::from_secs(device_code.interval as u64));
        println!("polling for token");
        let base = "https://oauth2.googleapis.com/token";
        let uri= format!("{base}?client_id={client_id}&device_code={}&grant_type=urn:ietf:params:oauth:grant-type:device_code", device_code.device_code); 

        let mut response = idcurl::Request::post(uri)
            .header("Accept", "application/json")
            .send()
            .expect("error sending/receive device code request");

        if response.status().is_success() {
            match serde_json::from_reader::<_, Token>(response) {
                Ok(token) => return Ok(token.access_token),
                Err(e) => continue,
            }
        }
    }
    Err("Failed to get token".to_string())
}

fn login(host: &str) -> Result<String, String> {
    request_device_code()
}
