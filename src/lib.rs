mod block;
pub mod blockchain;
pub use blockchain::Blockchain;
use std::time::Instant;

const DIFFICULTY_PREFIX: &str = "00";
const MINING_REWARD: f32 = 100f32;

//calculate hash for blocks

fn calculate_hash(id: u64, timestamp: i64, previous_hash: &str, data: &str, nonce: u64) -> Vec<u8> {
    let data  = serde_json::json!({
        "id" : id,
        "previous_hash": previous_hash,
        "data": data,
        "timestamp": timestamp,
        "nonce": nonce
    });

    let mut hasher = Sha256::new();
    hasher.update(data.to_string().as_bytes());
    hasher.finalize().as_slice().to_owned()
}

pub struct App{
    pub blocks: Vec,
}

fn hash_to_binary_representation(hash: &[u8]) -> String {
    let mut res: String = String::default();
    for c in hash{
        res.push_str(&format!("{:b}", c));
    }
    res
}

impl App{
    fn new() -> Self {
        Self {blocks:vec![]}
    }

    fn genesis(&mut self){
        let genesis_block = Block {
            id: 0,
            timestamp: Utc::now().timestamp(),
            previous_hash: String::from("genesis"),
            data: String::from("genesis!"),
            nonce:2836,
            hash:"0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43".to_string(),

        };
        self.blocks.push(genesis_block);
    }

    fn try_add_block(&mut self, block: Block) {
        let latest_block = self.blocks.last().expect("there is atleast one block produced");
        if self.is_block_valid(&block, latest_block){
            self.blocks.push(block);
        } else {
            error!("could not add block - invalid");
        }
    }

    fn is_block_valid(&self, block: &Block, previous_block: &Block) -> bool{
        if block.previous_hash != previous_block.hash {
            warn!("block with id: {} has wrong previous hash", block.id);
            return false;
        } else if block.id != previous_block.id + 1{
            warn!(
                "block with id: {} is not the next block after the latest: {}",
                block.id, previous_block.id
            );
            return false;
        } else if hex::encode(calculate_hash(
            block.id,
            block.timestamp,
            &block.previous_hash,
            &block.data,
            block.nonce,

        )) != block.hash
        
        {
            warn!("block with id: {} has invalid hash", block.id);
            return false;
        }

        true
    }

    fn is_chain_valid(&self, chain: &[Block]) -> bool{
        for i in 0..chain.len(){
            if i == 0 {
                continue;
            }
            let first = chain.get(i-1).expect("has to exist");
            let second = chain.get(i).expect("has to exist");
            if !self.is_block_valid(second, first){
                return false;
            }
        }
        true
    }

    //longest chain rule(We choose the longest valid chain)

    fn choose_chain(&mut self, local: Vec, remote: Vec) -> Vec {
        let is_local_valid = self.is_chain_valid(&local);
        let is_remote_valid = self.is_chain_valid(&remote);

        if is_local_valid && is_remote_valid{
            if local.len() >= remote.len() {
                local
            } else {
                remote
            }
        } else if is_remote_valid && !is_local_valid{
            remote
        } else if !is_remote_valid && is_local_valid {
            local
        } else {
            panic!("local and remote chains are both invalid");
        }
    }
}
