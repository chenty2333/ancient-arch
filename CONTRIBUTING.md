# 贡献指南 (Contributing Guide)

感谢您对 Ancient Arch 项目的关注！我们欢迎各种形式的贡献。

## 📋 目录

- [行为准则](#行为准则)
- [如何贡献](#如何贡献)
- [开发流程](#开发流程)
- [代码规范](#代码规范)
- [提交规范](#提交规范)
- [测试要求](#测试要求)

## 🤝 行为准则

参与本项目即表示您同意遵守以下准则：

- **尊重他人**: 尊重所有贡献者和用户
- **建设性反馈**: 提供有建设性的批评和建议
- **包容性**: 欢迎不同背景和经验水平的贡献者
- **专业性**: 保持专业和友好的态度

## 🎯 如何贡献

### 报告 Bug

发现 Bug？请通过以下步骤报告：

1. **检查已有 Issue**: 确认问题尚未被报告
2. **创建新 Issue**: 使用 Bug 报告模板
3. **提供详细信息**:
   - 问题描述
   - 复现步骤
   - 预期行为
   - 实际行为
   - 环境信息 (OS, Rust 版本等)
   - 日志和截图

### 建议新功能

有好想法？我们很乐意听取：

1. **创建 Feature Request Issue**
2. **描述清楚**:
   - 功能描述
   - 使用场景
   - 预期收益
   - 实现建议 (可选)

### 贡献代码

#### 小改动 (文档、拼写错误等)
1. Fork 仓库
2. 创建分支
3. 提交更改
4. 创建 Pull Request

#### 大改动 (新功能、重构等)
1. **先创建 Issue 讨论**: 避免重复工作
2. **等待反馈**: 确认方向正确后再开始
3. **按开发流程进行**

## 🔄 开发流程

### 1. 准备环境

```bash
# Fork 并克隆仓库
git clone https://github.com/YOUR_USERNAME/ancient-arch.git
cd ancient-arch

# 添加上游仓库
git remote add upstream https://github.com/chenty2333/ancient-arch.git

# 安装依赖
cd backend
cargo build
```

### 2. 创建分支

```bash
# 从 main 分支创建新分支
git checkout main
git pull upstream main
git checkout -b feature/your-feature-name

# 分支命名规范:
# - feature/xxx: 新功能
# - fix/xxx: Bug 修复
# - docs/xxx: 文档更新
# - refactor/xxx: 代码重构
# - test/xxx: 测试相关
```

### 3. 开发

#### 运行开发服务器

```bash
# 启动数据库
docker-compose up -d db

# 配置环境变量
cp .env.example .env
# 编辑 .env 文件

# 运行应用
cd backend
cargo run
```

#### 代码编写

- 遵循 [代码规范](#代码规范)
- 编写清晰的代码注释
- 更新相关文档
- 添加或更新测试

### 4. 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 运行并显示输出
cargo test -- --nocapture

# 检查代码格式
cargo fmt --check

# 运行 Clippy 检查
cargo clippy -- -D warnings
```

### 5. 提交代码

```bash
# 暂存更改
git add .

# 提交 (遵循提交规范)
git commit -m "feat: add user profile feature"

# 推送到你的 Fork
git push origin feature/your-feature-name
```

### 6. 创建 Pull Request

1. 访问 GitHub 上你的 Fork
2. 点击 "New Pull Request"
3. 填写 PR 模板:
   - 清晰的标题
   - 详细的描述
   - 关联的 Issue
   - 截图 (如有 UI 更改)
4. 提交 PR 并等待审核

### 7. 代码审核

- **回应反馈**: 及时回复审核意见
- **更新代码**: 根据反馈修改代码
- **保持同步**: 定期合并上游更改
  ```bash
  git fetch upstream
  git rebase upstream/main
  git push --force-with-lease origin feature/your-feature-name
  ```

## 📝 代码规范

### Rust 代码风格

遵循官方 [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/):

```rust
// ✅ 好的示例
pub async fn get_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
        .fetch_one(&pool)
        .await?;
    
    Ok(Json(user))
}

// ❌ 避免
pub async fn get_user(State(pool):State<PgPool>,Path(id):Path<i64>)->Result<Json<User>,AppError>{
let user=sqlx::query_as!(User,"SELECT * FROM users WHERE id = $1",id).fetch_one(&pool).await?;
Ok(Json(user))}
```

### 命名规范

- **函数**: `snake_case`
- **类型**: `PascalCase`
- **常量**: `SCREAMING_SNAKE_CASE`
- **模块**: `snake_case`

```rust
// 结构体和枚举
pub struct UserProfile { }
pub enum UserRole { Admin, User }

// 函数
pub async fn create_user() { }
pub async fn get_user_profile() { }

// 常量
pub const MAX_LOGIN_ATTEMPTS: u32 = 5;
pub const DEFAULT_PAGE_SIZE: i64 = 20;
```

### 注释规范

```rust
/// 创建新用户账户。
///
/// # 参数
/// * `pool` - 数据库连接池
/// * `payload` - 用户注册信息
///
/// # 返回
/// 成功返回新创建的用户信息，失败返回错误。
///
/// # 错误
/// - `AppError::Conflict`: 用户名已存在
/// - `AppError::BadRequest`: 输入验证失败
pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 实现...
}
```

### 错误处理

```rust
// ✅ 使用 ? 操作符
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
    .fetch_one(&pool)
    .await?;

// ✅ 提供上下文信息
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch user {}: {:?}", id, e);
        AppError::NotFound(format!("User {} not found", id))
    })?;

// ❌ 避免 unwrap/expect (除非在测试中)
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
    .fetch_one(&pool)
    .await
    .unwrap(); // 不要这样做！
```

## 📨 提交规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

### 格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type 类型

- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式 (不影响代码含义)
- `refactor`: 重构 (既不是新功能也不是 Bug 修复)
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建过程或辅助工具变动

### 示例

```bash
# 新功能
git commit -m "feat(auth): add password reset functionality"

# Bug 修复
git commit -m "fix(api): correct pagination offset calculation"

# 文档
git commit -m "docs(readme): update installation instructions"

# 重构
git commit -m "refactor(handlers): extract common validation logic"

# 多行提交
git commit -m "feat(profile): add user profile management

- Add GET /api/profile/me endpoint
- Add PUT /api/profile/me endpoint
- Add profile validation
- Update user model

Closes #123"
```

## 🧪 测试要求

### 测试类型

#### 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }
}
```

#### 集成测试
```rust
#[tokio::test]
async fn test_user_registration() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    
    let response = client
        .post(&format!("{}/api/auth/register", app))
        .json(&json!({"username": "test", "password": "test123"}))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 201);
}
```

### 测试覆盖率

- **新功能**: 必须包含测试
- **Bug 修复**: 添加防止回归的测试
- **目标覆盖率**: 60%+ (核心功能 80%+)

### 运行测试

```bash
# 所有测试
cargo test

# 特定模块
cargo test handlers::auth

# 显示输出
cargo test -- --nocapture --test-threads=1

# 测试覆盖率 (需要 tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## 🔍 代码审查检查清单

提交 PR 前，请自我检查：

- [ ] 代码符合项目规范
- [ ] 添加了必要的测试
- [ ] 所有测试通过
- [ ] 更新了相关文档
- [ ] 提交信息符合规范
- [ ] 没有引入新的警告
- [ ] 代码格式化 (`cargo fmt`)
- [ ] Clippy 检查通过 (`cargo clippy`)
- [ ] 没有遗留调试代码

## 🎓 学习资源

### Rust 学习
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Async Book](https://rust-lang.github.io/async-book/)

### Axum 框架
- [Axum 官方文档](https://docs.rs/axum/)
- [Axum 示例](https://github.com/tokio-rs/axum/tree/main/examples)

### SQLx
- [SQLx 文档](https://docs.rs/sqlx/)
- [SQLx 指南](https://github.com/launchbadge/sqlx)

## ❓ 获取帮助

- **GitHub Issues**: 技术问题和 Bug
- **GitHub Discussions**: 一般讨论和问题
- **项目文档**: README.md, DEPLOYMENT.md, SECURITY.md

## 🏆 贡献者

感谢所有贡献者！您的贡献将被记录在项目历史中。

---

再次感谢您的贡献！🎉
