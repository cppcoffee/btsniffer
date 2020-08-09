use btsniffer::{MetaWire, Result, DHT};

use async_std::task;
use clap::{App, Arg};
use log::{debug, error, info};

async fn run_server(addr: &str, port: &str, friends: usize) -> Result<()> {
    let mut dht = DHT::new(addr, port, friends);
    let rx = dht.run().await?;

    loop {
        let msg = rx.recv().await?;

        // TODO: limit spawn count.
        task::spawn(async {
            let mut wire = MetaWire::new(msg);
            match wire.fetch().await {
                Ok(meta) => {
                    // TODO: save torrent file.
                    // TODO: output torrent info.
                    println!("{:?}", meta);
                }
                Err(e) => {
                    // TODO: add peer in black list.
                    error!("fetch fail, {}", e);
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
        .get_matches();

    let addr = matches.value_of("addr").unwrap_or("0.0.0.0");
    let port = matches.value_of("port").unwrap_or("6881");
    let friends = matches.value_of("friends").unwrap_or("500");

    let friends = friends.parse().unwrap();

    info!("btsnfifer start.");
    task::block_on(async {
        if let Err(e) = run_server(addr, port, friends).await {
            error!("server failed: {}.", e);
            std::process::exit(1);
        }
    });
    info!("btsnfifer exit.");
}
