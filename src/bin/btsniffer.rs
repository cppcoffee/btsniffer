use btsniffer::{BlackList, MetaWire, Result, DHT};

use async_std::task;
use clap::{App, Arg};
use log::{debug, error, info};

async fn run_server(
    addr: &str,
    port: &str,
    friends: usize,
    peers: usize,
    timeout: u64,
    blsize: usize,
) -> Result<()> {
    let blacklist = BlackList::new(blsize);
    let mut dht = DHT::new(addr, port, friends, peers);
    let rx = dht.run().await?;

    loop {
        let msg = rx.recv().await?;

        if blacklist.contains(&msg.peer) {
            continue;
        }

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

    let matches = App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("addr")
                .short("a")
                .long("addr")
                .help("listen on given address (default all, ipv4 and ipv6)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("listen on given port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("friends")
                .short("f")
                .long("friends")
                .help("max fiends to make with per second")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .help("max time allowed for downloading torrents")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("peers")
                .short("e")
                .long("peers")
                .help("max peers to connect to download torrents")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("blacklist")
                .short("b")
                .long("blacklist")
                .help("max blacklist size for downloading torrents")
                .takes_value(true),
        )
        .get_matches();

    // parse args.
    let addr = matches.value_of("addr").unwrap_or("0.0.0.0");
    let port = matches.value_of("port").unwrap_or("6881");
    let friends = matches.value_of("friends").unwrap_or("500");
    let timeout = matches.value_of("timeout").unwrap_or("15");
    let peers = matches.value_of("peers").unwrap_or("400");
    let blacklist = matches.value_of("blacklist").unwrap_or("50000");

    let friends = friends.parse().unwrap();
    let timeout = timeout.parse().unwrap();
    let peers = peers.parse().unwrap();
    let blsize = blacklist.parse().unwrap();

    info!("btsnfifer start.");
    task::block_on(async {
        if let Err(e) = run_server(addr, port, friends, peers, timeout, blsize).await {
            error!("server failed: {}.", e);
            std::process::exit(1);
        }
    });
    info!("btsnfifer exit.");
}
