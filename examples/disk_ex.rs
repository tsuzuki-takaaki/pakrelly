use pakrelly::disk::{DiskManager, PAGE_SIZE};

fn main() -> std::io::Result<()> {
    let mut disk_manager = DiskManager::open("helloworld")?;
    println!("{:?}", disk_manager);
    let page_id_bef = disk_manager.allocate_page();
    let page_id_aft = disk_manager.allocate_page();

    // 2回allocate_pageしているため、4097bytes目から書き込まれる(切り上げられたspaceはNULL)
    disk_manager.write_page_data(page_id_aft, b"Hello world")?;

    let mut buf = vec![0; PAGE_SIZE];
    // [CAUTION] Error: Error { kind: UnexpectedEof, message: "failed to fill whole buffer" }
    // -> 4096bytes読み込もうとしても、fileにそれだけのデータが存在しないと失敗する
    // 書き込む際に、[0; PAGE_SIZE]にリサイズしてwriteすればいい
    disk_manager.read_page_data(page_id_bef, &mut buf)?;
    println!("{:?}", buf);

    Ok(())
}
