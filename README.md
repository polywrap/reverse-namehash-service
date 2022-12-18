# Reverse namehash service
The reverse namehash service is a service built with Rust and the Serverless framework that can be used to resolve ENS nodes to domain names.

## Endpoints
`POST /add` - Add new ENS domains to the database.

Example request body:
```json
[
  "example.eth",
  "hello.world.eth"
]
```

`POST /resolve` - Get the domain names for given ENS nodes.

Example request body:
```json
[
  "0xa2f...",
  "0xcf1..."
]
```

Example response body:
```json
[
  {
    "node": "0xa2f...",
    "domain": "example.eth",
  },
  {
    "node": "0xcf1...",
    "domain": "hello.world.eth"
  }
]
```
