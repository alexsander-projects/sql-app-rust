use actix_web::{http::header, web, App, HttpResponse, HttpServer, Responder};
use askama::Template;
use async_std::net::TcpStream;
use dotenv::dotenv;
use std::env;
use tiberius::{Client, Config};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    products: Vec<Products>,
}

#[derive(serde::Serialize)]
struct Products {
    ProductID: i32,
    ProductName: String,
    Quantity: String,
}

async fn get_products() -> impl Responder {
    dotenv().ok();
    let connection_string = env::var("CONNECTION_STRING").expect("CONNECTION_STRING must be set");

    let config = Config::from_ado_string(&connection_string).unwrap();
    let tcp = TcpStream::connect(config.get_addr()).await.unwrap();
    tcp.set_nodelay(true).unwrap();

    let mut client = Client::connect(config, tcp).await.unwrap();

    let mut products = Vec::<Products>::new();

    let rows = client
        .query(
            "SELECT ProductID, ProductName, Quantity FROM Products",
            &[&1i32],
        )
        .await
        .unwrap()
        .into_first_result()
        .await
        .unwrap();

    for row in rows {
        let ProductID: i32 = row.get("ProductID").unwrap();
        let ProductName: &str = row.get("ProductName").unwrap();
        let Quantity: i32 = row.get("Quantity").unwrap();

        products.push(Products {
            ProductID: ProductID.to_string().parse().unwrap(),
            ProductName: ProductName.to_string(),
            Quantity: Quantity.to_string(),
        });
    }

    let template = IndexTemplate { products };
    let rendered = template.render().unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(rendered)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::get().to(get_products)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
