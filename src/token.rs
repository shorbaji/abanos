/// This module gets an ID token for the CLI from the server.
/// usage: 
///
/// get_token("<host>".to_string());
///
use rouille::{Response, Server}; // needed to set up a local server to serve a redirect URL and receive the token
use std::path::PathBuf; // needed to save the token to a file
use std::sync::mpsc; // needed for the local server to communicate with the main thread

pub fn get_token(host: String) -> Result<String, String> {
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
        std::fs::read_to_string(path).map_err(|_| "Failed to read token file".to_string())
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

fn login(host: String) -> Result<String, String> {
    // we either open a browser with the auth login url OR
    // we ask the user to open a browser with a url to login and enter the resulting code
    // either way we need a base url to start with
    let url_base = format!("https://{host}/static/login.html");

    login_with_browser(&url_base)
        .or_else(|_| login_without_browser(&url_base))
}

fn login_with_browser(url_base: &String) -> Result<String, String> {
    let (tx, rx) = mpsc::channel();

    // we create the server without running it first so we can get the port
    let server = Server::new(
                    "localhost:6871",
                    move |request| handler(request, tx.clone()))
                    .map_err(|e| format!("rouille error: {:?}", e))?;
    let addr = server
                .server_addr()
                .port();

    // we use the port as part of the redirect url
    // let url = format!("{url_base}?signInSuccessUrl=http://localhost:{addr}");
    let url = get_authorization_url(addr);

    // try to open the browser with the url
    match open::that(url.as_str()) {
        Ok(_) => {
            // since we have success with the browser we start the server
            // and listen for the token to be sent to us by the server
            let (handle, sender) = server.stoppable();

            match rx.recv() {
                Ok(jwt) => {
                    sender
                        .send(()) // as the server to stop
                        .map_err(|e| format!("mpsc channel error: {:?}", e))?;
                    handle
                        .join() // wait for the server to stop
                        .map_err(|e| format!("server error: {:?}", e))?;
                    Ok(jwt)
                }
                Err(e) => Err(format!("mpsc channel error: {:?}", e)),
            }
        }
        Err(e) => Err(format!("open error: {:?}", e)),
    }
}

fn handler(request: &rouille::Request, tx: mpsc::Sender<String>) -> rouille::Response {
    // once a request comes in - look for the jwt param and send it to the main thread
    if let Some(jwt) = request.get_param("jwt") {
        match tx.send(jwt) {
            Ok(_) => Response::text("Login successful. You can close this tab."),
            Err(e) => Response::text(format!("mpsc channel error: {:?}", e)).with_status_code(500),
        }
    } else {
        // extract the jwt from the hash/fragement and redirect to a url with the jwt as a query param
        Response::html("<script>
            let loc = window.location;
            let hash = loc.hash;
            let jwt = hash.split('&').find((s) => s.startsWith('id_token=')).split('=')[1];
            window.location.replace(`http://localhost:6871/?jwt=${jwt}`);
        </script>")
    }
}

fn login_without_browser(url_base: &String) -> Result<String, String> {
    // if no browser, provide the user with a URL to open in their browser
    // redirecting to a page on the server that will show the code
    // then we ask the user to enter the code
    let mut jwt: String = String::new();

    let url = format!("{}/static/show_code.html", url_base);
    println!(
        "No brower detected. Please open the following URL in your browser and login: {}",
        url
    );
    println!("Enter the authorization code:");
    match std::io::stdin().read_line(&mut jwt) {
        Ok(_) => Ok(jwt),
        Err(e) => Err(format!("stdin error: {:?}", e)),
    }
}


use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::process::exit;

use serde::{Deserialize, Serialize};

use openidconnect::core::{
    CoreAuthDisplay, CoreClaimName, CoreClaimType, CoreClient, CoreClientAuthMethod, CoreGrantType,
    CoreIdTokenClaims, CoreIdTokenVerifier, CoreJsonWebKey, CoreJsonWebKeyType, CoreJsonWebKeyUse,
    CoreJweContentEncryptionAlgorithm, CoreJweKeyManagementAlgorithm, CoreJwsSigningAlgorithm,
    CoreResponseMode, CoreResponseType, CoreRevocableToken, CoreSubjectIdentifierType,
};
use openidconnect::reqwest::http_client;
use openidconnect::{
    AdditionalProviderMetadata, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse, ProviderMetadata, RedirectUrl, RevocationUrl,
    Scope,
};

fn handle_error<T: std::error::Error>(fail: &T, msg: &'static str) {
    let mut err_msg = format!("ERROR: {}", msg);
    let mut cur_fail: Option<&dyn std::error::Error> = Some(fail);
    while let Some(cause) = cur_fail {
        err_msg += &format!("\n    caused by: {}", cause);
        cur_fail = cause.source();
    }
    println!("{}", err_msg);
    exit(1);
}

// Teach openidconnect-rs about a Google custom extension to the OpenID Discovery response that we can use as the RFC
// 7009 OAuth 2.0 Token Revocation endpoint. For more information about the Google specific Discovery response see the
// Google OpenID Connect service documentation at: https://developers.google.com/identity/protocols/oauth2/openid-connect#discovery
#[derive(Clone, Debug, Deserialize, Serialize)]
struct RevocationEndpointProviderMetadata {
    revocation_endpoint: String,
}
impl AdditionalProviderMetadata for RevocationEndpointProviderMetadata {}
type GoogleProviderMetadata = ProviderMetadata<
    RevocationEndpointProviderMetadata,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJwsSigningAlgorithm,
    CoreJsonWebKeyType,
    CoreJsonWebKeyUse,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
>;

fn get_authorization_url(port: u16) -> String {

    let google_client_id = ClientId::new(
        env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );
    let issuer_url =
        IssuerUrl::new("https://accounts.google.com".to_string()).expect("Invalid issuer URL");

    // Fetch Google's OpenID Connect discovery document.
    //
    // Note: If we don't care about token revocation we can simply use CoreProviderMetadata here
    // instead of GoogleProviderMetadata. If instead we wanted to optionally use the token
    // revocation endpoint if it seems to be supported we could do something like this:
    //   #[derive(Clone, Debug, Deserialize, Serialize)]
    //   struct AllOtherProviderMetadata(HashMap<String, serde_json::Value>);
    //   impl AdditionalClaims for AllOtherProviderMetadata {}
    // And then test for the presence of "revocation_endpoint" in the map returned by a call to
    // .additional_metadata().

    let provider_metadata = GoogleProviderMetadata::discover(&issuer_url, http_client)
        .unwrap_or_else(|err| {
            handle_error(&err, "Failed to discover OpenID Provider");
            unreachable!();
        });

    let revocation_endpoint = provider_metadata
        .additional_metadata()
        .revocation_endpoint
        .clone();
    println!(
        "Discovered Google revocation endpoint: {}",
        revocation_endpoint
    );

    // Set up the config for the Google OAuth2 process.
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        google_client_id,
        None,
    )
    // This example will be running its own server at localhost:8080.
    // See below for the server implementation.
    .set_redirect_uri(
        RedirectUrl::new(format!("http://localhost:{port}")).expect("Invalid redirect URL"),
    )
    // Google supports OAuth 2.0 Token Revocation (RFC-7009)
    .set_revocation_uri(
        RevocationUrl::new(revocation_endpoint).expect("Invalid revocation endpoint URL"),
    );


    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::Implicit(true),
            CsrfToken::new_random,
            Nonce::new_random,
        )
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    authorize_url.to_string()

}