use pakrelly::{
    buffer::{BufferPool, BufferPoolManager},
    disk::DiskManager,
};

fn main() -> std::io::Result<()> {
    let disk_manager = DiskManager::open("hogehoge")?;
    let buffer_pool = BufferPool::new(5);

    let buffer_pool_manger = BufferPoolManager::new(disk_manager, buffer_pool);

    println!("{:?}", buffer_pool_manger);
    Ok(())
}
