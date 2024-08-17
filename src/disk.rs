use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, Write},
    path::Path,
};

// page sizeは4KB(4096)で設定
pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageId(pub u64);
impl PageId {
    pub fn to_u64(self) -> u64 {
        self.0
    }
}

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
    pub fn allocate_page(&mut self) -> PageId {
        // TODO: どのタイミングで新規pageを作成するでしょう？
        // 新規でpageを作成して、内部カウンタをインクリメント
        let page_id = self.next_page_id;
        self.next_page_id += 1;
        // returnするのは現在のpage_id
        PageId(page_id)
    }
    pub fn write_page_data(&mut self, page_id: PageId, data: &[u8]) -> io::Result<()> {
        let offset = PAGE_SIZE as u64 * page_id.to_u64();
        // heap fileにおいて、次に書き込むべき箇所を特定(page size * page_id)して、その分heap fileの先頭からseekして書き込む
        // ※ seek単位は4096bytesで、誤差は切り上げられるため、使われていないspaceには NULL が入る
        self.heap_file.seek(io::SeekFrom::Start(offset))?;
        self.heap_file.write_all(data)
    }
    pub fn read_page_data(&mut self, page_id: PageId, data: &mut [u8]) -> io::Result<()> {
        let offset = PAGE_SIZE as u64 * page_id.to_u64();
        self.heap_file.seek(io::SeekFrom::Start(offset))?;
        // 読み出し対象のpage idと、buffer(data)を引数に、pageのコンテンツをbufferに対して書き出す
        // read_exact: Reads the exact number of bytes required to fill buf.
        self.heap_file.read_exact(data)
    }
}
