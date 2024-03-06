

## HTTP, TCP and gRPC Rust Clients

```rust
//===== http requests setup
let http_endpoint = "https://api.panel.conse.app";
let api = Client::new_http(endpoint);
let res_post = api.send_post("/user/new", map_body, header);
let res_get = api.send_get("/users/get/?from=0&to=10", header);
//===== tcp streamin setup
let tcp_endpoint = "0.0.0.0:2455";
let streamer = Clinet::new_tcp(tcp_endpoint);
while let Some(socket) = streamer.await{
    // use socket to for write and read
    // ...
}
```
