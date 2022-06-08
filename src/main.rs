use reqwest;
use scraper::{Html, Selector};
use std::borrow::BorrowMut;
use std::error::Error;
use urlparse::Url;

use tokio::sync::mpsc::{Receiver, Sender};

use tokio::{
    self, select,
    sync::{mpsc, oneshot},
};

use stable_eyre::eyre::{eyre, Report, Result};


async fn crawl(url: (i32, String), work_tx: Sender<String>) -> Result<Vec<String>, Report> {
    let mut urls = Vec::new();
    {
        let body = reqwest::get(url).await?;
        let text = body.text().await.expect("Failed to get text of body");
        let ast = Html::parse_document(&text);
        let selector = Selector::parse("a").unwrap();
        for elem in ast.select(&selector) {
            if let Some(url) = elem.value().attr("href") {
                let url = Url::parse(url);
                let full_path = format!("{}://{}{}", url.scheme, url.netloc, url.path);
                urls.push(full_path);
            }
        }
    }
    for url in urls {
        work_tx.send(url).await;
    }
    // Function: response.text => Vec<URL>
    // +1 on the number of tasks we've ran
    // how do you handle errors without await-ing
    Err(eyre!("this failed"))
}

async fn crawler_dispatch(
    mut work_rx: Receiver<String>,
    work_tx: Sender<String>,
    mut quit: oneshot::Receiver<()>,
) {
    loop {
        select! {
            Some(url) = work_rx.recv() => {
                println!("doing some work on {url}");
                tokio::spawn(crawl(url, work_tx.clone()));
            },

            // why does this work since poll(mut self)
            // so why does borrow mut helps?
            stuff = quit.borrow_mut() => {
                println!("quit signal received, winding down dispatcher!");
                return
            }
        }
    }
}

#[tokio::main]
async fn main() {
    stable_eyre::install().expect("could not initialize stable_eyre");
    // suppose we get some CLI argument
    //          - website
    //          - # of tasks (links)

    let target = "https://erwanor.github.io/";
    let limit_tasks = 100;

    let (work_tx, work_rx) = mpsc::channel::<String>(limit_tasks);

    let (quit_tx, quit_rx) = oneshot::channel::<()>();

    let dispatch_handler = tokio::spawn(crawler_dispatch(work_rx, work_tx.clone(), quit_rx));
    let sending = work_tx.send(target.to_string());
    tokio::join!(dispatch_handler, sending);
}
