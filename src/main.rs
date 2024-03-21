fn main() {
    println!("Hello, world!");
}

/// Recipe 1:
/// Using clap as a config tool and/or cli interface
/// Requires `cargo add clap -F derive -F env`
#[cfg(never)]
mod clap_example {
    use std::net::SocketAddr;

    use clap::Parser;

    /// Put a short text here describing the program. It will be shown when running `cargo run -- --help`
    #[derive(Debug, Parser)]
    pub struct Config {
        /// Address the server should bind to
        #[clap(env, default_value = "0.0.0.0:8080")]
        pub bind_addr: SocketAddr,
    }

    /// To use this config globally you can store it in a once_cell (`cargo add once_cell`)
    use once_cell::sync::Lazy;
    pub static CONFIG: Lazy<Config> = Lazy::new(Config::parse);
}

/// Recipe 2:
/// Http server using axum
/// This example requires:
/// `cargo add axum`
/// `cargo add tokio -F macros -F rt-multi-thread -F signal -F net`
/// `cargo add serde -F derive`
#[cfg(never)]
mod axum_example {
    use axum::{extract::Path, http::StatusCode, routing::get, Json, Router};
    use serde::{Deserialize, Serialize};
    use tokio::net::TcpListener;

    // This macro allows out main function to be async
    #[tokio::main]
    pub async fn main() {
        let app = Router::new()
            .route("/", get(return_json).post(decode_json))
            .route("/hello/:name", get(greet));
        // Fast shutdown on crl_c
        let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
        axum::serve(listener, app)
            .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
            .await
            .unwrap();
    }

    async fn greet(Path(path): Path<String>) -> String {
        format!("Hello {path}")
    }

    /// This uses serde for serializing and deserializing this struct more info on serde.rs
    #[derive(Debug, Serialize, Deserialize)]
    struct MyJson {
        foo: String,
        bar: Vec<u32>,
    }

    async fn return_json() -> Json<MyJson> {
        Json(MyJson {
            foo: "foo".to_string(),
            bar: vec![98, 97, 114],
        })
    }

    /// A handler can take any number of arguments that implement [`FromRequestParts`](https://docs.rs/axum/latest/axum/extract/trait.FromRequestParts.html)
    /// followed by zero or one argument that implements [`FromRequest`](https://docs.rs/axum/latest/axum/extract/trait.FromRequest.html)
    /// The return type has to implement [`IntoResponse`](https://docs.rs/axum/latest/axum/response/trait.IntoResponse.html)
    async fn decode_json(Json(my_json): Json<MyJson>) -> axum::response::Result<String> {
        let decoded = my_json
            .bar
            .into_iter()
            // We map each u32 to a char which may fail
            .map(char::try_from) // That is why we now have an iterator over results
            // We can use collect to say it should collect to the first error if there is any otherwise we should collect the chars into a string
            // ::<> Syntax tells Rust what type to collect to as it can't be inferred in this case
            .collect::<Result<String, _>>() // The _ in this case tells the compiler to try to infer the Error type of the Result
            // We then map whatever error we got to a StatusCode which implements IntoResponse
            .map_err(|_| StatusCode::BAD_REQUEST)?; // We then use ? here to return the error if we got one
            // We can do this although we don't return a Result<String, StatusCode> because the ? operator implicitly performs a conversion using the From trait
            // And because out Error Type implements IntoResponse it can be converted

        Ok(format!("{} {}", my_json.foo, decoded))
    }
}

/// Recipe 3:
/// Logging using tracing
/// Requires `cargo add tracing`
/// And `cargo add tracing-subscriber -F env-filter`
#[cfg(never)]
mod tracing_example {
    use tracing::{debug, error, info, warn};
    use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

    pub fn main() {
        // Thats all you need to have basic logging the log level is configurable via the RUST_LOG env var
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .finish()
            .init();

        log_wherever_you_want();
    }

    fn log_wherever_you_want() {
        let x = 3;
        // There macros support all kinds of fancy formatting but the simplest way is to use them like the format! macro
        debug!("Debug {}", x);
        // This will give the log message context which will be printed before the message
        info!(x, "Info");
        // This crate is capable of a *lot* more than these 4 macros but 90% of the time this is all you need
        warn!("Warn");
        error!("Error")
    }
}

/// Recipe 4:
/// Http client
/// Requires `cargo add reqwest -F json`
/// Requires `cargo add serde -F derive`
#[cfg(never)]
mod reqwest_example {
    use reqwest::Client;
    use serde::{Deserialize, Serialize};

    /// This uses serde for serializing and deserializing this struct more info on serde.rs
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct SerdeTestStruct {
        foo: String,
        bar: Vec<u32>,
    }

    #[derive(Debug, Deserialize)]
    struct ResponseJson {
        json: SerdeTestStruct,
    }

    /// To await this function you will need to use tokio at some point like in the axum example as
    /// async functions can only be awaited in other async functions
    pub async fn client_example() {
        // This client should not be created every time this function is called
        // When using this client either clone an existing client or put the client in a
        // once_cell::sync::Lazy like in the clap example
        let client = Client::new();
        let data = SerdeTestStruct {
            foo: "Foo".into(),
            bar: vec![2, 3, 4],
        };
        let response_json = client
            .get("https://httpbin.org/anything")
            .json(&data) // You may add more request parameters with the builder pattern
            .send()
            .await // Send the request and wait for the response
            .unwrap() // Here we unwrap the first error that can happen. This error ususally happens when the url cant be reached or resolved to an ip.
            .json::<ResponseJson>() // Now we get the response as some untyped
            .await
            .unwrap();
        assert_eq!(response_json.json, data);
    }
}
