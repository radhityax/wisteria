use ksh::block::Block;

fn main() {
    let block = Block::new(&b"hello world"[..]).unwrap();
    println!("CID: {}", block.cid());
    println!("CID (base58btc): {}",
        block.cid().to_string_of_base(multibase::Base::Base58Btc).unwrap());
    println!("Size: {}", block.size());
    println!("Verify: {}", block.verify().unwrap());
}
