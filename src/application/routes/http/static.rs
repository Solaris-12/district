use rocket::response::content::RawHtml;

#[get("/")]
pub fn static_index_page() -> RawHtml<String> {
  RawHtml("<title>DISTRICT Server</title><style>html{height:100dvh;display:grid;place-items:center;}body{border:2px solid gray;padding:2rem;text-align:center;}
    </style><body><h1>DISTRICT Server</h1><p>You have found a DISTRICT server!</p><p>Don't worry, there is nothing to see here!</p></body>".to_string())
}

#[get("/test")]
pub(crate) async fn static_test() -> String {
  String::from("YES!")
}
