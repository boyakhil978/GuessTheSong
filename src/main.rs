mod api ;
use std::io;
use tokio::sync::Mutex;
use std::sync::Arc;
use api::SpotifyApi;
//use utility::prepare_recommendation_list;
mod utility;
mod game;


#[tokio::main]
async fn main() {
    const SPOTIFY_CLIENT_ID: &str  = "e50654fbca6d41208142e55d3e44dcab";
    const SPOTIFY_REDIRECT_URL: &str = "http://localhost:3000/callback";
    const SPOTIFY_CLIENT_SECRET: &str = "e4ba0d1291e540298692938d603fc759";
    const PRIMARY_STRING: &str = "primary";
    

    let spotify_api = SpotifyApi::new(SPOTIFY_CLIENT_ID, SPOTIFY_CLIENT_SECRET, SPOTIFY_REDIRECT_URL);
    let spotify_api_arc = Arc::new(Mutex::new(spotify_api));
    println!("Press Enter to Login To Spotify Premium Account, Press CTRL + C to exit");


    let mut _input = String::new();
    io::stdin()
        .read_line(&mut _input)
        .expect("Failed to read line");

    let spotify_api_arc_clone = Arc::clone(&spotify_api_arc);
    spotify_api_arc_clone.lock().await.login(PRIMARY_STRING).await;

    let tracks = spotify_api_arc_clone.lock().await.get_all_liked_songs(PRIMARY_STRING).await.unwrap();

    let mut final_tracks_pool = Vec::new();
    final_tracks_pool.extend(tracks);

    // Code removed due to spotify depreceating the reccomendations feature in their api
    // let recomendations: Vec<Track> = prepare_recommendation_list(spotify_api_arc_clone, PRIMARY_STRING, &tracks).await;
    // println!("Recomendations Mixed in :{}", recomendations.len());
    // final_tracks_pool.extend(recomendations);


    //DEBUG
    // println!("{}", final_tracks_pool[0].name);
    // println!("{}", final_tracks_pool[final_tracks_pool.len() - 1].name);
    // let _ = spotify_api_arc_clone.lock().await.play_song_with_id(final_tracks_pool[0].id.as_ref().unwrap(), PRIMARY_STRING).await;
    
    //prompt user to play any song first on their device and set their screen to hide the spotify window
    //TODO
    println!("If you want to end the game, press CTRL-C.");
    
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("\nCtrl+C received.");
        }
        _ = game::main_gameplay(final_tracks_pool, spotify_api_arc) => {
            println!("Game session ended.");
        }
    }

    println!("Shutting down. Hope you enjoyed playing!");
    return; 

}


