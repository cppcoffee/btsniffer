use btsniffer::{torrent, BlackList, Error, MetaWire, DHT};

use anyhow::Result;
use async_std::path::{Path, PathBuf};
use async_std::{fs, task};
use bencode::Value;
use log::{debug, error, info};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(
        short = "a",
        long = "addr",
        help = "listen on given address (default all, ipv4 and ipv6)",
        default_value = "0.0.0.0"
    )]
    addr: String,
    #[structopt(
        short = "p",
        long = "port",
        help = "listen on given port",
        default_value = "6881"
    )]
    port: String,
    #[structopt(
        short = "f",
        long = "friends",
        help = "max fiends to make with per second",
        default_value = "500"
    )]
    friends: usize,
    #[structopt(
        short = "t",
        long = "timeout",
        help = "max time allowed for downloading torrents",
        default_value = "15"
    )]
    timeout: u64,
    #[structopt(
        short = "e",
        long = "peers",
        help = "max peers to connect to download torrents",
        default_value = "500"
    )]
    peers: usize,
    #[structopt(
        short = "b",
        long = "blacklist",
        help = "max blacklist size for downloading torrents",
        default_value = "5000"
    )]
    blsize: usize,
    #[structopt(
        short = "d",
        long = "dir",
        help = "the directory to store the torrents",
        default_value = "./torrents/"
    )]
    dir: PathBuf,
}

async fn run_server(opt: Opt) -> Result<()> {
    let blacklist = BlackList::new(opt.blsize);
    let mut dht = DHT::new(&opt.addr, &opt.port, opt.friends, opt.peers);
    let rx = dht.run().await?;

    loop {
        let msg = rx.recv().await?;

        if blacklist.contains(&msg.peer) {
            debug!("peer {} in the blacklist, skip.", msg.peer);
            continue;
        }

        let path = join_torrent_path(&opt.dir, msg.infohash_hex()).await;
        if path.exists().await {
            debug!("torrent {:?} exist, skip.", path);
            continue;
        }

        let timeout = opt.timeout;
        let infohash_hex = msg.infohash_hex();
        let mut blist_clone = blacklist.clone();

        task::spawn(async move {
            let mut wire = MetaWire::new(&msg, timeout);
            match wire.fetch().await {
                Ok(meta) => {
                    let _ = store_torrent(&path, &meta)
                        .await
                        .map_err(|e| debug!("store_torrent failed, {}", e));

                    match torrent::from_bytes(infohash_hex, &meta) {
                        Ok(t) => println!("{}", serde_json::to_string(&t).unwrap()),
                        Err(e) => debug!("parse torrent failed, {}", e),
                    }
                }
                Err(e) => {
                    debug!("fetch fail, {}, {} add black list.", e, msg.peer);
                    blist_clone.insert(msg.peer);
                }
            }
        });
    }
}

async fn join_torrent_path(path: &PathBuf, infohash_hex: String) -> PathBuf {
    path.join(&infohash_hex[..2])
        .join(&infohash_hex[2..4])
        .join(infohash_hex + ".torrent")
}

async fn store_torrent(path: &Path, meta: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or(Error::Other(format!("path({:?}).parent fail", path)))?;

    fs::create_dir_all(parent).await?;

    let m = bencode::from_bytes(meta)?;
    let d = Value::from(bencode::map!(b"info".to_vec() => m));
    let data = bencode::to_bytes(&d)?;

    Ok(fs::write(path, data).await?)
}

fn main() {
    env_logger::init();
    let opt = Opt::from_args();

    info!("btsnfifer start.");
    task::block_on(async {
        if let Err(e) = run_server(opt).await {
            error!("server failed: {}.", e);
            std::process::exit(1);
        }
    });
    info!("btsnfifer exit.");
}
