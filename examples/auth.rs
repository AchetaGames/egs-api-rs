use webbrowser;
use std::io::{self, Read};
use egs_api::EpicAPI;

#[tokio::main]
async fn main() {
    if webbrowser::open("https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect").is_ok() {
        println!("Please enter the 'sid' value from the JSON response");
        let mut sid = String::new();
        let mut stdin = io::stdin(); // We get `Stdin` here.
        stdin.read_line(&mut sid).unwrap();
        sid = sid.trim().to_string();
        sid = sid.replace(|c: char| c=='"', "");
        let egs = EpicAPI::new();
        egs.auth_sid(sid.as_str()).await;
    }
}