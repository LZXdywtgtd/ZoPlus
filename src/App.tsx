import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface ItemInfo {
  item_id: number;
  title: string;
  authors: string;
  year: string;
}

function App() {
  const [dbStatus, setDbStatus] = useState<boolean | null>(null);
  const [items, setItems] = useState<ItemInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 检查数据库状态
  async function checkDatabase() {
    try {
      const status = await invoke<boolean>("check_db_status");
      setDbStatus(status);
      return status;
    } catch (e) {
      setError(`数据库状态检查失败: ${e}`);
      return false;
    }
  }

  // 获取文献列表
  async function fetchItems() {
    setLoading(true);
    setError(null);
    try {
      const items = await invoke<ItemInfo[]>("get_items");
      setItems(items);
    } catch (e) {
      setError(`获取文献列表失败: ${e}`);
    } finally {
      setLoading(false);
    }
  }

  // 页面加载时自动检查数据库
  useEffect(() => {
    checkDatabase();
  }, []);

  return (
    <main className="container">
      <h1>ZoPlus - 论文管理软件</h1>

      <div className="row">
        <div className="card">
          <h2>数据库状态</h2>
          <p>
            {dbStatus === null
              ? "检查中..."
              : dbStatus
              ? "已连接 Zotero 数据库"
              : "未找到 Zotero 数据库"}
          </p>
          <button onClick={checkDatabase} disabled={loading}>
            重新检查
          </button>
        </div>
      </div>

     <div className="row">
        <div className="card">
          <h2>文献列表</h2>
          <button onClick={fetchItems} disabled={loading || !dbStatus}>
            {loading ? "加载中..." : "获取文献列表"}
          </button>
        </div>
      </div>

      {error && (
        <div className="row">
          <div className="card error">
            <p>{error}</p>
          </div>
        </div>
      )}

      {items.length > 0 && (
        <div className="row">
          <div className="card">
            <h2>文献列表 ({items.length} 条)</h2>
            <ul>
              {items.slice(0, 20).map((item) => (
                <li key={item.item_id}>
                  <strong>{item.title}</strong>
                  <br />
                  <small>
                    作者: {item.authors || "未知"} | 年份:{" "}
                    {item.year || "未知"}
                  </small>
                </li>
              ))}
            </ul>
            {items.length > 20 && (
              <p>
                <em>仅显示前20 条，共 {items.length} 条文献</em>
              </p>
            )}
          </div>
        </div>
      )}
    </main>
  );
}

export default App;