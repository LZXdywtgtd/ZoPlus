# Zotero 数据库结构报告

## 1. 数据库基本信息

| 属性 | 值 |
| ---- | --- |
| 数据库路径 | `D:\Zotero\Date-Directary\zotero.sqlite` |
| 文件大小 | 5.0 MB |
| SQLite 版本 | 3.46.0 |
| 总表数 | 61 |

## 2. 所有表清单

### 2.1 items 表（文献表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | itemID | INTEGER | 否 | - | 是 |
| 1 | itemTypeID | INT | 是 | - | 否 |
| 2 | libraryID | INT | 是 | - | 否 |
| 3 | key | TEXT | 是 | - | 否 |
| 4 | name | TEXT | 否 | - | 否 |
| 5 | dateAdded | TIMESTAMP | 是 | CURRENT_TIMESTAMP | 否 |
| 6 | dateModified | TIMESTAMP | 是 | CURRENT_TIMESTAMP | 否 |
| 7 | clientDateModified | TIMESTAMP | 是 | CURRENT_TIMESTAMP | 否 |
| 8 | synced | INT | 是 | 0 | 否 |
| 9 | version | INT | 是 | 0 | 否 |

**说明**：items 表是 Zotero 的核心文献表，存储所有文献元数据的主信息。

### 2.2 itemCreators 表（文献-作者关联表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | itemID | INT | 是 | - | 是 |
| 1 | creatorID | INT | 是 | - | 否 |
| 2 | creatorTypeID | INT | 是 | 1 | 否 |
| 3 | orderIndex | INT | 是 | 0 | 否 |

**说明**：itemCreators 是连接 items 表和 creators 表的关联表。
- `orderIndex` 表示作者顺序（0 = 第一作者）
- `creatorTypeID` 表示作者类型（1 = 正常作者，其他值待查 creatorTypes 表）
- 使用 `(itemID, creatorID)` 复合主键

### 2.3 creators 表（作者表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | creatorID | INTEGER | 否 | - | 是 |
| 1 | firstName | TEXT | 否 | - | 否 |
| 2 | lastName | TEXT | 否 | - | 否 |
| 3 | fieldMode | INT | 否 | - | 否 |

**说明**：creators 表存储作者信息。
- `firstName` 和 `lastName` 分别存储名字和姓氏
- `fieldMode` 字段用于区分单字段模式（如 "Wang Tao"）和双字段模式（firstName + lastName）
- 当前数据示例显示 fieldMode = 0（双字段模式）

### 2.4 creatorTypes 表（作者类型表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | creatorTypeID | INTEGER | 否 | - | 是 |
| 1 | creatorType | TEXT | 否 | - | 否 |

### 2.5 itemTypes 表（文献类型表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | itemTypeID | INTEGER | 否 | - | 是 |
| 1 | type | TEXT | 否 | - | 否 |

### 2.6 itemData 表（文献数据表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | itemID | INT | 是 | - | 是 |
| 1 | fieldID | INT | 是 | - | 否 |
| 2 | value | TEXT | 否 | - | 否 |

**说明**：itemData 存储文献的动态字段数据，通过 fieldID 关联 itemDataFields 表获取字段名。

### 2.7 itemDataFields 表（文献字段定义表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | fieldID | INTEGER | 否 | - | 是 |
| 1 | fieldName | TEXT | 否 | - | 否 |

### 2.8 itemTags 表（文献-标签关联表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | itemID | INT | 是 | - | 是 |
| 1 | tagID | INT | 是 | - | 否 |
| 2 | orderIndex | INT | 否 | - | 否 |

### 2.9 tags 表（标签表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | tagID | INTEGER | 否 | - | 是 |
| 1 | name | TEXT | 是 | - | 否 |

### 2.10 collections 表（集合表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | collectionID | INTEGER | 否 | - | 是 |
| 1 | libraryID | INT | 是 | - | 否 |
| 2 | name | TEXT | 是 | - | 否 |
| 3 | key | TEXT | 是 | - | 否 |
| 4 | parentCollectionID | INT | 否 | - | 否 |
| 5 | clientDateModified | TIMESTAMP | 是 | CURRENT_TIMESTAMP | 否 |
| 6 | synced | INT | 是 | 0 | 否 |
| 7 | version | INT | 是 | 0 | 否 |

