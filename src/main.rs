use reqwest;
use std::borrow::BorrowMut;
use std::error::Error;

use tokio::sync::mpsc::Receiver;

use tokio::{
    self, select,
    sync::{mpsc, oneshot},
};

use stable_eyre::eyre::{eyre, Report, Result};

async fn crawl(url: String) -> Result<Vec<String>, Report> {
    let body = reqwest::get(url).await.expect("failed to fetch url");
    // Function: response.text => Vec<URL>
    // +1 on the number of tasks we've ran
    // how do you handle errors without await-ing
    Err(eyre!("this failed"))
}

async fn crawler_dispatch(mut work_rx: Receiver<String>, mut quit: oneshot::Receiver<()>) {
    loop {
        select! {
            Some(url) = work_rx.recv() => {
                println!("doing some work on {url}");
                tokio::spawn(crawl(url));
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

    let dispatch_handler = tokio::spawn(crawler_dispatch(work_rx, quit_rx));
    let sending = work_tx.send(target.to_string());
    tokio::join!(dispatch_handler, sending);
}
