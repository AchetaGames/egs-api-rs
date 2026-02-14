use egs_api::EpicGames;
use std::io;
use std::path::PathBuf;

const TOKEN_FILE: &str = ".egs-api/token.json";

fn token_path() -> PathBuf {
    dirs_or_home().join(TOKEN_FILE)
}

fn dirs_or_home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Save the current session token to disk
pub fn save_token(egs: &EpicGames) {
    let path = token_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let user_data = egs.user_details();
    match serde_json::to_string_pretty(&user_data) {
        Ok(json) => {
            if std::fs::write(&path, &json).is_ok() {
                println!("[auth] Token saved to {}", path.display());
            } else {
                eprintln!("[auth] Failed to write token to {}", path.display());
            }
        }
        Err(e) => eprintln!("[auth] Failed to serialize token: {}", e),
    }
}

/// Try to restore a session from a saved token, refreshing if needed.
/// If no valid token exists, prompt for an authorization code.
/// Returns true if logged in successfully.
pub async fn login_or_restore(egs: &mut EpicGames) -> bool {
    let path = token_path();
    if path.exists() {
        if let Ok(json) = std::fs::read_to_string(&path) {
            if let Ok(user_data) = serde_json::from_str(&json) {
                egs.set_user_details(user_data);
                println!("[auth] Loaded saved token from {}", path.display());

                if egs.login().await {
                    println!("[auth] Session restored successfully");
                    save_token(egs);
                    return true;
                }
                println!("[auth] Saved token expired, need fresh login");
            }
        }
    }

    prompt_auth_code(egs).await
}

/// Open the Epic login page and prompt for the authorization code.
/// Returns true if login succeeded.
pub async fn prompt_auth_code(egs: &mut EpicGames) -> bool {
    let login_url = "https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect%3FclientId%3D34a02cf8f4414e29b15921876da36f9a%26responseType%3Dcode";

    if webbrowser::open(login_url).is_err() {
        println!("Please open this URL in your browser:");
        println!("{}", login_url);
    }

    println!("Enter the 'authorizationCode' value from the JSON response:");
    let mut code = String::new();
    io::stdin().read_line(&mut code).unwrap();
    let code = code.trim().replace('"', "");

    if code.is_empty() {
        eprintln!("[auth] No authorization code provided");
        return false;
    }

    if egs.auth_code(None, Some(code)).await {
        println!("[auth] Logged in successfully");
        save_token(egs);
        true
    } else {
        eprintln!("[auth] Login failed");
        false
    }
}
