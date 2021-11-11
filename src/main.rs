mod my_service;
mod self_polling;

use axum::extract::Extension;
use axum::{extract::ConnectInfo, routing::get, Router};
use futures::future::poll_fn;
use hyper::client::HttpConnector;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, Http};
use hyper::Body;
use std::net::SocketAddr;
use std::pin::Pin;
use tokio::net::TcpListener;
use tower::MakeService;

use crate::my_service::MyMaker;

type Client = hyper::client::Client<HttpConnector, Body>;

async fn handler(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> String {
    format!("Hello {}\n", addr)
}

async fn handler1(Extension(addr): Extension<i32>) -> String {
    format!("Hello {}\n", addr)
}

#[tokio::main]
async fn main() {
    let app1 = Router::new().route("/", get(handler));
    let mut my_service_maker: MyMaker<_, SocketAddr> = MyMaker::new(app1.clone());
    let norm_server = axum::Server::bind(&"0.0.0.0:3000".parse().unwrap()).serve(my_service_maker);

    let server_addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    let mut tcp_listener = TcpListener::bind(server_addr).await.unwrap();
    let mut tcp_listener = AddrIncoming::from_listener(tcp_listener).unwrap();
    let new_server = tokio::spawn(async move {
        loop {
            let stream = poll_fn(|cx| Pin::new(&mut tcp_listener).poll_accept(cx))
                .await
                .unwrap()
                .unwrap();

            let mut my_service_maker: MyMaker<_, SocketAddr> = MyMaker::new(handler.clone());
            let mut axum_server_maker = app1.into_make_service_with_connect_info::<SocketAddr, _>();
            let axum_made_service = axum_server_maker.make_service(&stream).await.unwrap();
            let my_made_service = my_service_maker.make_service(&stream).await.unwrap();

            let _ = Http::new()
                .serve_connection(stream, axum_made_service)
                .with_upgrades()
                .await;
        }
    });
    new_server.await.unwrap();
}
