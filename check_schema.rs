// 临时脚本：检查 Zotero 数据库 items 表的实际列名
use rusqlite::Connection;

fn main() {
    let db_path = "D:\\Zotero\\Date-Directary\\zotero.sqlite";
    let conn = Connection::open(db_path).expect("无法打开数据库");

    println!("=== items 表结构 ===");
    let mut stmt = conn.prepare("PRAGMA table_info(items)").expect("查询失败");
    let cols = stmt.query_map([], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
    }).expect("查询失败");
    for col in cols {
        let (cid, name, dtype) = col.unwrap();
        println!(" {}: {} ({})", cid, name, dtype);
    }

    println!("\n=== items 表数据样本 ===");
    let mut stmt = conn.prepare("SELECT * FROM items LIMIT 2").expect("查询失败");
    let mut rows = stmt.query([]).expect("查询失败");
    while let Some(row) = rows.next().expect("查询失败") {
        println!("{:?}", row);
    }
}