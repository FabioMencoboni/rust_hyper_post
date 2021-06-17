
use std::{env, fmt};
use serde::{self, Serialize, Deserialize, de::DeserializeOwned};
use serde_json;
use hyper::body; // brings the to_bytes() method into scope:
use hyper::{Request, Body, Method, Client};


// Make a GenericError and Generic Result that Box a bunch of statically-typed stuff
pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type GenericResult<T> = std::result::Result<T, GenericError>;


#[derive(Deserialize, Debug)]
struct Task {
    id: i32, 
    userId: i32,
    title: String,
    completed: bool,
}


#[derive(Serialize)]
struct Payload {
    userId: i32,
    title: String,
    body: String,
}

#[derive(Deserialize, Debug)]
struct SocialMediaPost {
    userId: i32,
    title: String,
    body: String,
    id: i32,
}


#[tokio::main]
async fn main() -> GenericResult<()> {

    // GET seems to buffer the whole body as expected
    let task: Task = get("http://jsonplaceholder.typicode.com/todos/10").await.unwrap();
    println!("{:?}", task); // WORKS: prints Task { id: 10, userId: 1, title: "illo est ratione doloremque quia maiores aut", completed: true }

    
    // But why does POST have to have a header to get the whole thing back?
    let payload = Payload {
        userId: 1i32,
        title: "here are my thoughts".to_string(),
        body: "some idea here".to_string(),
    };
    let smp: SocialMediaPost = post("http://jsonplaceholder.typicode.com/posts", &payload).await.unwrap();
    println!("{:?}", smp);

    Ok(())
}


pub async fn get<T: DeserializeOwned>(url: &str) -> GenericResult<T> {
    let request = Request::builder()
        .method(Method::GET)
        .uri(url)
        .header("accept", "application/json")
        .body(Body::empty()).unwrap();
    let client = Client::new();
    let resp = client.request(request).await.unwrap();
    let bytes = body::to_bytes(resp.into_body()).await.unwrap();
    let foo = serde_json::from_slice::<T>(&bytes).unwrap();
    Ok(foo)
}



async fn post<U: Serialize, T: DeserializeOwned>(url: &str, payload: &U) -> GenericResult<T> {
    let body_string = serde_json::to_string(payload).unwrap();
    let request = Request::builder()
        .method(Method::POST)
        .uri(url)
        .header("accept", "application/json")
        // IF YOU DON'T INCLUDE THIS HEADER, ONLY THE FIRST PROPERTY OF THE STRUCT GETS RETURNED???
        .header("Content-type", "application/json; charset=UTF-8")
        .body(Body::from(body_string)).unwrap();
    let client = Client::new();
    let resp = client.request(request).await.unwrap();
    let bytes = body::to_bytes(resp.into_body()).await.unwrap();
    println!("GOT BYTES: {}", std::str::from_utf8(&bytes).unwrap() );
    let foo = serde_json::from_slice::<T>(&bytes).unwrap();
    Ok(foo)
}
