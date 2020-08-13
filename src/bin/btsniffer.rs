use btsniffer::{BlackList, MetaWire, Result, DHT};

use async_std::task;
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
}

async fn run_server(opt: Opt) -> Result<()> {
    let blacklist = BlackList::new(opt.blsize);
    let mut dht = DHT::new(&opt.addr, &opt.port, opt.friends, opt.peers);
    let rx = dht.run().await?;

    loop {
        let msg = rx.recv().await?;

        if blacklist.contains(&msg.peer) {
            continue;
        }

        let timeout = opt.timeout;
        let mut blist_clone = blacklist.clone();
        task::spawn(async move {
            let mut wire = MetaWire::new(&msg, timeout);
            match wire.fetch().await {
                Ok(meta) => {
                    // TODO: save torrent file.
                    // TODO: output torrent info.
                    println!("{:?}", meta);
                }
                Err(e) => {
                    debug!("fetch fail, {}, {} add black list.", e, msg.peer);
                    blist_clone.insert(msg.peer);
                }
            }
        });
    }
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
