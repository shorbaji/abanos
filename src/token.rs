use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Token {
    pub jwt: String,
}

impl Token {
    fn new(jwt: String) -> Self {
        Self { jwt }
    }

}

use rouille::{Response, Server};

fn login(host: String) -> Result<Token, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    // let (otx, orx) = std::sync::mpsc::channel();

    let server = Server::new("localhost:0", move |request| {
        let jwt = request.get_param("jwt").expect("problem logging in");
        tx.send(jwt).unwrap();
        Response::text("logged in")
    }).unwrap();

    let addr = server.server_addr().port();
    let url_base = format!("https://{host}/static/login.html");

    let url = format!("{}?signInSuccessUrl=http://localhost:{}", url_base, addr);

    let mut jwt: String = String::new();

    match open::that(url.as_str()) {
        Ok(_) => {
            println!("Waiting for browser login to complete ...");
            let (handle, sender) = server.stoppable();    
            jwt = rx.recv().unwrap();
            sender.send(()).unwrap();
            handle.join().unwrap();                
        
        }
        Err(_) => {
            let url = format!("{}/static/show_code.html", url_base);
            println!("No brower detected. Please open the following URL in your browser and login: {}", url);
            println!("Enter the authorization code:");
            std::io::stdin().read_line(&mut jwt).unwrap();
        }
    }

    Ok(Token::new(jwt))
}

fn get_config_path() -> Result<PathBuf, String> {
    if let Some(mut path) = home::home_dir() {
        path.push(".abanos");
        if !path.clone().is_dir() {
            println!("Creating the ~/.abanos directory");
            std::fs::create_dir_all(path.clone()).map_err(|_| "unable to create ~/.abanos".to_string())?;
            Ok(path)
        } else {
            Ok(path)
        }
    } else {
        Err("Failed to get the home directory".to_string())
    }
}

fn get_token_from_file() -> Result<Token, String> {
    // get the path to ~/.abanos - create it if it doesn't exist
    let path = get_config_path();

    // check if a token file exists
    let token_file = path.clone().unwrap().join("token");

    // if it doesn't exist, call login
    // if it exists read the token and check if it is expired
    // if it is expired, call login
    // if it is not expired, use it
    if token_file.exists() {
        let token = std::fs::read_to_string(token_file).unwrap();
        let token = serde_json::from_str::<Token>(&token).unwrap();
        Ok(token)
    } else {
        Err("Token file not found".to_string())
    }
}

pub fn save_token(token: &Token)  {
    let path = get_config_path().unwrap().join("token");
    let s = serde_json::to_string(&token).unwrap();
    std::fs::write(path, s).unwrap();
}

pub fn get_token(host: String) -> Result<Token, String> {
    get_token_from_file()
    // .and_then(check_not_expired)
    .or_else(|_| login(host).inspect(save_token))
}
