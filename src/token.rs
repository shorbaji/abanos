use openidconnect::{
    AdditionalProviderMetadata, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse, ProviderMetadata, RedirectUrl, RevocationUrl,
    Scope, TokenResponse,
};

use openidconnect::core::{
    CoreAuthDisplay, CoreClaimName, CoreClaimType, CoreClient, CoreClientAuthMethod, CoreGrantType,
    CoreIdTokenClaims, CoreIdTokenVerifier, CoreJsonWebKey, CoreJsonWebKeyType, CoreJsonWebKeyUse,
    CoreJweContentEncryptionAlgorithm, CoreJweKeyManagementAlgorithm, CoreJwsSigningAlgorithm,
    CoreResponseMode, CoreResponseType, CoreRevocableToken, CoreSubjectIdentifierType,
};

use openidconnect::reqwest::http_client;

use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Token {
    pub jwt: String,
    expiry: chrono::DateTime<chrono::Local>,
}

impl Token {
    fn new(jwt: String, expiry: chrono::DateTime<chrono::Local>) -> Self {
        Self { jwt, expiry }
    }

    fn is_expired(&self) -> bool {
        self.expiry < chrono::Local::now()
    }
}

fn handle_error<T: std::error::Error>(fail: &T, msg: &'static str) {
    let mut err_msg = format!("ERROR: {}", msg);
    let mut cur_fail: Option<&dyn std::error::Error> = Some(fail);
    while let Some(cause) = cur_fail {
        err_msg += &format!("\n    caused by: {}", cause);
        cur_fail = cause.source();
    }
    println!("{}", err_msg);
    std::process::exit(1);
}

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

fn login() -> Result<Token, String> {

    let google_client_id = ClientId::new(
        env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );
    let google_client_secret = ClientSecret::new(
        env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
    );
    let issuer_url =
        IssuerUrl::new("https://accounts.google.com".to_string()).expect("Invalid issuer URL");

    let provider_metadata = GoogleProviderMetadata::discover(&issuer_url, http_client)
    .unwrap_or_else(|err| {
        handle_error(&err, "Failed to discover OpenID Provider");
        unreachable!();
    });

    let revocation_endpoint = provider_metadata
        .additional_metadata()
        .revocation_endpoint
        .clone();

    // Set up the config for the Google OAuth2 process.
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        google_client_id,
        Some(google_client_secret),
    )
    // This example will be running its own server at localhost:8080.
    // See below for the server implementation.
    .set_redirect_uri(
        RedirectUrl::new("https://api.staging.abanos.io/iam/auth/callback".to_string()).expect("Invalid redirect URL"),
    )
    // Google supports OAuth 2.0 Token Revocation (RFC-7009)
    .set_revocation_uri(
        RevocationUrl::new(revocation_endpoint).expect("Invalid revocation endpoint URL"),
    );

        // Generate the authorization URL to which we'll redirect the user.
        let (authorize_url, csrf_state, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        // .add_scope(Scope::new("https://www.googleapis.com/auth/cloud-platform".to_string()))
        // .add_scope(Scope::new("https://www.googleapis.com/auth/service.management".to_string()))
        .url();

    open::that(authorize_url.as_str()).unwrap();

    let mut code = String::new();

    println!("Enter the authorization code: ");
    std::io::stdin().read_line(&mut code).unwrap();
    let code = AuthorizationCode::new(code);

    let token_response = client
    .exchange_code(code)
    .request(http_client)
    .unwrap_or_else(|err| {
        handle_error(&err, "Failed to contact token endpoint");
        unreachable!();
    });

    let time_delta = chrono::TimeDelta::new(token_response.expires_in().unwrap().as_secs() as i64, 0);

    if let Some(td) = time_delta {
        println!("Token expires in: {}", td);
        let expiry = chrono::Local::now() + td;

        let token = Token::new(token_response.id_token().unwrap().to_string(), expiry);

        Ok(token)
    
    } else {
        Err("invalid expiry time".to_string())
    }
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

pub fn get_token() -> Result<Token, String> {
    get_token_from_file()
    .and_then(|token| {
        if token.is_expired() {
            Err("Token expired".to_string())
        } else {
            Ok(token)
        }
    })
    .or_else(|_| 
        login()
        .inspect(|token| {
            let path = get_config_path().unwrap().join("token");
            let s = serde_json::to_string(&token).unwrap();
            std::fs::write(path, s).unwrap();
        })
    )
}
