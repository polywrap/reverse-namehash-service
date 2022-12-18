use std::env;

use ethers_providers::ens::namehash;
use http::Response;
use lambda_runtime::{service_fn, Error};
use serde_derive::*;
use aws_sdk_dynamodb::Client;
use lambda_http::{run, http::StatusCode, IntoResponse, Request, RequestExt};
use serde_json::json;

#[derive(Debug, Serialize)]
struct ResponseWithBody {
    pub body: String,
}

type RequestBody = Option<Vec<String>>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let func = service_fn(handler);
    run(func).await?;

    Ok(())
}

async fn handler(event: Request) -> Result<impl IntoResponse, Error> {
    let ens_nodes_table_name = env::var("ENS_NODES_TABLE").expect("ENS_NODES_TABLE is not set");

    let body = event.payload::<RequestBody>()?.unwrap();

    let domains = if let Some(domains) = body {
        domains
    } else {
        return response(
            StatusCode::BAD_REQUEST,
            "".to_string()
        );
    };

    let node_domains = domains.into_iter()
        .map(|x| {
            let node = namehash(&x);
            let node = hex::encode(node.as_bytes());
            let node = "0x".to_string() + &node;

            (node, x.clone())
        }).collect();

    let client = get_dynamo_client().await;

    ens_nodes_db::add_domains(&client, &ens_nodes_table_name, &node_domains).await?;

    response(
        StatusCode::OK,
        json!(ResponseWithBody {
            body: "Success".to_string()
        }).to_string()
    )
}

async fn get_dynamo_client() -> Client {
    let shared_config = aws_config::from_env().region("us-east-1").load().await;
    Client::new(&shared_config)
}

fn response(status_code: StatusCode, body: impl Into<String>) -> Result<impl IntoResponse, Error> {
    let response = Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(body.into())
        .map_err(Box::new)?;

    Ok(response)
}
