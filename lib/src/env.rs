use crate::builtin::Builtin;
use crate::continuation::{Arg, Closure, Context, Continuation};
use crate::value::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum EnvMsg {
    Get(String, oneshot::Sender<Option<Value>>),
    Set(String, Value, oneshot::Sender<Option<Value>>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Env {
    #[serde(skip)]
    Dust {
        path: Vec<String>,
        tx: Option<mpsc::Sender<EnvMsg>>,
    },
    #[serde(skip)]
    Local {
        hm: HashMap<String, Value>,
        parent: Option<Arc<Mutex<Env>>>,
    },
}

impl Default for Env {
    fn default() -> Self {
        Env::Local {
            hm: HashMap::new(),
            parent: None,
        }
    }
}

impl Env {
    pub fn new_dust(path: Vec<String>, tx: mpsc::Sender<EnvMsg>) -> Self {
        Env::Dust { path, tx: Some(tx) }
    }

    pub fn new_local(parent: Option<Arc<Mutex<Env>>>) -> Self {
        Env::Local {
            hm: HashMap::new(),
            parent,
        }
    }

    pub async fn get(
        &self,
        symbol: &String,
        context: &Context,
        k: Box<Closure>,
    ) -> Result<Continuation, String> {
        match self {
            Env::Local { ref hm, ref parent } => {
                if let Some(value) = hm.get(symbol) {
                    Ok(Continuation {
                        closure: *k,
                        arg: Arg::Value(value.clone()),
                    })
                } else if let Some(parent) = parent {
                    let closure = Closure::Lookup {
                        r: parent.clone(),
                        context: context.clone(),
                        k: Some(k),
                    };
                    let arg = Arg::Value(Value::Symbol(symbol.clone()));
                    Ok(Continuation { closure, arg })
                } else {
                    Err("not found".to_string())
                }
            }
            Env::Dust { ref path, ref tx } => {
                let (otx, orx) = oneshot::channel();

                match tx {
                    Some(tx) => {
                        let command = EnvMsg::Get(format!("{}/{}", path.join("/"), symbol), otx);

                        let _ = tx.send(command).await;

                        if let Ok(Some(value)) = orx.await {
                            Ok(Continuation {
                                closure: *k,
                                arg: Arg::Value(value),
                            })
                        } else {
                            match self.standard(symbol).await {
                                Some(value) => Ok(Continuation {
                                    closure: *k,
                                    arg: Arg::Value(value),
                                }),
                                None => Err("not found".to_string()),
                            }
                        }
                    }
                    None => Err("error looking up symbol in dust".to_string()),
                }
            }
        }
    }

    pub async fn standard(&self, symbol: &String) -> Option<Value> {
        let builtin = match symbol.as_str() {
            "+" => Builtin {
                name: symbol.to_string(),
                min_args: 1,
                max_args: None,
                f: crate::stdlib::number::add,
            },
            "call/cc" => Builtin {
                name: symbol.to_string(),
                min_args: 1,
                max_args: Some(1),
                f: crate::stdlib::control::call_cc,
            },
            _ => return None,
        };

        Some(Value::Builtin(builtin))
    }

    pub async fn set(&mut self, symbol: &str, value: &Value) -> Option<Value> {
        match self {
            Env::Dust { path, tx } => match tx {
                Some(tx) => {
                    let (otx, orx) = oneshot::channel();
                    let command =
                        EnvMsg::Set(format!("{}/{}", path.join("/"), symbol), value.clone(), otx);

                    let _ = tx.send(command).await;

                    if let Ok(Some(value)) = orx.await {
                        Some(value)
                    } else {
                        None
                    }
                }
                None => None,
            },

            Env::Local { hm, parent: _ } => hm.insert(symbol.to_string(), value.clone()),
        }
    }
}
