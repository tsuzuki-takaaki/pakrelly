use pakrelly::disk::DiskManager;

fn main() -> std::io::Result<()> {
    let mut disk_manager = DiskManager::open("helloworld")?;
    println!("{:?}", disk_manager);
    let page_id = disk_manager.allocate_page();
    println!("{:?}", page_id);
    println!("{:?}", disk_manager);
    Ok(())
}
