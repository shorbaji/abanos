/// Connection
///
/// Connect the CLI to the abanos server
///
/// This module contains the `Connection` struct and its implementation. The `Connection` struct
/// represents a connection to the server. It has methods to check the health of the server and
/// to send expressions to the server for evaluation.
///
///
use lib::expr::Expr;
use lib::value::Value;

#[derive(Debug)]
pub struct Connection {
    host: String,
    port: u16,
}

impl Connection {
    pub fn new(host: String, port: u16) -> Connection {
        Connection { host, port }
    }

    /// Check the health of the server
    ///
    /// This method will send a GET request to the server's health endpoint and return
    /// the connection if the server responds with a 200 OK status code. Otherwise, it
    /// will return an error.
    pub fn healthcheck(&self) -> Result<&Self, String> {
        let url = format!("http://{}:{}/health", self.host, self.port);
        debug!("health check calling {url}");

        ureq::get(url.as_str())
            .call()
            .map_err(|e| format!("error: {}", e))
            .and_then(|response| {
                if response.status() == 200 {
                    debug!("healthcheck received response status 200 OK");
                    Ok(self)
                } else {
                    Err(format!(
                        "healthcheck: unexpected status code: {}",
                        response.status()
                    ))
                }
            })
    }

    /// Send an expression to the server for evaluation
    ///
    /// This method will send a POST request to the server's eval endpoint with the given
    /// expression and return the response from the server.

    #[allow(clippy::result_large_err)]
    pub fn send(&self, expr: Expr) -> Result<Value, String> {
        let url = format!("http://{}:{}/eval", self.host, self.port);

        match ureq::post(url.as_str()).send_json(expr) {
            Ok(response) => {
                if response.status() == 200 {
                    match response.into_json::<Result<Value, String>>() {
                        Ok(r) => r,
                        Err(e) => Err(format!("error: {}", e)),
                    }
                } else {
                    Err(format!("unexpected status code: {}", response.status()))
                }
            }
            Err(e) => Err(format!("error: {}", e)),
        }
    }
}