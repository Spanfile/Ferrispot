use dotenvy::dotenv;
use ferrispot::{
    client::{SpotifyClientBuilder, UnscopedClient},
    model::{
        album::CommonAlbumInformation,
        artist::CommonArtistInformation,
        track::{CommonTrackInformation, FullTrackInformation},
        ItemType,
    },
};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let spotify_client =
        SpotifyClientBuilder::new(std::env::var("CLIENT_ID").expect("Spotify client ID not in environment"))
            .client_secret(std::env::var("CLIENT_SECRET").expect("Spotify client secret not in environment"))
            .build()
            .await
            .expect("failed to build Spotify client");

    let first_page = spotify_client
        .search("hatsune miku")
        .types([ItemType::Track])
        .send()
        .await
        .unwrap();

    println!("First page:");
    for track in first_page.tracks().unwrap().items {
        println!(
            "{} - {} ({})",
            track.name(),
            track.artists().first().unwrap().name(),
            track.album().name()
        );
    }

    let second_page = spotify_client
        .search("hatsune miku")
        .types([ItemType::Track])
        // there are 20 items in a page by default, so offset the search by 20 to get the second page
        .offset(20)
        .send()
        .await
        .unwrap();

    println!("\nSecond page:");
    for track in second_page.tracks().unwrap().items {
        println!(
            "{} - {} ({})",
            track.name(),
            track.artists().first().unwrap().name(),
            track.album().name()
        );
    }
}
