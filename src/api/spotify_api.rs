use oauth2::{
    AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use oauth2::basic::BasicClient;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use warp::Filter;
use open;
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct LikedSongsResponse {
    items: Vec<TrackItem>,
}

#[derive(Debug, Deserialize)]
struct RecommendationsResponse {
    tracks: Vec<Track>,
}

#[derive(Debug, Deserialize)]
struct TrackItem {
    track: Track,
}

#[derive(Debug, Deserialize)]
pub struct Track {
    pub id: Option<String>,
    pub name: String,
    pub artists: Vec<Artist>,
    pub album: Album,
}

#[derive(Debug, Deserialize)]
pub struct Artist {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Album {
    pub name: String,
}

pub struct SpotifyApi {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    users: Vec<(String, String)>,
}

impl SpotifyApi {
    pub fn new(client_id: &str, client_secret: &str, redirect_uri: &str) -> Self {
        SpotifyApi {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            redirect_uri: redirect_uri.to_string(),
            users: vec![]
        }
    }

    pub async fn login(&mut self, user: &str) {
        let client = BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            AuthUrl::new("https://accounts.spotify.com/authorize".to_string()).unwrap(),
            Some(TokenUrl::new("https://accounts.spotify.com/api/token".to_string()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(self.redirect_uri.clone()).unwrap());

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("user-read-private".to_string()))
            .add_scope(Scope::new("user-read-email".to_string()))
            .add_scope(Scope::new("user-library-read".to_string()))
            .add_scope(Scope::new("user-modify-playback-state".to_string()))
            .url();

        println!("Opening browser for login...");
        open::that(auth_url.to_string()).unwrap();

        let code_holder = Arc::new(Mutex::new(None));
        let state_holder = Arc::new(Mutex::new(None));

        let code_clone = Arc::clone(&code_holder);
        let state_clone = Arc::clone(&state_holder);

        let routes = warp::get()
            .and(warp::path("callback"))
            .and(warp::query::<HashMap<String, String>>())
            .map(move |params: HashMap<String, String>| {
                if let Some(code) = params.get("code") {
                    *code_clone.lock().unwrap() = Some(code.to_string());
                }
                if let Some(state) = params.get("state") {
                    *state_clone.lock().unwrap() = Some(state.to_string());
                }
                "You have succesfully logged in to GuessTheSong, you can close this tab and return to the app.".to_string()
            });
        
        let code_holder_clone = code_holder.clone();
        let (_addr, server) = warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], 3000), async move {
            while code_holder_clone.lock().unwrap().is_none() {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });

        tokio::spawn(server).await.unwrap();

        let code = code_holder.lock().unwrap().clone().unwrap();
        let state = state_holder.lock().unwrap().clone().unwrap();

        if state.as_str() != csrf_token.secret() {
            panic!("CSRF token mismatch!");
        }

        let token_result = client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .expect("Failed to get token");

        //println!("\nAccess token for {}: {}", user, token_result.access_token().secret());
        println!("Logged in!");
        

        self.users.push((user.to_string(), token_result.access_token().secret().to_string()));
        }

    pub async fn get_all_liked_songs(&self, user: &str) -> Result<Vec<Track>, Box<dyn std::error::Error>> {
        let token = self.users.iter()
            .find(|(username, _)| username == user)
            .map(|(_, token)| token)
            .ok_or("User not logged in")?;

        let client = reqwest::Client::new();
        let mut all_tracks = Vec::new();
        let mut offset = 0;

        loop {
            let resp = client
                .get("https://api.spotify.com/v1/me/tracks")
                .query(&[
                    ("limit", "50"),
                    ("offset", &offset.to_string())
                ])
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .send()
                .await?
                .error_for_status()?
                .json::<LikedSongsResponse>()
                .await?;

            if resp.items.is_empty() {
                break;
            }

            all_tracks.extend(resp.items.into_iter().map(|item| item.track));
            offset += 50;
        }

        Ok(all_tracks)
    }

    pub async fn get_recommendations(
        &self,
        user: &str,
        seed_tracks: &[String],
    ) -> Result<Vec<Track>, Box<dyn std::error::Error>> {
        let token = self.users.iter()
            .find(|(username, _)| username == user)
            .map(|(_, token)| token)
            .ok_or("User not logged in")?;
    
        let seed_param = seed_tracks.join(",");

    
        let client = reqwest::Client::new();
        let resp = client
            .get("https://api.spotify.com/v1/recommendations")
            .query(&[
                ("limit", "1"),
                ("seed_tracks", &seed_param),
            ])
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await?;
            eprintln!("Spotify API error ({}): {}", status, text);
            return Err("Spotify returned an error".into());
        }
        let parsed: RecommendationsResponse = resp.json().await?;
        Ok(parsed.tracks)
    
    }

    pub async fn play_song_with_id(&self, id: &str, user: &str) -> Result<(), Box<dyn std::error::Error>>{
        let token = self.users.iter()
            .find(|(username, _)| username == user)
            .map(|(_, token)| token)
            .ok_or("User not logged in")?;

        let uri = format!("spotify:track:{}", id);
        let client = reqwest::Client::new();

        let response = client
            .put("https://api.spotify.com/v1/me/player/play")
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .json(&serde_json::json!({
                "uris": [uri]
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            eprintln!("Failed to play song ({}): {}", status, text);
            return Err("Spotify play API failed".into());
        }

        Ok(())
    }
    
}







