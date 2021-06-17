# hyper::http POST requests

*Why do you only get part of the body back without the proper headers?*

This GitHub repo was created to accompany a [post](https://users.rust-lang.org/t/hyper-http-post-only-gets-part-of-the-body-based-on-headers/61233) on the rust-lang users forum.



I wanted to make a quick post having spent a couple hours trying to diagnose odd behavior using the hyper::http client. I hope it will save someone some trouble in the future. But I do have a couple related questions to those with more experience in the tokio/hyper ecosystem.

I understand that the `Response<Body>` from a hyper::http request does not get sent all at once and may need to be buffered if you need the whole thing to deserialize into a struct etc. The below code (which I also placed in [this repo](https://github.com/FabioMencoboni/rust_hyper_post)) works using [this approach described on SO](https://stackoverflow.com/a/59428403/155423):

```

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

```

The GET request worked fine. However, the POST request only returned the first property of the return struct `{id: 101}` until I included the header `  .header("Content-type", "application/json; charset=UTF-8")`. **I do not understand why this would change the amount of data that was sent back?** The header did not seem to be necessary when I make the same request with Postman.

**Output from this code:**

> Task { id: 10, userId: 1, title: "illo est ratione doloremque quia maiores aut", completed: true }
> GOT BYTES: {
> "userId": 1,
> "title": "here are my thoughts",
> "body": "some idea here",
> "id": 101
> }
> SocialMediaPost { userId: 1, title: "here are my thoughts", body: "some idea here", id: 101 }

**Output without the Content-type header**

> Task { id: 10, userId: 1, title: "illo est ratione doloremque quia maiores aut", completed: true }
> GOT BYTES: {
> "id": 101
> }
> thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Error("missing field `userId`", line: 3, column: 1)', src/main.rs:90:51
> note: run with `RUST_BACKTRACE=1` environment variable

As one final question, the code above creates a `Client::new()` with each request. I**s there any advantage to client re-use vs. this approach?**