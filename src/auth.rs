/// This module gets an ID token for the CLI from the server.
/// usage:
///
/// get_token("<host>".to_string());
///
use std::env;
use std::path::PathBuf;

use openidconnect::{ CsrfToken, Nonce, };

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Token {
    access_token: String,
    token_type: String,
    scope: String,
    expires_in: u32,
    id_token: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DeviceCode {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    expires_in: u32,
    interval: u32,
}

impl Token {
    pub fn get(&self) -> String {
        self.access_token.clone()
    }
}

pub fn get_token(args: &crate::Args) -> Result<Token, String> {
    // ~/.abanos/token - create ~/.abanos if it doesn't exist
    let path = config_path()?.join("token");

    read_token_from_file(&path)
        .or_else(|_|
            login(args).inspect(|token| {
                let s = serde_json::to_string(&token).unwrap();
                let _ = std::fs::write(path, s); }))
}

fn read_token_from_file(path: &PathBuf) -> Result<Token, String> {
    // if ~/abanos/token exists read it and return the token
    if path.exists() {
        std::fs::read_to_string(path)
            .map_err(|_| "Failed to read token file".to_string())
            .and_then(|s| serde_json::from_str(&s).map_err(|_| "Failed to parse token file".to_string()))
        
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

fn request_token(
    request: &rouille::Request,
    tx: &std::sync::mpsc::Sender<Token>,
    csrf: &str,
    verifier: &openidconnect::PkceCodeVerifier,
    token_url: &str,
) -> rouille::Response {
    // Step 4 of the Authorization Code Flow with PKCE
    // we receive the code and state from the server
    // we check that the state matches the csrf token
    // we then request a token from the server

    let code = request.get_param("code").unwrap();
    let state = request.get_param("state").unwrap();

    // we need to compare the csrf token with the state
    // we then need to reuest a token from the server

    if state == csrf {
        println!("state matches csrf");
    } else {
        println!("state does not match csrf");
    }

    println!("token_url: {:?}", token_url);
    // call iam/token
    let body = serde_json::json!({
        "grant_type": "authorization_code",
        "code": code,
        "code_verifier": verifier.secret(),
    });

    let body = serde_json::to_string(&body).unwrap();

    let request = reqwest::blocking::Client::new()
        .post(token_url)
        .header("content-type", "application/json")
        .body(body);

    let response = request.send();

    match response {
        Ok(response) => {
            if response.status().is_success() {
                let text = response.text().unwrap();
                let token: Token = serde_json::from_str(&text).unwrap();
                tx.send(token).unwrap();
                rouille::Response::text("ok")
            } else {
                rouille::Response::text("error")
            }
        }
        Err(_) => rouille::Response::text("error"),
    }

}

fn login(args: &crate::Args) -> Result<Token, String> {

    // Step 1 of the Authorization Code Flow with PKCE
    // is to request an authorization code from the server

    // to make the request we need to build an authorization url
    // the url consists of a {scheme}::/{host}:{port}/{auth_path}?{query}
    let scheme = if args.no_tls { "http" } else { "https" };
    let host = args.host.clone();
    let port = args.port;
    let auth_path = "/iam/authorize";

    // query consists of:
    // 1. client id (default is "")
    // 2. client secret (default is "")
    // 3. response type (default is "code")
    // 4. scope (default is "openid")
    // 5. state (a random string)
    // 6. code challenge
    // 7. code challenge method
    // 8. redirect uri

    let client_id = env::var("ABANOS_AUTH_CLIENT_ID").ok().unwrap_or("".to_string());
    let client_secret = env::var("ABANOS_AUTH_CLIENT_SECRET").ok().unwrap_or("".to_string());
    let response_type = "code";
    let scope = "openid";
    let state = CsrfToken::new_random().secret().clone();
    let (code_challenge, code_verifier) = openidconnect::PkceCodeChallenge::new_random_sha256();
    let code_challenge_method = "S256";

    let _nonce = Nonce::new_random();

    // next we create a server to listen for the callback on a random port
    // and build the auth url including using the redirect uri http://localhost:{port}/callback
    let (tx, rx) = std::sync::mpsc::channel();
    let csrf = state.clone();

    let token_path = "/iam/token";
    let token_url = format!("{}://{}:{}{}", &scheme, &host, &port, &token_path);
    println!("token_url passed to local server: {:?}", token_url);

    let server = rouille::Server::new(
        "localhost:0",
        move |request| {
            rouille::router!(request,
                (GET) (/callback) => {
                    request_token(
                        request,
                        &tx,
                        &csrf,
                        &code_verifier,
                        &token_url,
                    )
                },
                _ => rouille::Response::text("Not found")
            )
        }
    ).map_err(|_| "Failed to start server".to_string())?;

    let redirect_uri = format!("http://localhost:{}/callback", server.server_addr().port());
    
    let query = [
        format!("client_id={}", client_id),
        format!("client_secret={}", client_secret),
        format!("response_type={}", response_type),
        format!("scope={}", scope),
        format!("state={}", state),
        format!("code_challenge={}", code_challenge.as_str()),
        format!("code_challenge_method={}", code_challenge_method),
        format!("redirect_uri={}", redirect_uri),
    ].join("&");

    let auth_url = format!("{}://{}:{}{}?{}", scheme, &host, &port, &auth_path, &query);

    // now that we have the auth url and the server
    // we start the server and open the url in the browser
    let (handle, sender) = server.stoppable();

    open::that(&auth_url)
    .inspect_err(|_| println!("Browse to {}", &auth_url))
    .unwrap_or(());

    // we wait for the server to send us the token
    let token = rx
        .recv()
        .map_err(|_| "Failed to receive token".to_string())
        .and_then(|token| {
            sender.send(())  // we send a stop message to the server
            .map(|_| token)
            .map_err(|e| 
                format!("Failed to send stop message: {:?}", e))
        })?;


    // we wait for the server to stop and return the token
    handle.join().unwrap();

    Ok(token)
}