### 2.11 collectionItems 表（集合-文献关联表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | collectionID | INT | 是 | - | 是 |
| 1 | itemID | INT | 是 | - | 否 |
| 2 | orderIndex | INT | 是 | 0 | 否 |

### 2.12 settings 表（设置表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | setting | TEXT | 否 | - | 是 |
| 1 | key | TEXT | 否 | - | 否 |
| 2 | value | BLOB | 否 | - | 否 |

### 2.13 itemNotes 表（文献笔记表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | itemID | INTEGER | 否 | - | 是 |
| 1 | note | TEXT | 否 | - | 否 |
| 2 | parentItemID | INT | 否 | - | 否 |
| 3 | clientDateModified | TIMESTAMP | 否 | CURRENT_TIMESTAMP | 否 |

### 2.14 deletedItems 表（已删除文献表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | itemID | INTEGER | 否 | - | 是 |
| 1 | dateDeleted | TEXT | 是 | CURRENT_TIMESTAMP | 否 |

### 2.15 fulltextItems 表（全文索引表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | itemID | INTEGER | 否 | - | 是 |
| 1 | version | INT | 否 | - | 否 |
| 2 | indexedPages | TEXT | 否 | - | 否 |

### 2.16 fulltextWords 表（全文词表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | itemID | INT | 是 | - | 是 |
| 1 | word | TEXT | 否 | - | 否 |
| 2 | wordLocations | TEXT | 否 | - | 否 |

### 2.17 itemAttributes 表（文献属性表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | itemID | INT | 是 | - | 是 |
| 1 | attributeID | INT | 是 | - | 否 |
| 2 | value | TEXT | 否 | - | 否 |

### 2.18 itemRatings 表（文献评分表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ------ | ---- |
| 0 | itemID | INTEGER | 否 | - | 是 |
| 1 | rating | INT | 否 | - | 否 |

### 2.19 itemRelations 表（文献关系表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | itemID | INT | 是 | - | 是 |
| 1 | predicateID | INT | 是 | - | 否 |
| 2 | object | TEXT | 是 | - | 否 |

### 2.20 relationPredicates 表（关系谓词表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | predicateID | INTEGER | 否 | - | 是 |
| 1 | predicate | TEXT | 否 | - | 否 |

### 2.21 libraries 表（文献库表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | libraryID | INTEGER | 否 | - | 是 |
| 1 | type | INT | 是 | - | 否 |
| 2 | name | TEXT | 否 | - | 否 |
| 3 | edition | INT | 是 | 0 | 否 |

### 2.22 savedSearches 表（保存的搜索表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | savedSearchID | INTEGER | 否 | - | 是 |
| 1 | savedSearchName | TEXT | 是 | - | 否 |
| 2 | clientDateModified | TIMESTAMP | 是 | CURRENT_TIMESTAMP | 否 |
| 3 | libraryID | INT | 是 | - | 否 |
| 4 | key | TEXT | 是 | - | 否 |
| 5 | version | INT | 是 | 0 | 否 |
| 6 | synced | INT | 是 | 0 | 否 |

### 2.23 savedSearchConditions 表（搜索条件表）

| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | savedSearchID | INT | 是 | - | 是 |
| 1 | searchConditionID | INT | 是 | - | 否 |
| 2 | condition | TEXT | 是 | - | 否 |
| 3 | operator | TEXT | 否 | - | 否 |
| 4 | value | TEXT | 否 | - | 否 |
| 5 | required | NONE | 否 | - | 否 |

### 2.24 同步相关表

#### syncObjectTypes 表
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | syncObjectTypeID | INTEGER | 否 | - | 是 |
| 1 | name | TEXT | 否 | - | 否 |

#### syncQueue 表
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | libraryID | INT | 是 | - | 是 |
| 1 | key | TEXT | 是 | - | 否 |
| 2 | syncObjectTypeID | INT | 是 | - | 否 |
| 3 | lastCheck | TIMESTAMP | 否 | - | 否 |
| 4 | tries | INT | 否 | - | 否 |

#### syncCache 表
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | libraryID | INT | 是 | - | 是 |
| 1 | key | TEXT | 是 | - | 否 |
| 2 | syncObjectTypeID | INT | 是 | - | 否 |
| 3 | version | INT | 是 | - | 否 |
| 4 | data | TEXT | 否 | - | 否 |

