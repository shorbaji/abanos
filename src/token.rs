use rouille::{Response, Server};
use std::path::PathBuf;
use std::sync::mpsc;

pub fn get_token(host: String) -> Result<String, String> {
    // get or create the path to ~/.abanos
    let path = config_path()?;

    let path = path.join("token");
    get_token_from_file(&path).or_else(|_| {
        login(host).inspect(|token| {
            save_token(&path, token).unwrap_or_else(|e| println!("Failed to save token: {}", e))
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

pub fn save_token(path: &PathBuf, token: &String) -> Result<(), String> {
    serde_json::to_string(&token)
        .map_err(|_| "Failed to serialize token".to_string())
        .and_then(|s| std::fs::write(path, s).map_err(|_| "Failed to write token file".to_string()))
}

fn login(host: String) -> Result<String, String> {
    // we either open a browser with the auth login url OR
    // we ask the user to open a browser with a url to login and enter the resulting code
    let url_base = format!("https://{host}/static/login.html");

    login_with_browser(&url_base).or_else(|_| login_without_browser(&url_base))
}

fn login_with_browser(url_base: &String) -> Result<String, String> {
    let (tx, rx) = mpsc::channel();

    let server = Server::new("localhost:0", move |request| handler(request, tx.clone()));

    let server = server.map_err(|e| format!("rouille error: {:?}", e))?;

    let addr = server.server_addr().port();

    let url = format!("{}?signInSuccessUrl=http://localhost:{}", url_base, addr);

    match open::that(url.as_str()) {
        Ok(_) => {
            println!("Waiting for browser login to complete ...");
            let (handle, sender) = server.stoppable();

            match rx.recv() {
                Ok(jwt) => {
                    sender
                        .send(())
                        .map_err(|e| format!("mpsc channel error: {:?}", e))?;
                    handle
                        .join()
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
    if let Some(jwt) = request.get_param("jwt") {
        match tx.send(jwt) {
            Ok(_) => Response::text("Login successful. You can close this tab now."),
            Err(e) => Response::text(format!("mpsc channel error: {:?}", e)).with_status_code(500),
        }
    } else {
        Response::text("No jwt found").with_status_code(400)
    }
}

fn login_without_browser(url_base: &String) -> Result<String, String> {
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
