# GuessTheSong
This Repo is the Final Project for CS128 Honors, the code for this app is written in Rust

This project is a game using the spotify API and the saved songs of the user, randomly playing a song out of their library while the user(s) attempt to guess the song. The app could be used by the users with a group of friends or family. 

Group Members:

Akhil Boyapati (akhilb5)
Aarav Mittal (aaravm3)

Technical Overview: 

Spotify Api is used to fetch user's saved songs after prompting user to log in, then one of the saved songs are selected at random and used to drive the gameplay round. The song is played in the users device of choice, and it is left up to the user ensure that the answer remains a secret to him by minimizing the spotify app. The user attempts to guess the name, artist or album of the song and then reveals the answer.

The Spoitfy authentication is handled by the spotify_api.rs which returns a vector of a custom type of Track. This is then sent to the game.rs file which then selects a Track item at random (ensuring that the same item cannot be selected again). Then it uses this track to drive the game_round which also calls upon the spotify_api in order to actually play the selected song.

The fetching of the songs with the api is limited by the fact that the api has a limit of 50 songs per request, which is bypassed by multiple api calls each with a offset.





