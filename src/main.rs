
use rchain::*;

async fn main() {
    pretty_env_logger::init();

    info!("Peer Id: {}", p2p::PEER_ID.clone());
    let (response_sender, mut response_rcv) = mpsc::unbounded_channel();
    let (init_sender, mut init_rcv) = mpsc::unbounded_channel();

    let auth_keys = Keypair::new()
        .into_authentic(&p2p::KEYS)
        .expect("can create auth keys");

    let transp = TokioTcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseCongi::xx(auth_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    let behaviour = p2p::AppBehaviour::new(App::new(), response_sender, init_sender.clone()).await;

    let mut swarm = SwarmBuilder::new(transp,behaviour, *p2p::PEER_ID)
        .executor(Box::new(|fut|{
            spawn(fut);
        }))
        .build();

    let stdin = BufReader::new(stdin()).lines();

    Swarm::listen_on(
        &mut swarm,
        "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("can get a local socket"),
    )
    .expect("swarm can be started");

    spawn(async move {
            sleep(Duration::from_secs(1)).await;
            info!("sending init event");
            init_sender.send(true).expect("can send init event");
        });
     loop {
         let evt = {
            select! {
                line = stdin.next_line() => SOme(p2p::EventType::Input(line.expect("can get line").expect("can read line from stdin"))),
                response = response_rcv.recv() => {
                    Some(p2p::EventType::LocalChainResponse(response.expect("response exists")))
                },

                _init = init_rcv.recv() => {
                    Some(p2p::EventType::Init)
                }

                event = swarm.select_next_some() => {
                    info!("Unhandled Swarm Event: {:?}", event);
                    None
                },
            }
         };

         if let Some(event) = evt {
             match event {
                 p2p::EventType::Init => {
                     let peers = p2p::get_list_peers(&swarm);
                     swarm.behaviour_mut().app.genesis();

                     info!("connected nodes: {}", peers.len());
                     if !peers.is_empty(){
                         let req = p2p::LocalChainRequest {
                             from_peer_id: peers
                                .iter()
                                .last()
                                .expect("at least one peer")
                                .to_string(),

                         };

                         let json = serde_json::to_string(&req).expect("can jsonify request");
                         swarm 
                            .behaviour_mut()
                            .floodsub
                            .publish(p2p::CHAIN_TOPIC.clone(), json.as_bytes());
                     }
                 }
                 p2p::EventType::LocalChainResponse(resp) => {
                     let json = serde_json::to_string(&resp).expect("can jsonify response");
                     swarm 
                        .behaviour_mut()
                        .floodsub
                        .publish(p2p::CHAIN_TOPIC.clone(), json.as_bytes());
                 }
                 p2p::EventType::Input(line) => match line.as_str(){
                    "ls p" => p2p::handle_print_peers(&swarm),
                    cmd if cmd.starts_with("ls c") => p2p::handle_print_chain(&swarm),
                    cmd if cmd.starts_with("create b") => p2p::handle_create_block(cmd, &mut swarm),
                    _=> error!("unknown command"),
                 },
             };
         }
     }
     pub fn get_list_peers(swarm: &Swarm) -> Vec {
         info!("Discovered Peers:");
         let nodes = swarm.behaviour().mdns.discovered_nodes();
         let mut unique_peers = HashSet::new();
         for peer in nodes {
             unique_peers.insert(peer);
         }
         unique_peers.iter().map(|p| p.to_string()).collect()
     }
     pub fn handle_print_peers(swarm: &Swarm){
         let peers = get_list_peers(swarm);
         peers.iter().for_each(|p| info!("{}", p));
     }
     pub fn handle_print_chain(swarm: &Swarm){
         info!("Local Blockchain:");
         let pretty_json = serde_json::to_string_prettu(&swarm.behaviour().app.blocks).expect("can jsonify blocks");
         info!("{}", pretty_json);
     }
     pub fn handle_create_block(cmd: &str, swarm: &mut Swarm){
        if let Some(data) = cmd.strip_prefix("create b")  {
            let behaviour = swarm.behaviour_mut();
            let latest_block = behaviour
                .app
                .blocks
                .last()
                .expect("there is at least one block");
            let block = Block::new(
                latest_block.id + 1,
                latest_block.hash.clone(),
                data.to_owned(),
            );

            let json =  Serde_json::to_string(&block).expect("can jsonify request");
            behaviour.app.blocks.push(block);
            info!("broadcasting new block");
            behaviour
                .floodsub
                .publish(BLOCK_TOPIC.clone(), json.as_bytes());
        } 
     }

}
