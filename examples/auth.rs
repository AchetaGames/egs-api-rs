use webbrowser;
use std::io::{self};
use egs_api::EpicGames;

#[tokio::main]
async fn main() {
    if webbrowser::open("https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect").is_ok() {
        println!("Please enter the 'sid' value from the JSON response");
        let mut sid = String::new();
        let stdin = io::stdin(); // We get `Stdin` here.
        stdin.read_line(&mut sid).unwrap();
        sid = sid.trim().to_string();
        sid = sid.replace(|c: char| c == '"', "");
        let mut egs = EpicGames::new();

        match egs.auth_sid(sid.as_str()).await {
            None => { println!("No exchange token, cannot login.") }
            Some(exchange_token) => {
                egs.auth_code(exchange_token).await;
            }
        }
    }
}