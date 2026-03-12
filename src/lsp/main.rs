// Viper Language Server Protocol implementation
// This is a skeleton implementation

use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    id: Option<i64>,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    id: Option<i64>,
    result: Option<serde_json::Value>,
    error: Option<ResponseError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseError {
    code: i64,
    message: String,
}

fn main() {
    println!("Viper Language Server v{}", env!("CARGO_PKG_VERSION"));
    println!("Started on stdin/stdout");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        if let Ok(json_str) = line {
            if json_str.is_empty() {
                continue;
            }

            match serde_json::from_str::<Request>(&json_str) {
                Ok(request) => {
                    let response = handle_request(request);
                    if let Ok(response_json) = serde_json::to_string(&response) {
                        println!("{}", response_json);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse request: {}", e);
                }
            }
        }
    }
}

fn handle_request(request: Request) -> Response {
    match request.method.as_str() {
        "initialize" => handle_initialize(request.id),
        "textDocument/hover" => handle_hover(request.id, request.params),
        "textDocument/completion" => handle_completion(request.id, request.params),
        "textDocument/didOpen" => handle_did_open(request.id, request.params),
        "textDocument/didChange" => handle_did_change(request.id, request.params),
        "textDocument/didSave" => handle_did_save(request.id, request.params),
        "textDocument/didClose" => handle_did_close(request.id, request.params),
        "shutdown" => handle_shutdown(request.id),
        "exit" => {
            std::process::exit(0);
        }
        _ => Response {
            id: request.id,
            result: None,
            error: Some(ResponseError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
            }),
        },
    }
}

fn handle_initialize(id: Option<i64>) -> Response {
    Response {
        id,
        result: Some(serde_json::json!({
            "capabilities": {
                "textDocumentSync": 1,
                "hoverProvider": true,
                "completionProvider": {
                    "triggerCharacters": [".", "("]
                }
            },
            "serverInfo": {
                "name": "viper-lsp",
                "version": env!("CARGO_PKG_VERSION")
            }
        })),
        error: None,
    }
}

fn handle_hover(id: Option<i64>, params: Option<serde_json::Value>) -> Response {
    Response {
        id,
        result: Some(serde_json::json!({
            "contents": "Hover information not yet implemented"
        })),
        error: None,
    }
}

fn handle_completion(id: Option<i64>, params: Option<serde_json::Value>) -> Response {
    Response {
        id,
        result: Some(serde_json::json!([])),
        error: None,
    }
}

fn handle_did_open(id: Option<i64>, _params: Option<serde_json::Value>) -> Response {
    Response {
        id,
        result: Some(serde_json::json!(null)),
        error: None,
    }
}

fn handle_did_change(id: Option<i64>, _params: Option<serde_json::Value>) -> Response {
    Response {
        id,
        result: Some(serde_json::json!(null)),
        error: None,
    }
}

fn handle_did_save(id: Option<i64>, _params: Option<serde_json::Value>) -> Response {
    Response {
        id,
        result: Some(serde_json::json!(null)),
        error: None,
    }
}

fn handle_did_close(id: Option<i64>, _params: Option<serde_json::Value>) -> Response {
    Response {
        id,
        result: Some(serde_json::json!(null)),
        error: None,
    }
}

fn handle_shutdown(id: Option<i64>) -> Response {
    Response {
        id,
        result: Some(serde_json::json!(null)),
        error: None,
    }
}
