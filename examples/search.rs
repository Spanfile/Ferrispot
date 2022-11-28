use dotenvy::dotenv;
use ferrispot::{client::SpotifyClientBuilder, model::ItemType, prelude::*};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let spotify_client =
        SpotifyClientBuilder::new(std::env::var("CLIENT_ID").expect("Spotify client ID not in environment"))
            .client_secret(std::env::var("CLIENT_SECRET").expect("Spotify client secret not in environment"))
            // a synchronous (blocking) client may be built with .build_sync() if the "sync" crate feature is enabled
            .build_async()
            .await
            .expect("failed to build Spotify client");

    let search_results = spotify_client
        .search("hatsune miku")
        .types([ItemType::Track])
        .send_async()
        .await
        .unwrap();

    let first_page = search_results.tracks().unwrap();

    println!("First page:");
    for track in first_page.items() {
        println!(
            "{} - {} ({}) [{}]",
            track.name(),
            track.artists().first().unwrap().name(),
            track.album().name(),
            track.id(),
        );
    }

    let second_page = first_page.next_page_async(&spotify_client).await.unwrap().unwrap();

    println!("\nSecond page:");
    for track in second_page.items() {
        println!(
            "{} - {} ({}) [{}]",
            track.name(),
            track.artists().first().unwrap().name(),
            track.album().name(),
            track.id(),
        );
    }

    let third_page = second_page.next_page_async(&spotify_client).await.unwrap().unwrap();

    println!("\nThird page:");
    for track in third_page.items() {
        println!(
            "{} - {} ({}) [{}]",
            track.name(),
            track.artists().first().unwrap().name(),
            track.album().name(),
            track.id(),
        );
    }
}
