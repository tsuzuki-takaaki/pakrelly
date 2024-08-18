use anyhow::Result;

use pakrelly::btree::BTree;
use pakrelly::buffer::{BufferPool, BufferPoolManager};
use pakrelly::disk::DiskManager;

fn main() -> Result<()> {
    let disk_manager = DiskManager::open("fugafuga")?;
    let buffer_pool = BufferPool::new(10);
    let mut buffer_pool_manager = BufferPoolManager::new(disk_manager, buffer_pool);

    let btree = BTree::create(&mut buffer_pool_manager)?;

    btree.insert(&mut buffer_pool_manager, b"Kanagawa", b"Yokohama")?;
    btree.insert(&mut buffer_pool_manager, b"Osaka", b"Osaka")?;
    btree.insert(&mut buffer_pool_manager, b"Aichi", b"Nagoya")?;
    btree.insert(&mut buffer_pool_manager, b"Hokkaido", b"Sapporo")?;
    btree.insert(&mut buffer_pool_manager, b"Fukuoka", b"Fukuoka")?;
    btree.insert(&mut buffer_pool_manager, b"Hyogo", b"Kobe")?;

    buffer_pool_manager.flush()?;

    Ok(())
}
