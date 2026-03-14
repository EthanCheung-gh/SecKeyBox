# SecKeyBox Phase 3 - 剩余修复任务

## 已完成 ✅

### Phase 3 核心功能
- [x] Favorites - 收藏功能（星星标记、侧边栏筛选）
- [x] Dark/Light Theme - 暗色/亮色主题切换
- [x] Password Strength - 密码强度指示器
- [x] Import/Export - 数据导入导出（支持 Merge/Replace 模式）

### 稳定性修复
- [x] 数据库连接问题 - 修复了内存数据库与文件数据库切换问题
- [x] 密码验证 - 修复了构建后密码失效的问题
- [x] Schema 迁移 - 修复了重复迁移导致的错误

## 待修复 🐛

### 1. API Keys / Environment Variables 添加失败
**问题：** 除了 Account 分组外，其他分组无法成功添加 item
**错误信息：** `invalid args 'groupId' for command 'create_new_api_key_item'`

**原因分析：**
- Tauri 2.0 的参数序列化默认使用 camelCase
- 后端 Rust 函数期望 snake_case 参数名（group_id）
- 但前端传递的是 camelCase（groupId）

**解决方案选项：**
1. **选项 A：** 修改前端调用方式，使用对象包装参数（已尝试部分实现）
2. **选项 B：** 在 Rust 端使用结构体接收参数（CreateApiKeyRequest）
3. **选项 C：** 统一使用 camelCase 作为后端参数名（最简单）

**建议：** 使用选项 C，将所有后端函数的参数名改为 camelCase

### 2. 删除 Item 后不立即刷新
**问题：** 点击删除后 item 仍在列表中，需要手动刷新才消失

**可能原因：**
- deleteItem 后调用 loadItems 没有正确更新 UI
- 或者 selectedItem 状态影响

## 修复步骤

### 修复 API Keys/Env Var 添加问题

1. 修改 `src-tauri/src/commands/items.rs`：
   - 将 `create_new_api_key_item` 的参数从 `group_id` 改为 `groupId`
   - 将 `create_new_env_var_item` 的参数从 `group_id` 改为 `groupId`
   - 同时更新 update 函数

2. 修改 `src/stores/vault.ts`：
   - 调用时使用正确的参数名

### 修复删除不刷新问题

1. 在 `deleteItem` 成功后：
   - 立即清除 selectedItem（如果删除的是当前选中的）
   - 确保 loadItems 正确触发 UI 更新

## 测试清单

### 添加 Item
- [ ] 在 Accounts 分组添加账号 ✓
- [ ] 在 API Keys 分组添加 API Key
- [ ] 在 Environment Variables 分组添加环境变量

### 删除 Item
- [ ] 删除 item 后立即从列表消失
- [ ] 如果删除的是当前选中的，自动清空详情面板

### 其他功能
- [ ] Favorites 筛选
- [ ] 主题切换
- [ ] 导入/导出
