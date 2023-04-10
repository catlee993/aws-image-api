use std::{env, io};
use std::error::Error as StdError;
use std::fs::File;
use std::io::Read;

use actix_multipart::Multipart;
use actix_web::{App, HttpResponse, HttpResponseBuilder, HttpServer, web};
use actix_web::http::{Error, header};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client as S3Client, primitives::ByteStream};
use aws_sdk_s3::Client as s3Client;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::put_object::{PutObjectError, PutObjectOutput};
use aws_sdk_ssm::{Client as SsmClient, Error as SsmError};
use chrono::{Duration, Utc};
use cloudfront_sign::{get_signed_url, SignedOptions};
use dotenv::dotenv;
use env_logger::Env;
use futures::StreamExt;
use log::error;
use rusoto_signature::{Region, SignedRequest};
use serde::{Deserialize, Serialize};

// TODO something with websockets, for fun
//use futures::StreamExt;

//use tungstenite::Message;
//use tokio_tungstenite::WebSocketStream;

// type Tx = mpsc::UnboundedSender<String>;
//
// async fn ws_handler(ws: WebSocketStream<tokio::net::TcpStream>, tx: Tx) {
//     let (ws_tx, mut ws_rx) = ws.split();
//     while let Some(result) = ws_rx.next().await {
//         match result {
//             Ok(Message::Text(message)) => {
//                 tx.send(format!("Received message: {}", message)).unwrap();
//             }
//             Ok(Message::Close(_)) => {
//                 break;
//             }
//             _ => {}
//         }
//     }
// }

#[derive(Serialize, Deserialize)]
struct UploadResponse {
    success: bool,
    message: String,
}

#[derive(Serialize, Deserialize)]
struct GetImagesResponse {
    success: bool,
    urls: Vec<String>,
}

const PATH_PREFIX: &str = "uploaded-crap";
const S3_BUCKET_KEY: &str = "dabucks";
const KEY_PAIR_ID_KEY: &str = "KEY_PAIR_ID";
const PRIVATE_KEY_PATH_KEY: &str = "PRIVATE_KEY_PATH";
const S3_DOMAIN_KEY: &str = "S3_DOMAIN";

async fn load_parameters() -> Result<(), SsmError> {
    let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = SsmClient::new(&shared_config);

    let resp = client.get_parameters_by_path().
        path("/").
        recursive(true).
        with_decryption(true).
        send().await?;

    for param in resp.parameters().unwrap().iter() {
        env::set_var(param.name().unwrap(), param.value().unwrap_or_default());
    }

    Ok(())
}

async fn post_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();

        let content_type = field.content_disposition().to_string();
        let filename = content_type
            .split("filename=")
            .collect::<Vec<&str>>()[1]
            .replace("\"", "");

        let mut data = Vec::new();
        while let Some(chunk) = field.next().await {
            let bytes = chunk.unwrap();
            data.extend_from_slice(&bytes);
        }

        let resp = upload_to_s3(data, filename.as_str()).await;
        return match resp {
            Ok(_) => {
                handle_upload_ok_resp()
            }
            Err(err) => {
                error!("{}", err);
                let api_response = UploadResponse {
                    success: false,
                    message: "Something went wrong...".to_string(),
                };
                Ok(headers(HttpResponse::InternalServerError()).json(api_response))
            }
        };
    }

    handle_upload_ok_resp()
}

async fn upload_to_s3(data: Vec<u8>, key: &str) -> Result<PutObjectOutput, SdkError<PutObjectError>> {
    let client = get_s3_client().await;
    let resp = client
        .put_object()
        .bucket(env::var(S3_BUCKET_KEY).unwrap())
        .key(PATH_PREFIX.to_owned() + "/" + key)
        .body(ByteStream::from(data))
        .content_type("image/png")
        .send()
        .await;

    return resp;
}

async fn get_s3_items() -> Result<HttpResponse, Error> {
    let client = get_s3_client().await;
    let resp = client
        .list_objects_v2()
        .bucket(env::var(S3_BUCKET_KEY).unwrap())
        .send()
        .await;

    return match resp {
        Ok(stuff) => {
            let mut keys: Vec<String> = Vec::new();
            match stuff.contents() {
                Some(thing) => {
                    for obj in thing {
                        keys.push(obj.key().unwrap().to_string())
                    }
                }
                None => {
                    // do nothing, don't care
                }
            }

            let bucket_name = env::var(S3_BUCKET_KEY).unwrap();
            let s3_domain = env::var(S3_DOMAIN_KEY).unwrap();
            let key_pair_id = env::var(KEY_PAIR_ID_KEY).unwrap();
            let private_key_result = get_private_key(
                env::var(PRIVATE_KEY_PATH_KEY).unwrap()
            );

            let private_key = match private_key_result {
                Ok(k) => { k }
                Err(err) => {
                    error!("{}", err);
                    return handle_image_error();
                }
            };

            let mut urls: Vec<String> = Vec::new();
            for k in keys {
                let s3_url = format!("https://{}/{}", s3_domain, k);
                let signed = sign_url(
                    s3_url.as_str(),
                    key_pair_id.to_string(),
                    private_key.to_string(),
                ).await;
                urls.push(signed.unwrap());
            }

            let api_response = GetImagesResponse {
                success: true,
                urls,
            };
            Ok(image_header(headers(HttpResponse::Ok())).json(api_response))
        }
        Err(err) => {
            error!("{}", err);
            handle_image_error()
        }
    };
}

fn headers(mut resp: HttpResponseBuilder) -> HttpResponseBuilder {
    resp.append_header((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
        .append_header((header::ACCESS_CONTROL_ALLOW_METHODS, "POST"))
        .append_header((header::ACCESS_CONTROL_ALLOW_HEADERS, "content-type"));

    resp
}

fn image_header(mut resp: HttpResponseBuilder) -> HttpResponseBuilder {
    resp.append_header((header::CONTENT_TYPE, "image/png"));

    resp
}

fn handle_image_error() -> Result<HttpResponse, Error> {
    let api_response = GetImagesResponse {
        success: false,
        urls: vec![],
    };
    Ok(headers(HttpResponse::InternalServerError()).json(api_response))
}

fn handle_upload_ok_resp() -> Result<HttpResponse, Error> {
    let api_response = UploadResponse {
        success: false,
        message: "Nothing uploaded...".to_string(),
    };
    Ok(headers(HttpResponse::Ok()).json(api_response))
}

async fn sign_url(s3_url: &str, key_pair_id: String, private_key: String) -> Result<String, Box<dyn StdError>> {
    let options = SignedOptions {
        key_pair_id,
        private_key,
        ..Default::default()
    };
    let signed_url = get_signed_url(s3_url, &options).unwrap();

    Ok(signed_url)
}

fn get_private_key(path: String) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents.trim().to_string())
}

async fn get_s3_client() -> S3Client {
    let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
    let s3config = aws_config::from_env().region(region_provider).load().await;
    let client = s3Client::new(&s3config);

    client
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let _ = load_parameters().await;
    dotenv().ok();

    HttpServer::new(||
        App::new()
            .service(
                web::resource("/junk")
                    .route(web::post().to(post_file)).
                    route(web::get().to(get_s3_items))
            )).bind("0.0.0.0:8080")?
        .run()
        .await
}