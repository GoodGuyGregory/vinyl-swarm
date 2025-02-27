use std::collections::HashMap;
use base64::{encode};
use serde::{Deserialize, Serialize};

/// REQWEST Basic Tutorial Guide: 
/// 
/// Link: https://blog.logrocket.com/making-http-requests-rust-reqwest/
use reqwest::{self, header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}, Response};


 // deserialize this nonsense into actual human readable data that might be interesting.

#[derive(Serialize, Deserialize, Debug)]
struct ExternalUrls {
    spotify: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Artist {
    name: String,
    external_urls: ExternalUrls,
}

#[derive(Serialize, Deserialize, Debug)]
struct Album {
    name: String,
    artists: Vec<Artist>,
    external_urls: ExternalUrls,
}

#[derive(Serialize, Deserialize, Debug)]
struct Track {
    name: String,
    href: String,
    popularity: u32,
    // nested struct. so nice.
    album: Album,
    external_urls: ExternalUrls,
}

#[derive(Serialize, Deserialize, Debug)]
struct Items<T> {
    items: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug)]
struct APIResponse {
    tracks: Items<Track>,
}

// tokio let's us use "async" on our main function
#[tokio::main]
async fn main() {
    println!("Using reqwest in Rust");

    let _response  = reqwest::get("https://api.spotify.com/v1/search")
            .await
            // each response is wrapped in a `Result` type 
            // well unwrap here for simplicity
            .unwrap()
            .text()
            .await;

    // println!("Got {:?}", response);

    // add a client to specify content-type accept and pass authorization headers
    let client = reqwest::Client::new();
    let _client_resp = client
        .get("https://api.spotify.com/v1/search")
        // confirm the request using send
        .send()
        .await
        // the rest is the same here!
        .unwrap()
        .text()
        .await;

    // println!("Client Response {:?}", client_resp);

    // finally authorization:

    // Spotify Access Token Documentation: 
    // https://developer.spotify.com/documentation/web-api/concepts/access-token

    // CLI Client Credentials Flow: 
    // https://developer.spotify.com/documentation/web-api/tutorials/client-credentials-flow

    // get the client credentials

    // let mut params_body = HashMap::new();
    // // add the body required
    // params_body.insert("grant_type","client_credentials" ); 

    // create the base64 encoded string 
    // let encoded_client_id_secret = encode("0cb3ce7b94834781a455de4869aba0d9:96c6fb28271c45bea2a0ff8fb2a28630");
    // let basic_auth_encoded = format!("{} {}", "Basic ", encoded_client_id_secret);

    // let bearer_token = client.post("https://accounts.spotify.com/api/token")
    //     .header(AUTHORIZATION, &basic_auth_encoded )
    //     .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
    //     .header(ACCEPT, "application/json")
    //     .form(&params_body)
    //     .send()
    //     .await
    //     .unwrap()
    //     .text()
    //     .await;

    // println!("Access Token: {:?}", bearer_token);
    let bearer_token = "Bearer BQDs8pD550-EzMjc2mFGkGpOZvA86XnHuSR1S1VGKXMp2NMawaEa3Legcui-6_L-7Jtnw1KbYnnyMr60rERWu7bt59PiwvfwqBd5_jkOaL6Wxv3p_gK_a22GMhjd_QTvBaGZZtvH1kA";

    let url = format!(
        "https://api.spotify.com/v1/search?query={query}&type=artist&limit=5&market=US", query = "Khruangbin"
    );

    // reqwest crate: https://docs.rs/reqwest/0.11.5/reqwest/#making-post-requests-or-setting-request-bodies

    let authorized_resp = client
        .get(url)
        .header(AUTHORIZATION, bearer_token)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .send()
        .await
        .unwrap()
        //get the text 
        .text()
        .await;

    // hot garbage json. yuck...
    println!("{:?}", authorized_resp);
    
    
    


}