#### syncDeleteLog 表
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | syncObjectTypeID | INT | 是 | - | 否 |
| 1 | libraryID | INT | 是 | - | 否 |
| 2 | key | TEXT | 是 | - | 否 |
| 3 | dateDeleted | TEXT | 是 | CURRENT_TIMESTAMP | 否 |

#### syncedSettings 表
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | setting | TEXT | 是 | - | 是 |
| 1 | libraryID | INT | 是 | - | 否 |
| 2 | value | BLOB | 是 | - | 否 |
| 3 | version | INT | 是 | 0 | 否 |
| 4 | synced | INT | 是 | 0 | 否 |

### 2.25 其他重要表

#### baseFieldMappings 表（基础字段映射表）
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | itemTypeID | INT | 否 | - | 是 |
| 1 | baseFieldID | INT | 否 | - | 否 |
| 2 | fieldID | INT | 否 | - | 否 |

#### itemTypeFields 表（文献类型字段表）
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | itemTypeID | INT | 否 | - | 是 |
| 1 | fieldID | INT | 否 | - | 否 |
| 2 | orderIndex | INT | 否 | - | 否 |

#### customItemData 表（自定义数据表）
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | itemID | INT | 是 | - | 是 |
| 1 | fieldID | INT | 是 | - | 否 |
| 2 | value | TEXT | 否 | - | 否 |

#### dataCache 表（数据缓存表）
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | cacheID | INTEGER | 否 | - | 是 |
| 1 | lastModified | INT | 否 | - | 否 |
| 2 | data | TEXT | 否 | - | 否 |

#### proxyProxies 表（代理表）
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | proxyID | INTEGER | 否 | - | 是 |
| 1 | hostname | TEXT | 否 | - | 否 |
| 2 | port | INT | 否 | - | 否 |
| 3 | ssl | INT | 否 | - | 否 |
| 4 | autoAssociate | INT | 否 | - | 否 |

#### proxyHosts 表（代理主机表）
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | hostID | INTEGER | 否 | - | 是 |
| 1 | proxyID | INTEGER | 否 | - | 否 |
| 2 | hostname | TEXT | 否 | - | 否 |

#### retractedItems 表（撤稿表）
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | itemID | INTEGER | 否 | - | 是 |
| 1 | data | TEXT | 否 | - | 否 |
| 2 | flag | INT | 否 | 0 | 否 |

#### storageDeleteLog 表（存储删除日志表）
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | libraryID | INT | 是 | - | 是 |
| 1 | key | TEXT | 是 | - | 否 |
| 2 | dateDeleted | TEXT | 是 | CURRENT_TIMESTAMP | 否 |

#### translatorCache 表（翻译器缓存表）
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | fileName | TEXT | 否 | - | 是 |
| 1 | metadataJSON | TEXT | 否 | - | 否 |
| 2 | lastModifiedTime | INT | 否 | - | 否 |

#### users 表（用户表）
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | userID | INTEGER | 否 | - | 是 |
| 1 | name | TEXT | 是 | - | 否 |

#### version 表（版本表）
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | schema | TEXT | 否 | - | 是 |
| 1 | version | INT | 是 | - | 否 |

#### charsets 表（字符集表）
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | charsetID | INTEGER | 否 | - | 是 |
| 1 | charset | TEXT | 否 | - | 否 |

#### collectionRelations 表（集合关系表）
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | collectionID | INT | 是 | - | 是 |
| 1 | predicateID | INT | 是 | - | 否 |
| 2 | object | TEXT | 是 | - | 否 |

#### libraryDropStyles 表
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | libraryID | INT | 是 | - | 是 |
| 1 | styleID | TEXT | 是 | - | 否 |

#### publicationsItems 表
| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |
| ---- | ------ | ---- | ---- | ---- | ---- |
| 0 | itemID | INTEGER | 否 | - | 是 |

## 3. 作者表关系图

```
items (文献)
    |
    +---> itemCreators (文献-作者关联)
              |
              +---> creators (作者信息)
                        |
                        +---> creatorTypes (作者类型定义)
```

### 详细关联说明

1. **items -> itemCreators**：一对多关系
   - 一个文献可以有多个作者（通过 itemCreators 连接）
   - itemCreators.itemID -> items.itemID

2. **itemCreators -> creators**：多对一关系
   - 多个文献引用同一作者时，只在 creators 表中存一条记录
   - itemCreators.creatorID -> creators.creatorID

