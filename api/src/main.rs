use warp::Filter;

#[derive(Debug)]
struct InvalidParameter;

impl warp::reject::Reject for InvalidParameter {}

/// Imagine you can handle rejection
async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    if err.is_not_found() {
        Ok(warp::reply::with_status(
            "NOT_FOUND",
            warp::http::StatusCode::NOT_FOUND,
        ))
    } else if let Some(e) = err.find::<InvalidParameter>() {
        Ok(warp::reply::with_status(
            "BAD_REQUEST",
            warp::http::StatusCode::BAD_REQUEST,
        ))
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        Ok(warp::reply::with_status(
            "INTERNAL_SERVER_ERROR",
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

async fn hello(name: String) -> Result<String, warp::Rejection> {
    Ok(format!("Hello {}", name))
}

#[tokio::main]
async fn main() {
    let hello = warp::get()
        .and(warp::path("hello"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(hello)
        .recover(handle_rejection);

    warp::serve(hello).run(([127, 0, 0, 1], 3000)).await;
}
