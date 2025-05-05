
/* pub fn main_gameplay(tracks: Vec<Tracks>) {
    pick a random track
    call game_round(track)

    user has to press enter to reveal answer/name of the song /album title / artist 
    
    once used, cannot be used again 
    */

use rand::Rng;
use rand::rng;
use tokio::task;
use std::io;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::api::{spotify_api::Track, SpotifyApi};

const PRIMARY_STRING: &str = "primary";


pub async fn main_gameplay(mut tracks: Vec<Track>, spotify_api_arc: Arc<Mutex<SpotifyApi>>) {
    let mut rng = rng(); 

    while !tracks.is_empty() {
        //Pick a random index 
        let idx = rng.random_range(0..tracks.len());

        // Remove that track so we don't use it again 
        let track = tracks.swap_remove(idx);

        
        game_round(&track, Arc::clone(&spotify_api_arc)).await; 

    }

}

pub async fn game_round(track: &Track, spotify_api_arc: Arc<Mutex<SpotifyApi>>) {
    let track_id = track.id.clone()
        .expect("Track must have an ID to play");
    
    
    let _ = spotify_api_arc.lock().await.play_song_with_id(&track_id, PRIMARY_STRING).await;

    println!("Now playing. Press Enter to stop and reveal the answer...");

    let _ = task::spawn_blocking(|| {
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).ok();
    })
    .await;


    //Reveal track information
    let artists = track.artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ");

    println!("-------------------------------------------------------------------------");
    println!("\n\"{}\"", track.name);
    println!(" by {}", artists);
    println!(" from the album \"{}\"\n", track.album.name);
    println!("-------------------------------------------------------------------------");

    

}


