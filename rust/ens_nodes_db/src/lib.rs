use std::collections::HashMap;

use aws_sdk_dynamodb::{Client, Error as DynamoError,  model::{AttributeValue, KeysAndAttributes, WriteRequest, PutRequest}};

pub struct NodeDomainPair {
    pub node: String,
    pub domain: String,
}

pub async fn add_domain(client: &Client, table_name: &str, ens_node: &str, ens_domain: &str) -> Result<(), DynamoError> {
    let request = client
        .put_item()
        .table_name(table_name)
        .item("node", AttributeValue::S(ens_node.into()))
        .item("domain", AttributeValue::S(ens_domain.into()));

    request.send().await?;

    Ok(())
}

pub async fn add_domains(client: &Client, table_name: &str, node_domains: &Vec<(String, String)>) -> Result<(), DynamoError> {
    let key = "node".to_string();
    let attribute = "domain".to_string();

    let mut builder = PutRequest::builder();

    for (node, domain) in node_domains {
        builder = builder.set_item(Some(HashMap::from(
            [
                (
                    key.to_string(),
                    AttributeValue::S(
                        node.to_string(),
                    ),
                ),
                (
                    attribute.to_string(),
                    AttributeValue::S(
                        domain.to_string()
                    ),
                ),
            ],
        )));
    }
   
    let request = client
        .batch_write_item()
        .request_items(
            table_name,
            vec![
                WriteRequest::builder()
                    .put_request(
                        builder.build()
                    )
                    .build(),
            ]
        );

    request.send().await?;

    Ok(())
}

pub async fn get_domain(client: &Client, table_name: &str, ens_node: &str) -> Result<Option<String>, DynamoError> {
    let result = client
        .get_item()
        .table_name(table_name)
        .key("node", aws_sdk_dynamodb::model::AttributeValue::S(ens_node.into()))
        .send()
        .await;

    let result = result?;

    let result = match result.item() {
        Some(res) => res,
        None => return Ok(None),
    };

    let result = match result.get("domain") {
        Some(res) => res,
        None => return Ok(None),
    };

    let result = match result.as_s() {
        Ok(res) => res,
        Err(_) => return Ok(None),
    };

    Ok(Some(result.to_string()))
}

pub async fn get_domains(client: &Client, table_name: &str, ens_nodes: &Vec<&str>) -> Result<Option<Vec<NodeDomainPair>>, DynamoError> {
    let key = "node".to_string();
    let attribute = "domain".to_string();
    
    let mut builder = KeysAndAttributes::builder();

    for node in ens_nodes {
        builder = builder.keys(HashMap::from([(
            key.clone(),
            AttributeValue::S(
                node.to_string(),
            ),
        )]));
    }

    let result = client
        .batch_get_item()
        .request_items(
            table_name,
            builder.build(),
        )
        .send()
        .await?;

    let result = match result.responses() {
        Some(result) => result,
        None => return Ok(None),
    };

    let result = match result.get(table_name) {
        Some(result) => result,
        None => return Ok(None),
    };

    let result = result.into_iter().filter_map(|x| {
        let node = match x.get(&key) {
            Some(result) => result,
            None => return None,
        };

        let node = match node.as_s() {
            Ok(result) => result,
            Err(_) => return None,
        };

        let domain = match x.get(&attribute) {
            Some(result) => result,
            None => return None,
        };

        let domain = match domain.as_s() {
            Ok(result) => result,
            Err(_) => return None,
        };

        Some(NodeDomainPair {
            node: node.clone(),
            domain: domain.clone()
        })
    }).collect();

    Ok(Some(result))
}
