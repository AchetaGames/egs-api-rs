use hyper::{body::HttpBody, client::HttpConnector, Body, Client, Request, Response, Error};
use hyper::header::{SET_COOKIE};
use hyper_tls::HttpsConnector;

type WebClient = Client<HttpsConnector<HttpConnector>, Body>;

pub struct EpicAPI {
    client: WebClient,
}

impl EpicAPI {
    pub fn new() -> Self {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        EpicAPI {
            client
        }
    }

    pub async fn auth_sid(&self, sid: &str) {
        // get first set of cookies (EPIC_BEARER_TOKEN etc.)
        let url = format!("https://www.epicgames.com/id/api/set-sid?sid={}", sid);
        let mut builder = Request::builder()
            .method("GET")
            .uri(url)
            .header("X-Epic-Event-Action", "login")
            .header("X-Epic-Event-Category", "login")
            .header("X-Epic-Strategy-Flags", "")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("User-Agent", "EpicGamesLauncher/11.0.1-14907503+++Portal+Release-Live ");
        let req = builder.body(Body::empty()).expect("request builder");
        match self.client.request(req).await {
            Ok(mut resp) => {
                println!("{:?}", resp);
                let cookies = resp
                    .headers()
                    .get_all(SET_COOKIE);
                for cookie in cookies.iter() {
                    println!("Got cookie: {:?}", cookie);
                }
            }
            _ => {}
        }
    }
}