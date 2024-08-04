use std::{
    fs::{File, OpenOptions},
    io,
    path::Path,
};

// page sizeは4KB(4096)で設定
pub const PAGE_SIZE: usize = 4096;

// 特定のファイル(heap_file)を、page(4KB)という単位の配列として捉える
// heap_file = [page0(4KB), page1(4KB), page2(4KB), ...]
#[derive(Debug)]
pub struct DiskManager {
    heap_file: File,
    next_page_id: u64,
}

impl DiskManager {
    pub fn new(heap_file: File) -> io::Result<Self> {
        // file descriptorを受け取り、現状のファイルの書き込まれ具合を取得
        let heap_file_size = heap_file.metadata()?.len();
        // ファイルの状態から、対象のファイルをpageという単位で分けた時に、次に書き込むべきpageはどこなのかを特定(page0, page1, ...etc)
        let next_page_id = heap_file_size / PAGE_SIZE as u64;
        Ok(Self {
            heap_file,
            next_page_id,
        })
    }
    pub fn open(heap_file_path: impl AsRef<Path>) -> io::Result<Self> {
        // pathを指定して、fileをopenする(存在しなければcreate)
        // そのfile descriptorを引数にDiskManager::newを実行して構造体を返す
        let heap_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(heap_file_path)?;
        Self::new(heap_file)
    }
}
