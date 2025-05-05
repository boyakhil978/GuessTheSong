use tokio::sync::Mutex;
use std::sync::Arc;

use crate::api::{spotify_api::Track, SpotifyApi};
use futures::future::join_all;



pub async fn prepare_recommendation_list(spotify_api: Arc<Mutex<SpotifyApi>>, user: &str, tracks: &Vec<Track>) -> Vec<Track> {
    let mut tasks = vec![];

    for chunk in tracks.chunks(5) {
        let spotify_api_clone = Arc::clone(&spotify_api);
        let seed_tracks: Vec<String> = chunk.iter().filter_map(|t| t.id.clone()).collect();
        let fut = async move{
            spotify_api_clone.lock().await.get_recommendations(user, &seed_tracks).await
        };
        tasks.push(fut);
    }

    let results = join_all(tasks).await;

    let mut all_recommendations = Vec::new();

    for res in results {
        match res {
            Ok(mut recs) => all_recommendations.append(&mut recs),
            Err(e) => eprintln!("Error fetching recommendations: {:?}", e),
        }
    }

    return all_recommendations;
}

