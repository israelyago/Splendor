use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use iroh::{protocol::Router, Endpoint, NodeAddr};
use iroh_gossip::{
    net::{Gossip, GossipReceiver, GossipSender},
    proto::TopicId,
};
use splendor::{
    network_message::Message,
    network_subscribe::{subscribe_client_loop, subscribe_server_loop},
    ticket::Ticket,
};

/// Splendor Server
///
/// Start or join a Splendor server peer-to-peer
#[derive(Parser, Debug)]
struct Args {
    /// Set the bind port for our socket. By default, a random port will be used.
    #[clap(short, long, default_value = "0")]
    bind_port: u16,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// Creates a new game session and print a ticket for others to join.
    Create,
    /// Join a server from a ticket.
    Join {
        /// The ticket, as base32 string.
        ticket: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let p2p_network = get_network_parameters(&args.command).await?;

    // print a ticket that includes our own node id and endpoint addresses
    println!("> Ticket to join this server: {}", p2p_network.ticket);

    // join the gossip topic by connecting to known nodes, if any
    let (sender, receiver) = join_p2p_network(&p2p_network).await?;
    println!("> connected!");

    // subscribe and print loop
    match &args.command {
        Command::Create => tokio::spawn(subscribe_server_loop(
            receiver,
            sender.clone(),
            p2p_network.endpoint.node_id(),
        )),
        Command::Join { ticket: _ } => tokio::spawn(subscribe_client_loop(receiver)),
    };

    let _ = listen_for_local_input(sender).await;

    p2p_network.router.shutdown().await?;

    Ok(())
}

async fn join_p2p_network(
    p2p_network: &InitialNetworkConnection,
) -> Result<(GossipSender, GossipReceiver)> {
    let node_ids = p2p_network.nodes.iter().map(|p| p.node_id).collect();
    if p2p_network.nodes.is_empty() {
        println!("> waiting for players to join us...");
    } else {
        println!("> trying to connect to peers...");
        // add the peer addrs from the ticket to our endpoint's addressbook so that they can be dialed
        for node in p2p_network.nodes.clone().into_iter() {
            p2p_network.endpoint.add_node_addr(node)?;
        }
    };
    let (sender, receiver) = p2p_network
        .gossip
        .subscribe_and_join(p2p_network.topic, node_ids)
        .await?
        .split();

    Ok((sender, receiver))
}

struct InitialNetworkConnection {
    topic: TopicId,
    nodes: Vec<NodeAddr>,
    endpoint: Endpoint,
    gossip: Gossip,
    router: Router,
    ticket: Ticket,
}

async fn get_network_parameters(command: &Command) -> Result<InitialNetworkConnection> {
    let (topic, nodes) = match command {
        Command::Create => {
            let topic = TopicId::from_bytes(rand::random());
            println!("> Starting a new server ({topic})");
            (topic, vec![])
        }
        Command::Join { ticket } => {
            let ticket = Ticket::from_str(ticket)?;
            let topic = ticket.topic();
            let nodes = Vec::from(ticket.nodes());
            println!("> Connecting to the server {topic}");
            (topic, nodes)
        }
    };

    let endpoint = Endpoint::builder().discovery_n0().bind().await?;

    println!("> Our node id: {}", endpoint.node_id());
    let mib = 1024usize.pow(2);
    let gossip = Gossip::builder()
        .max_message_size(mib)
        .spawn(endpoint.clone())
        .await?;

    let router = Router::builder(endpoint.clone())
        .accept(iroh_gossip::ALPN, gossip.clone())
        .spawn()
        .await?;

    let ticket = {
        let me = endpoint.node_addr().await?;
        let nodes = vec![me];
        Ticket::new(topic, nodes)
    };

    Ok(InitialNetworkConnection {
        endpoint,
        gossip,
        nodes,
        router,
        topic,
        ticket,
    })
}

async fn listen_for_local_input(sender: GossipSender) -> Result<()> {
    let (line_tx, mut line_rx) = tokio::sync::mpsc::channel(1);
    std::thread::spawn(move || input_loop(line_tx));

    while let Some(user_input) = line_rx.recv().await {
        match serde_json::from_str::<Message>(&user_input) {
            Ok(message) => {
                sender.broadcast(message.to_vec().into()).await?;
            }
            Err(decoding_error) => {
                println!("Malformatted json: {}", decoding_error);
            }
        }
    }

    Ok(())
}

fn input_loop(line_tx: tokio::sync::mpsc::Sender<String>) -> Result<()> {
    let mut buffer = String::new();
    let stdin = std::io::stdin(); // We get `Stdin` here.
    loop {
        stdin.read_line(&mut buffer)?;
        line_tx.blocking_send(buffer.clone())?;
        buffer.clear();
    }
}
