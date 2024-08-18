use std::io;
use std::ops::{Index, IndexMut};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};
use thiserror;

use crate::disk::{DiskManager, PageId, PAGE_SIZE};

pub type Page = [u8; PAGE_SIZE];

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("no free buffer available in buffer pool")]
    NoFreeBuffer,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BufferId(usize);
// RefCell: データ競合について、コンパイル時ではなく実行時に検査する
// Cell: 読み取り専用の値の中に書き込み可能な値をつくる
#[derive(Debug)]
pub struct Buffer {
    pub page_id: PageId,
    pub page: RefCell<Page>,
    pub is_dirty: Cell<bool>, // BufferでPageに対してWriteした場合、is_dirtyをフラグとしてもつ(Diskに戻す時に整合性を取るため)
}
impl Default for Buffer {
    fn default() -> Self {
        Self {
            page_id: Default::default(),
            page: RefCell::new([0u8; PAGE_SIZE]),
            is_dirty: Cell::new(false),
        }
    }
}
// Bufferに対するpointer + metadata(usage_countはそのbufferがどれだけ使われたかを持つ)
// Rc: 対象のデータへの参照の数を実行時にtrackingする. カウントが0になってどこからも利用されてないことがわかったら対象のメモリ領域を自動で解放する
//     Rc::cloneをすることによって、対象に対するReference counterがincrementされる
#[derive(Debug, Default)]
pub struct Frame {
    usage_count: u64,
    buffer: Rc<Buffer>, // MySQLのような、一つのsessionに対して一つのThreadを割り当てるような設計を実現するためにはRcではなくて、Arcを使う必要が出てくる
}
#[derive(Debug)]
pub struct BufferPool {
    buffers: Vec<Frame>,
    next_victim_id: BufferId, // linked-list
}

impl Index<BufferId> for BufferPool {
    type Output = Frame;
    fn index(&self, index: BufferId) -> &Self::Output {
        &self.buffers[index.0]
    }
}

impl IndexMut<BufferId> for BufferPool {
    fn index_mut(&mut self, index: BufferId) -> &mut Self::Output {
        &mut self.buffers[index.0]
    }
}

// BufferPoolを管理する責務
// どれだけのbufferをpoolingするかは、initialize(BufferPool::new)時に引数で渡す
impl BufferPool {
    pub fn new(pool_size: usize) -> Self {
        let mut buffers = vec![];
        buffers.resize_with(pool_size, Default::default);
        let next_victim_id = BufferId::default();
        Self {
            buffers,
            next_victim_id,
        }
    }
    fn size(&self) -> usize {
        self.buffers.len()
    }
    // 捨てるbufferを決めて、そのBufferIdを返す
    // BufferPoolで管理しているFrameたち(linked-listで繋がっている)をなめて、精査する
    fn evict(&mut self) -> Option<BufferId> {
        // initialize時に注入される、poolingの最大値
        let pool_size = self.size();
        // poolingしてるbufferの貸し出し数管理
        let mut consecutive_pinned = 0;
        let victim_id = loop {
            let next_victim_id = self.next_victim_id;
            let frame = &mut self[next_victim_id];

            // 特定のBufferのcounterが0であればそのBufferIdを返して終了
            if frame.usage_count == 0 {
                break self.next_victim_id;
            }

            // Rc::get_mutは、他に参照が存在する場合にNoneを返す
            // 実行時に参照されていない場合は、usege_countをdecrement
            if Rc::get_mut(&mut frame.buffer).is_some() {
                frame.usage_count -= 1;
                consecutive_pinned = 0;
            } else {
                // 参照があった際の処理
                consecutive_pinned += 1;
                if consecutive_pinned >= pool_size {
                    return None;
                }
            }
            self.next_victim_id = self.increment_id(self.next_victim_id)
        };
        Some(victim_id)
    }
    fn increment_id(&self, buffer_id: BufferId) -> BufferId {
        BufferId((buffer_id.0 + 1) % self.size())
    }
}

// 実際のPageData供給layer(BufferPoolとDiskを両方みて、BufferPoolにあればそこから、なければDiskから取ってきてよしなに供給する責務)
// poolを見に行って、あれば使い、なければDiskManager越しにDisk I/O
#[derive(Debug)]
pub struct BufferPoolManager {
    disk: DiskManager,
    pool: BufferPool,
    page_table: HashMap<PageId, BufferId>, // どのpageデータがどのbufferに入っているかをmapping
}

impl BufferPoolManager {
    pub fn new(disk: DiskManager, pool: BufferPool) -> Self {
        let page_table = HashMap::new();
        Self {
            disk,
            pool,
            page_table,
        }
    }
    // Pageの実データを返す(Frame.buffer)
    pub fn fetch_page(&mut self, page_id: PageId) -> Result<Rc<Buffer>, Error> {
        // BufferPoolに必要なpageが存在する場合(HashMapで持っているpage_tableを探索)
        if let Some(&buffer_id) = self.page_table.get(&page_id) {
            let frame = &mut self.pool[buffer_id];
            frame.usage_count += 1;
            // Reference Counterをincrement
            return Ok(Rc::clone(&frame.buffer));
        }
        // BufferPoolに存在せず、Diskからfetchする必要がある場合
        // diskからfetchしたpageを乗せるbufferを探索する(必要無くなったBuffer) -> 存在しなければ早期return
        let buffer_id = self.pool.evict().ok_or(Error::NoFreeBuffer)?;
        // メモリキャッシュの必要がなくなったPageデータを管理しているFrame(捨てられる予定)
        let frame = &mut self.pool[buffer_id];
        // いらないbufferのpageデータのPageId
        let evict_page_id = frame.buffer.page_id;
        {
            let buffer = Rc::get_mut(&mut frame.buffer).unwrap();
            // cacheしてからPageに対してWriteしていた場合、Diskに戻す際にDisk Writeが必要になる
            if buffer.is_dirty.get() {
                self.disk
                    .write_page_data(evict_page_id, buffer.page.get_mut())?;
            }
            // 捨てる対象のPageの整合性が取れたので、新しくBufferingするPageに関する処理に進む
            buffer.page_id = page_id;
            buffer.is_dirty.set(false);

            self.disk.read_page_data(page_id, buffer.page.get_mut())?;
            frame.usage_count = 1;
        }

        let page = Rc::clone(&frame.buffer);
        self.page_table.remove(&evict_page_id);
        self.page_table.insert(page_id, buffer_id);
        Ok(page)
    }

    // 既存のPageをfetchするのではなく、そもそもPage作成から行う
    pub fn create_page(&mut self) -> Result<Rc<Buffer>, Error> {
        let buffer_id = self.pool.evict().ok_or(Error::NoFreeBuffer)?;
        let frame = &mut self.pool[buffer_id];
        let evict_page_id = frame.buffer.page_id;
        let page_id = {
            let buffer = Rc::get_mut(&mut frame.buffer).unwrap();
            if buffer.is_dirty.get() {
                self.disk
                    .write_page_data(evict_page_id, buffer.page.get_mut())?;
            }
            let page_id = self.disk.allocate_page();
            *buffer = Buffer::default();
            buffer.page_id = page_id;
            buffer.is_dirty.set(true);
            frame.usage_count = 1;
            page_id
        };
        let page = Rc::clone(&frame.buffer);
        self.page_table.remove(&evict_page_id);
        self.page_table.insert(page_id, buffer_id);
        Ok(page)
    }
}
