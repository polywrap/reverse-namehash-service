use std::env;

use http::Response;
use lambda_runtime::{service_fn, Error};
use serde_derive::*;
use aws_sdk_dynamodb::Client;
use lambda_http::{run, http::StatusCode, IntoResponse, Request, RequestExt};
use serde_json::json;

#[derive(Debug, Serialize)]
struct NodeDomainPair {
    pub node: String,
    pub domain: String
}

type RequestBody = Option<Vec<Option<String>>>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let func = service_fn(handler);
    run(func).await?;

    Ok(())
}

async fn handler(event: Request) -> Result<impl IntoResponse, Error>  {
    let ens_nodes_table_name = env::var("ENS_NODES_TABLE").expect("ENS_NODES_TABLE is not set");
  
    let body = event.payload::<RequestBody>()?.unwrap();

    let nodes = &body;
    let nodes = match nodes {
        Some(nodes) => nodes,
        None => return response(
            StatusCode::BAD_REQUEST,
            None
        )
    };

    let nodes = nodes.into_iter()
        .filter_map(|x| match x.as_ref() {
            Some(x) => if x.len() == 66 && x.starts_with("0x") { Some(x) } else { None },
            None => None
        })
        .map(|x| x.as_ref())
        .collect();

    let client = get_dynamo_client().await;
    
    let result = ens_nodes_db::get_domains(&client, &ens_nodes_table_name, &nodes).await?;

    let results = if let Some(result) = result {
        result.into_iter()
            .map(|x| NodeDomainPair {
                node: x.node,
                domain: x.domain
            })
            .collect()
    } else {
        vec![]
    };

    response(
        StatusCode::OK,
        Some(json!(results).to_string())
    )
}

async fn get_dynamo_client() -> Client {
    let shared_config = aws_config::from_env().region("us-east-1").load().await;
    Client::new(&shared_config)
}

fn response(status_code: StatusCode, body: Option<String>) -> Result<impl IntoResponse, Error> {
    let response = Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(if let Some(body) = body { body } else { "".to_string() })
        .map_err(Box::new)?;
   
    Ok(response)
}