3. **itemCreators.creatorTypeID -> creatorTypes.creatorTypeID**：多对一关系
   - 确定每个作者的作者类型（第一作者、编辑、译者等）

### 作者数据示例

**creators 表示例**：
| creatorID | firstName | lastName | fieldMode |
| --------- | --------- | -------- | --------- |
| 1 | Tao | Wang | 0 |
| 2 | Yichen | Zhang | 0 |
| 3 | Haoyue | Han | 0 |
| 4 | Lei | Wang | 0 |
| 5 | Xuan | Ye | 0 |

**itemCreators 表示例**：
| itemID | creatorID | orderIndex | creatorTypeID |
| ------ | --------- | ---------- | ------------- |
| 2 | 1 | 0 | 10 |
| 2 | 2 | 1 | 10 |
| 2 | 3 | 2 | 10 |
| 2 | 4 | 3 | 10 |
| 2 | 5 | 4 | 10 |

**说明**：
- orderIndex = 0 表示第一作者
- creatorTypeID = 10 表示"普通作者"类型
- fieldMode = 0 表示双字段模式（firstName + lastName 分开存储）

## 4. 其他核心表关系

### 文献数据存储架构
```
items (文献主表)
    |
    +---> itemData (动态字段数据)
    |         |
    |         +---> itemDataFields (字段定义)
    |
    +---> itemCreators ---> creators (作者信息)
    |
    +---> itemTags ---> tags (标签)
    |
    +---> itemNotes (笔记)
    |
    +---> collectionItems ---> collections (集合)
    |
    +---> itemRelations ---> relationPredicates (关系谓词)
    |
    +---> itemRatings (评分)
    |
    +---> itemAttributes (属性)
    |
    +---> fulltextItems ---> fulltextWords (全文索引)
    |
    +---> deletedItems (已删除)
    |
    +---> retractedItems (撤稿信息)
```

### 集合/文件夹架构
```
collections (集合表)
    |
    +---> collectionItems ---> items (文献)
    |
    +---> collectionRelations (集合间关系)

collectionItems 表结构：
- collectionID: 集合ID
- itemID: 文献ID
- orderIndex: 在集合中的排序
```

### 同步相关架构
```
syncQueue (同步队列)
syncCache (同步缓存)
syncDeleteLog (同步删除日志)
syncedSettings (已同步设置)

syncObjectTypes (同步对象类型)
    |
    +---> syncQueue.syncObjectTypeID
    +---> syncCache.syncObjectTypeID
```

## 5. 结论与建议

### 5.1 数据库特点
1. **61 个表**：Zotero 数据库结构复杂，支持丰富的元数据功能
2. **规范化设计**：作者、标签、集合等使用独立表存储，通过关联表连接
3. **全文索引**：使用 fulltextItems 和 fulltextWords 表支持 PDF 全文搜索
4. **同步机制**：完整的同步队列和缓存机制支持多设备同步

### 5.2 ZoPlus 开发建议

#### 作者信息查询
```sql
-- 查询某文献的所有作者（按顺序）
SELECT c.firstName, c.lastName, c.fieldMode, ic.orderIndex
FROM itemCreators ic
JOIN creators c ON ic.creatorID = c.creatorID
WHERE ic.itemID = ?
ORDER BY ic.orderIndex;
```

#### 关键注意事项
1. **只读访问**：开发阶段应严格遵守只读原则，不修改 Zotero 原生数据
2. **extra 字段**：如需存储自定义数据，建议使用 items 表的 extra 字段或新建独立表
3. **字段映射**：itemData 表通过 fieldID 存储动态字段，需结合 itemDataFields 获取字段名
4. **作者类型**：creatorTypeID 的具体含义需查询 creatorTypes 表获取

#### 索引优化建议
1. itemCreators 表的 (itemID, orderIndex) 是高频查询，应确保索引有效
2. items 表的 (libraryID, key) 是唯一性查询的关键索引
3. fulltextWords 表的 word 字段是全文搜索的基础

### 5.3 数据安全
1. 所有数据库操作使用 `PRAGMA query_only = ON` 强制只读模式
2. 写入操作仅允许写入 extra 字段或项目自建数据表
3. 禁止修改 Zotero 原生业务字段，避免破坏 Zotero Word 插件兼容性
