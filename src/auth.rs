use std::{env, path};
use std::path::PathBuf;

use openidconnect::{ CsrfToken, Nonce};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DeviceCode {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    expires_in: u32,
    interval: u32,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Credential {
    access_token: String,
    token_type: String,
    scope: String,
    expires_in: u32,
    id_token: String,
    #[serde(skip, default="String::new")]
    _refresh_token: String,
}

impl Credential {
    pub fn get_id_token(&self) -> &str {
        &self.id_token
    }

    pub fn _get_access_token(&self) -> &str {
        &self.access_token
    }

    pub fn _get_refresh_token(&self) -> &str {
        &self._refresh_token
    }
}

impl TryFrom<&path::PathBuf> for Credential {
    type Error = String;

    fn try_from(path: &path::PathBuf) -> Result<Self, Self::Error> {
        let s = std::fs::read_to_string(path)
            .map_err(|_| "Failed to read credential file".to_string())?;

        s.as_str().try_into()
    }
}

impl TryFrom<&str> for Credential {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(s)
            .map_err(|_| "Failed to parse credential string".to_string())
    }
}

pub fn get_credential(args: &crate::Args) -> Result<Credential, String> {
    // ~/.abanos/credential - create ~/.abanos if it doesn't exist
    let path = config_path()?.join("credential");

    Credential::try_from(&path)
        .or_else(|_| get_credential_with_oauth2(args))
            .inspect(|credential| {
                let s = serde_json::to_string(&credential).unwrap();
                let _ = std::fs::write(path, s); })
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

fn get_credential_with_oauth2(args: &crate::Args) -> Result<Credential, String> {
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
    // and build the auth url including using the redirect uri http://127.0.0.1:{port}/callback
    let (tx, rx) = std::sync::mpsc::channel();
    let csrf = state.clone();

    let token_path = "/iam/token";
    let token_url = format!("{}://{}:{}{}", &scheme, &host, &port, &token_path);

    let server = rouille::Server::new(
        "0.0.0.0:0",
        move |request| {
            rouille::router!(request,
                (GET) (/callback) => {
                    request_credential(
                        request,
                        tx.clone(),
                        &csrf,
                        &code_verifier,
                        &token_url,
                    )
                },
                _ => rouille::Response::text("Not found")
            )        
        }
    ).map_err(|_| "Failed to start server".to_string())?;


    let redirect_uri = format!("http://127.0.0.1:{}/callback", server.server_addr().port());
    
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

    // we wait for the server to send us the credential
    let credential = rx
        .recv()
        .map_err(|_| "Failed to receive credential".to_string())
        .and_then(|credential| {
            sender.send(())  // we send a stop message to the server
            .map(|_| credential)
            .map_err(|e| 
                format!("Failed to send stop message: {:?}", e))
        })?;


    // we wait for the server to stop and return the credential
    handle.join().unwrap();

    Ok(credential)
}

fn request_credential(
    request: &rouille::Request,
    tx: std::sync::mpsc::Sender<Credential>,
    csrf: &str,
    verifier: &openidconnect::PkceCodeVerifier,
    token_url: &str,
) -> rouille::Response {
    // Step 2 of the Authorization Code Flow with PKCE
    // we receive the code and state from the server
    // we check that the state matches the csrf token
    // we then request a credential from the server
    // Return codes:
    // 400 Bad Request is no code parameter
    // 400 Bad Request is no state parameter
    // 400 Bad Request is state does not match csrf
    // 500 Internal Server Error is error sending request back to server
    // 500 Internal Server Error is error reading response from token server
    // 400 Bad Request is error parsing credential
    // 200 Ok if credential sent

    request.get_param("code")
    .ok_or((400, "no code".to_string())) 
    .and_then(|code| {
        request.get_param("state")
        .ok_or((400, "no state".to_string())) 
        .and_then(|state| {
            (state == csrf).then_some(()) 
            .ok_or((400, "state does not match csrf".to_string())) 
            .and_then(|_| {
                let form = [
                    ("grant_type", "authorization_code"),
                    ("code", &code),
                    ("code_verifier", verifier.secret()),
                ];
                reqwest::blocking::Client::new()
                .post(token_url)
                .header("content-type", "application/json")
                .form(&form)
                .send() 
                .map_err(|e| (500, format!("error sending request {e:?}")))
                .and_then(|response| {
                    let status = response.status();
                    response.text()
                    .map_err(|e| (500, format!("error reading response {e:?}"))) 
                    .and_then(|text| {
                        status
                        .is_success()
                        .then_some(text.clone())
                        .ok_or((status.as_u16(), text))
                        .and_then(|text| {
                            serde_json::from_str::<Credential>(&text)
                            .map_err(|e| (400, format!("error parsing credential {e:?}")))
                            .and_then(|credential| {
                                tx.send(credential)
                                .map(|_| (200, "ok".to_string()))
                                .map_err(|e| (500, format!("error sending credential {e:?}")))
                            })    
                        })
                    })
                })
            })
        })
    })
    .map(|(code, text)| rouille::Response::text(text).with_status_code(code))
    .unwrap_or_else(|(code, text)| rouille::Response::text(text).with_status_code(code))
}

