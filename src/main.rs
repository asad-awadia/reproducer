use crypto::digest::Digest;
use crypto::sha1::Sha1;
use futures_util::*;
use roux::comment::CommentData;
use roux_stream::stream_comments;
use serde_json::{json, Value};
use std::{borrow::Borrow, collections::HashSet, time::Duration};
use tokio_retry::strategy::ExponentialBackoff;

#[tokio::main]
async fn main() {
    println!("starting reddit comment indexer..");
    let subreddits = HashSet::from([
        "Guitar", "AskHistorians", "PS4"
    ]);

    let mut tasks = Vec::with_capacity(subreddits.len());
    for sr in subreddits {
        println!("{}", sr);
        let task = tokio::spawn(async move {
            get_comments_for_sub_reddit(sr).await;
        });
        tasks.push(task);
    }

    for task in tasks {
        let t = task.await;
        if t.is_err() {
            println!("{:?}", t.unwrap_err().to_string());
        }
    }
}

async fn get_comments_for_sub_reddit(subreddit_name: &str) {
    let subreddit = roux::subreddit::Subreddit::new(&subreddit_name);

    let retry_strategy = ExponentialBackoff::from_millis(5).factor(100).take(3);

    let (mut stream, join_handle) = stream_comments(
        &subreddit,
        Duration::from_secs(std::env::var("SLEEP").unwrap_or("10".to_string()).parse::<u64>().unwrap_or(10)),
        retry_strategy,
        Some(Duration::from_secs(30)),
    );

    while let Some(comment) = stream.next().await {
        // `comment` is an `Err` if getting the latest comments
        // from Reddit failed even after retrying.
        let c = comment.unwrap();

        println!("got comment {:?}", c);

        // if comment.is_err() {
        //     println!(
        //         "got comment error {} - sleeping for 2 seconds sr-name {}",
        //         comment.err().unwrap().to_string(),
        //         subreddit_name
        //     );
        //     tokio::time::sleep(Duration::from_secs(5)).await;
        //     continue;
        // }

    }

    // In case there was an error sending the submissions through the
    // stream, `join_handle` will report it.
    println!("joining");
    join_handle.await.unwrap().unwrap();
}
