#![feature(await_macro, async_await, futures_api, uniform_paths)]
#![allow(non_snake_case, non_upper_case_globals, unknown_lints)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::Into;
use std::fmt::Debug;
use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::io::{BufRead, BufReader, BufWriter};
use std::net::TcpStream;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::{ChildStdin, ChildStdout, Stdio};
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

extern crate tokio;
use futures::sync::{mpsc as fmpsc, oneshot};
use futures::Stream;
use tokio::await;
use tokio::prelude::StreamExt;

#[macro_use]
extern crate log;
extern crate log4rs;

extern crate failure;
use failure::{bail, format_err, Fail};
#[allow(unused_imports)]
use failure::{err_msg, Error, ResultExt};

extern crate maplit;
use maplit::hashmap;

extern crate serde;
use serde::de::DeserializeOwned;
use serde::Serialize;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use serde_json::json;

extern crate jsonrpc_core as rpc;
use crate::rpc::{Params, Value};

extern crate languageserver_types as lsp;
use crate::lsp::*;

extern crate url;
use url::Url;

extern crate pathdiff;
use pathdiff::diff_paths;

extern crate diff;

extern crate glob;
extern crate regex;

extern crate notify;
#[allow(unused_imports)]
use notify::Watcher;

extern crate structopt;
use structopt::StructOpt;

extern crate shellexpand;

mod types;
use crate::types::*;
mod utils;
use crate::utils::*;
mod languageclient;
use languageclient::{LanguageClient};
mod logger;
mod rpchandler;
mod vim;

mod rpcclient;

#[derive(Debug, StructOpt)]
struct Arguments {}

fn main() -> Fallible<()> {
    let version = format!("{} {}", env!("CARGO_PKG_VERSION"), env!("GIT_HASH"));
    let args = Arguments::clap().version(version.as_str());
    let _ = args.get_matches();

    let (tx, mut rx): (fmpsc::UnboundedSender<Call>, fmpsc::UnboundedReceiver<Call>) =
        fmpsc::unbounded();
    let languageclient = LanguageClient::new(State::new(tx)?);

    tokio::run_async(
        async move {
            info!("Starting main loop");

            while let Some(request) = await!(tokio::prelude::StreamAsyncExt::next(&mut rx)) {
                if let Ok(request) = request {
                    info!("{:?}", request);
                    languageclient.clone().handle_request(request);
                }
            }
        },
    );

    Ok(())
}
