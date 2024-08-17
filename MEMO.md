| RDBMSのアーキテクチャ | 用途 |
| ---- | ---- |
| 構文解析(parser) | クエリをparseして抽象構文木を構築 |
| query planner・実行計画 | 抽象構文木を元に実行計画を作成(EXPLAINで出力してるのはこれ) |
| query executer | 実行計画の通りにアクセスメソッドを呼び出す |
| アクセスメソッド | ディスク上のデータ構造を辿って、結果を返す(B+Treeとか) |
| バッファプールマネージャ | アクセスメソッドの要求に対してディスク上のデータを貸し出す |
| ディスクマネージャー | 実際のディスクの読み書き |

## Chapter2(Disk Manger)
- File I/Oをmanage
- `page`: File I/Oの最小単位
    - 4096bytesの整数倍であることが多い
    - MySQL: 16KB
        - https://dev.mysql.com/doc/refman/8.0/ja/innodb-physical-structure.html
    - PostgreSQL: 8KB
        - https://www.postgresql.org/message-id/3c840f8b-73f0-aae7-6bcf-e22d2a0a6a40%40gusw.net
    - SQLite: 4KB
        - https://www.sqlite.org/pgszchng2016.html
- OSのFileシステムはブロック単位でI/Oを行っていて、それが4096bytesであることがほとんど(?)
    - https://linux.die.net/man/8/mkfs.ext4
    - RDBMSアプリケーション側で、pageサイズを4096よりも小さくしたところで、最終的なOSのFile I/Oで切り上げられてしまうため、そっちに合わせるのが無難
- RustだとこのFile systemを扱っているのが、`std::fs`
    - これがFile I/Oのsyscallをしてくれるcrate
    - ちな`std::fs::File`はファイルディスクリプタ
```rs
use std::{fs::File, io::Write};

fn main() -> std::io::Result<()> {
  let mut file = File::create("example.txt")?;
  file.write_all(b"Hello World\n")?;
  Ok(())
}
```
```sh
$ strace -e trace=open,close,read,write ./target/debug/sandbox
close(3)                                = 0
read(3, "\177ELF\2\1\1\0\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\0\0\0\0\0\0\0\0"..., 832) = 832
close(3)                                = 0
read(3, "\177ELF\2\1\1\3\0\0\0\0\0\0\0\0\3\0>\0\1\0\0\0\220\243\2\0\0\0\0\0"..., 832) = 832
close(3)                                = 0
read(3, "636387e28000-636387e2e000 r--p 0"..., 1024) = 1024
read(3, ":01 6494                       /"..., 1024) = 1024
read(3, "77c307000 r-xp 00001000 ca:01 64"..., 1024) = 789
close(3)                                = 0
write(3, "Hello World\n", 12)           = 12
close(3)                                = 0
+++ exited with 0 +++
```

## Chapter3(BufferPool Manager)
- Disk I/Oの遅さを隠蔽するため
  - Disk I/Oはメモリアクセスに比べるとはるかに遅い(CPUとメモリよりも外側に出るものは基本遅い)
  - page読み込みごとに毎回Disk Managerを呼び出していると性能が悪いため、BufferPool Managerを使ってメモリ上にキャッシュしておくことで高速化する
- 1度目のDiskアクセスの際は遅いが、2度目以降はメモリと同程度の速さで読み出せる
- 全部メモリ上に乗せられるはずもないので、特定のアルゴリズムを使ってどのページをキャッシュして、どのページを捨てるのかを決定する
  - -> Clock-sweep

![スクリーンショット 2024-08-17 13 01 43](https://github.com/user-attachments/assets/9ff5bd7c-aa95-40f4-8ce7-f8434b0932fb)

## Chapter4(アクセスメソッド(B+Tree))
### B-Tree
https://github.com/tsuzuki-takaaki/brain/tree/main/DB/btree
