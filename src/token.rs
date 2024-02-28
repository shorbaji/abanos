/// This module gets an ID token for the CLI from the server.
/// usage:
///
/// get_token("<host>".to_string());
///
use rouille::{Response, Server}; // needed to set up a local server to serve a redirect URL and receive the token
use std::path::PathBuf; // needed to save the token to a file
use std::sync::mpsc; // needed for the local server to communicate with the main thread

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

fn login(host: &String) -> Result<String, String> {
    // we either open a browser with the auth login url OR
    // we ask the user to open a browser with a url to login and enter the resulting code
    // either way we need a base url to start with
    login_with_browser(host).or_else(|_| login_without_browser(host))
}

fn login_with_browser(host: &String) -> Result<String, String> {
    let (tx, rx) = mpsc::channel();

    // we create the server without running it first so we can get the port
    let server = Server::new("localhost:0", move |request| handler(request, tx.clone()))
        .map_err(|e| format!("rouille error: {:?}", e))?;
    let addr = server.server_addr().port();

    // we use the port as part of the redirect url
    // let url = format!("{url_base}?signInSuccessUrl=http://localhost:{addr}");
    let redirect_url = format!("http://localhost:{addr}");
    let url = get_authorization_url(host, &redirect_url);

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
            Ok(_) => Response::html("<script>window.close();</script>"),
            Err(e) => Response::text(format!("mpsc channel error: {:?}", e)).with_status_code(500),
        }
    } else {
        Response::text("no jwt found").with_status_code(400)
    }
}

fn login_without_browser(host: &String) -> Result<String, String> {
    // if no browser, provide the user with a URL to open in their browser
    // redirecting to a page on the server that will show the code
    // then we ask the user to enter the code
    let mut jwt: String = String::new();

    let url = get_authorization_url(host, "/static/show_code.html");
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

fn get_authorization_url(host: &String, redirect_url: &str) -> String {
    format!("https://{host}/static/login.html?signInSuccessUrl={redirect_url}")
}
