use super::*;
use crate::block::Block;

#[derive(Debug)]
pub struct Blockchain{
    pub blocks: Vec<Block>,
    unmined_transactions: Vec<Transaction>,
    mining_reward: f32,
}

impl Blockchain{

    pub fn new()-> Self {
        Blockchain {blocks: vec![]}
    }

    pub fn mine_unmined_transactions(&mut self, miner_address: String) {
        let transactions = &self.unmined_transactions;
        let mut block = Block::new(transactions.to_vec());
        match self.blocks.last(){
            Some(pre_block) => block.set_previous_hash(pre_block.block_hash.to_owned()),
            None => block.set_previous_hash("0".to_string()), //genesis block
        }
        block.set_hash();
        block.mine();
        self.blocks.push(block);
        self.unmined_transactions = vec![Transaction{
            sender: String::new(),
            receiver: miner_address,
            amount: self.mining_reward,
        }];
    }

    pub fn is_valid_chain(&self) -> bool {
        let blocks = &self.blocks;

        for(i, block) in blocks.iter().enumerate(){
            if block.block_hash
                != calculate_hash(
                    &block.previous_hash,
                    &block.transaction,
                    &block.timestamp,
                    &block.nonce,
                )
            {
                return false;
            }
            if i > 0 && block.previous_hash != blocks[i - 1].block_hash{
                return false;
            }
        }

        return true;
    }

    pub fn mine(&mut self) {
        let target = get_difficult_string();

        while &self.block_hash[..DIFFICULT_LEVEL as usize] != target {

            self.nonce += 1;
            self.block_hash = calculate_hash(
                &self.previous_hash,
                &self.transaction,
                &self.timestamp,
                &self.nonce
            )
        }
        println!("Block Mined");
    }


}