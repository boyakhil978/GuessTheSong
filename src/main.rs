mod api ;
use std::io;

use api::SpotifyApi;

#[tokio::main]
async fn main() {
    const SPOTIFY_CLIENT_ID: &str  = "e50654fbca6d41208142e55d3e44dcab";
    const SPOTIFY_REDIRECT_URL: &str = "http://localhost:3000/callback";
    const SPOTIFY_CLIENT_SECRET: &str = "e4ba0d1291e540298692938d603fc759";
    let mut spotify_api = SpotifyApi::new(SPOTIFY_CLIENT_ID, SPOTIFY_CLIENT_SECRET, SPOTIFY_REDIRECT_URL);

    println!("Press Enter to Login To Spotify Premium Account, Press CTRL + C to exit");

    let mut _input = String::new();
    io::stdin()
        .read_line(&mut _input)
        .expect("Failed to read line");
    spotify_api.login("primary").await;

    
}


