use dotenvy::dotenv;
use ferrispot::{
    client::SpotifyClientBuilder,
    model::{
        id::Id,
        user::{CurrentUser, PrivateUser},
    },
    prelude::*,
    scope::Scope,
};

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

    let incomplete_auth_code_client = spotify_client
        .authorization_code_client("http://localhost/callback")
        .scopes([Scope::UserReadEmail, Scope::UserReadPrivate])
        .show_dialog(true)
        .build();

    let authorize_url = incomplete_auth_code_client.get_authorize_url();
    println!("Authorize URL: {authorize_url}");

    let mut code = String::new();
    let mut state = String::new();

    println!("Code:");
    std::io::stdin().read_line(&mut code).unwrap();

    println!("State:");
    std::io::stdin().read_line(&mut state).unwrap();

    let user_client = incomplete_auth_code_client
        .finalize(code.trim(), state.trim())
        .await
        .expect("failed to finalize authorization code client");

    // optionally, if you have a valid refresh token (with the correct scope), you may use it as such:
    // let user_client = spotify_client
    //     .authorization_code_client_with_refresh_token("refresh token")
    //     .await
    //     .unwrap();

    let public_user = user_client
        .user_profile(Id::from_bare("smedjan").unwrap())
        .send_async()
        .await
        .unwrap();

    println!(
        "{} has {} followers",
        public_user.display_name().unwrap_or(public_user.id().as_str()),
        public_user.followers().total
    );

    let you = user_client.current_user_profile().send_async().await.unwrap();

    // this will fail if the user is PublicUser, which should never be returned from the endpoint
    let current_user: CurrentUser = you.clone().try_into().unwrap();

    if let Some(display_name) = current_user.display_name() {
        print!("Hi {display_name}! ");
    }

    println!("You have {} followers", current_user.followers().total);

    match PrivateUser::try_from(you) {
        Ok(private_user) => println!("Your subscription level is: {}", private_user.product()),
        Err(_) => println!("I don't have access to your private profile information"),
    }
}
