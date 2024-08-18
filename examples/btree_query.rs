use anyhow::{Ok, Result};

use pakrelly::btree::{BTree, SearchMode};
use pakrelly::buffer::{BufferPool, BufferPoolManager};
use pakrelly::disk::{DiskManager, PageId};

fn main() -> Result<()> {
    let disk_manager = DiskManager::open("fugafuga")?;
    let buffer_pool = BufferPool::new(10);
    let mut buffer_pool_manager = BufferPoolManager::new(disk_manager, buffer_pool);

    let btree = BTree::new(PageId(0));
    let mut iter = btree.search(&mut buffer_pool_manager, SearchMode::Key(b"Hyogo".to_vec()))?;
    let (key, value) = iter.next(&mut buffer_pool_manager)?.unwrap();
    println!("{:02x?} = {:02x?}", key, value);
    Ok(())
}
