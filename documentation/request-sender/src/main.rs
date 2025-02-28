use std::collections::HashMap;
use base64::{encode};
use serde_json::Result;
use serde::{Deserialize, Serialize};
use dotenv::dotenv;
use std::env;

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

#[derive(serde::Serialize, serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SpotifyClient {
    client_id: String,
    client_secret: String
}

fn gather_client_secrets() -> SpotifyClient {
    dotenv().ok();

    // Access the environment variables
    let client_id = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID must be set");
    let client_secret = env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET must be set");

    // build the ClientSecret
    let client_secret = SpotifyClient {
        client_id: client_id.to_owned(),
        client_secret: client_secret.to_owned()
    };
    client_secret
}

async fn _generate_basic_spotify_request() {
        let response  = reqwest::get("https://api.spotify.com/v1/search")
            .await
            // each response is wrapped in a `Result` type 
            // well unwrap here for simplicity
            .unwrap()
            .text()
            .await;

    println!("Got {:?}", response);
}

async fn _generate_spotify_request_with_client() {
    let client = reqwest::Client::new();
    let client_resp = client
        .get("https://api.spotify.com/v1/search")
        // confirm the request using send
        .send()
        .await
        // the rest is the same here!
        .unwrap()
        .text()
        .await;

    println!("Client Response {:?}", client_resp);
}

/// get_spotify_authorization
/// returns the authentication based on the client's secret
async fn get_spotify_authorization(client_id: &str, client_secret: &str, ) -> std::result::Result<std::string::String, reqwest::Error>{
    // Spotify Access Token Documentation: 
    // https://developer.spotify.com/documentation/web-api/concepts/access-token

    // CLI Client Credentials Flow: 
    // https://developer.spotify.com/documentation/web-api/tutorials/client-credentials-flow

    // get the client credentials
    let client = reqwest::Client::new();
    let mut params_body = HashMap::new();
    // // add the body required
    params_body.insert("grant_type","client_credentials" ); 

    // create the base64 encoded string 
    let combined_client_credentials: String = format!("{}:{}", client_id, client_secret);
    let encoded_client_id_secret = encode(&combined_client_credentials);
    let basic_auth_encoded = format!("{} {}", "Basic ", encoded_client_id_secret);

    let bearer_token = client.post("https://accounts.spotify.com/api/token")
        .header(AUTHORIZATION, &basic_auth_encoded )
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(ACCEPT, "application/json")
        .form(&params_body)
        .send()
        .await
        .unwrap()
        .text()
        .await;

    println!("Access Token: {:?}", bearer_token);
    bearer_token
}

// tokio let's us use "async" on our main function
#[tokio::main]
async fn main() {
    println!("Using reqwest in Rust");


    let spotify_client = gather_client_secrets();

    // make a basic request
    // generate_basic_spotify_request()
    //     .await;

    // add a client to specify content-type accept and pass authorization headers
    // generate_spotify_request_with_client()
    //     .await;

    // finally authorization:
    let resp_token = get_spotify_authorization(&spotify_client.client_id, &spotify_client.client_secret)
                                    .await
                                    .unwrap();

                                // returns OK get the value from the serde_json
    let bearer_token_json:TokenResponse = serde_json::from_str(&resp_token).unwrap();
    
    let bearer_token = format!("Bearer {access_token}", access_token=bearer_token_json.access_token);
    println!("Token: {}", &bearer_token);

    let url = format!(
        "https://api.spotify.com/v1/search?query={query}&type=artist&limit=5&market=US", query = "Khurangbin"
    );

    // reqwest crate: https://docs.rs/reqwest/0.11.5/reqwest/#making-post-requests-or-setting-request-bodies
    let client = reqwest::Client::new();
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

#[cfg(test)]
mod tests {
    use super::*;
    // add the async library 
    use tokio::test;

    #[test]
    async fn test_can_access_credentials() {
        dotenv().ok();

          // Access the environment variables
        let client_id = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID must be set");
        let client_secret = env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET must be set");
        
        let spotify_client = gather_client_secrets();

        assert_eq!(spotify_client.client_id, client_id);
        assert_eq!(spotify_client.client_secret, client_secret);
    }

    #[test]
    async fn test_response_token_granted() {
          // Access the environment variables
        
        let spotify_client = gather_client_secrets();
        
        let resp_token =  get_spotify_authorization(&spotify_client.client_id, &spotify_client.client_secret)
                                    .await
                                    .unwrap();

                                // returns OK get the value from the serde_json
        let bearer_token_json:TokenResponse = serde_json::from_str(&resp_token).unwrap();

        // check the access_token 
        assert!(bearer_token_json.access_token.len() > 0 );
        // check token-type
        assert_eq!(bearer_token_json.token_type, "Bearer");
        // expires_in should be 3600
        assert_eq!(bearer_token_json.expires_in, 3600);
    }

}
