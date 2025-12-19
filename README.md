# Ancient Arch - 古建筑管理系统

一个基于 Rust (Axum) 和 PostgreSQL 构建的古建筑知识管理与社区平台。

## 📋 项目概览

Ancient Arch 是一个全栈 Web 应用，旨在提供：
- 古建筑信息展示与管理
- 用户认证与授权系统
- 知识问答与资格考试
- 社区互动功能（帖子、评论、点赞、收藏）
- 贡献者系统与审核流程

## 🏗️ 技术栈

### 后端
- **框架**: Axum 0.8 (异步 Web 框架)
- **数据库**: PostgreSQL 16
- **认证**: JWT + Argon2 密码哈希
- **API 文档**: OpenAPI/Swagger (utoipa)
- **日志**: tracing + tracing-subscriber
- **数据验证**: validator

### 基础设施
- **容器化**: Docker + Docker Compose
- **反向代理**: Nginx (支持速率限制)
- **数据库迁移**: SQLx migrations

## 🚀 快速开始

### 前置要求
- Docker & Docker Compose
- Rust 1.75+ (用于本地开发)

### 使用 Docker Compose 部署

1. 克隆仓库：
```bash
git clone <repository-url>
cd ancient-arch
```

2. 配置环境变量：
```bash
cp .env.example .env
# 编辑 .env 文件，设置安全的密码和密钥
```

3. 启动服务：
```bash
docker-compose up -d
```

4. 访问应用：
- API: http://localhost:8080/api
- Swagger 文档: http://localhost:8080/api/doc

### 本地开发

1. 安装依赖：
```bash
cd backend
cargo build
```

2. 设置数据库：
```bash
# 确保 PostgreSQL 正在运行
# 配置 .env 中的 DATABASE_URL
```

3. 运行迁移：
```bash
cargo sqlx migrate run
```

4. 启动服务：
```bash
cargo run
```

## 📚 API 文档

### 认证相关
- `POST /api/auth/register` - 用户注册
- `POST /api/auth/login` - 用户登录
- `GET /api/auth/qualification` - 获取资格考试题目
- `POST /api/auth/qualification/submit` - 提交考试答案

### 古建筑管理
- `GET /api/architectures` - 获取建筑列表
- `GET /api/architectures/{id}` - 获取建筑详情
- `POST /api/admin/architectures` - 创建建筑 (管理员)
- `PUT /api/admin/architectures/{id}` - 更新建筑 (管理员)
- `DELETE /api/admin/architectures/{id}` - 删除建筑 (管理员)

### 社区功能
- `GET /api/posts` - 获取帖子列表
- `GET /api/posts/{id}` - 获取帖子详情
- `POST /api/posts` - 创建帖子 (需认证)
- `DELETE /api/posts/{id}` - 删除帖子 (需认证)
- `POST /api/posts/{id}/like` - 点赞/取消点赞
- `POST /api/posts/{id}/favorite` - 收藏/取消收藏
- `POST /api/posts/{id}/comments` - 添加评论
- `GET /api/posts/{id}/comments` - 获取评论列表

### 问答系统
- `GET /api/quiz/generate` - 生成随机试卷
- `POST /api/quiz/submit` - 提交试卷 (需认证)
- `GET /api/quiz/leaderboard` - 查看排行榜

### 用户管理
- `GET /api/profile/me` - 获取个人信息
- `GET /api/profile/posts` - 获取我的帖子
- `GET /api/profile/favorites` - 获取我的收藏
- `GET /api/profile/contributions` - 获取我的贡献

### 贡献系统
- `POST /api/contributions` - 提交贡献 (需认证)
- `GET /api/admin/contributions` - 获取贡献列表 (管理员)
- `PUT /api/admin/contributions/{id}/review` - 审核贡献 (管理员)

### 管理员功能
- `GET /api/admin/users` - 用户列表
- `POST /api/admin/users` - 创建用户
- `PUT /api/admin/users/{id}` - 更新用户
- `DELETE /api/admin/users/{id}` - 删除用户
- `POST /api/admin/questions` - 创建题目
- `PUT /api/admin/questions/{id}` - 更新题目
- `DELETE /api/admin/questions/{id}` - 删除题目

## 🏆 项目完成度评估

### ✅ 已完成的功能

#### 核心功能 (90%)
- ✅ 用户认证系统 (注册、登录、JWT)
- ✅ 权限控制 (用户、管理员角色)
- ✅ 古建筑信息管理 (CRUD)
- ✅ 问答系统 (题目生成、提交、评分)
- ✅ 社区功能 (帖子、评论、点赞、收藏)
- ✅ 贡献者系统 (提交、审核、验证)
- ✅ 数据库迁移系统

#### 安全特性 (85%)
- ✅ Argon2 密码哈希
- ✅ JWT 令牌认证
- ✅ CORS 配置
- ✅ 输入验证 (validator)
- ✅ SQL 注入防护 (SQLx 编译时检查)
- ✅ Nginx 速率限制
- ✅ 环境变量配置管理

#### 基础设施 (80%)
- ✅ Docker 容器化
- ✅ Docker Compose 编排
- ✅ 数据库健康检查
- ✅ 自动数据库迁移
- ✅ 日志系统 (文件 + 控制台)
- ✅ Nginx 反向代理

#### 代码质量 (75%)
- ✅ 模块化架构
- ✅ 错误处理统一化
- ✅ 异步编程模式
- ✅ 基础集成测试
- ✅ API 文档 (Swagger)

### ⚠️ 需要改进的部分

#### 测试覆盖率 (40%)
- ⚠️ 单元测试覆盖率低
- ⚠️ 仅有 2 个集成测试文件
- ⚠️ 缺少端到端测试
- ⚠️ 缺少性能测试
- ⚠️ 缺少安全测试

#### 文档 (30%)
- ⚠️ 缺少项目 README (本文档刚添加)
- ⚠️ 缺少 API 使用示例
- ⚠️ 缺少部署指南
- ⚠️ 缺少贡献指南
- ⚠️ 缺少架构设计文档

#### 安全加固 (60%)
- ⚠️ 默认密码需要更强的策略
- ⚠️ 缺少 HTTPS 配置 (已准备但未启用)
- ⚠️ 缺少请求大小限制
- ⚠️ 缺少会话管理
- ⚠️ 缺少审计日志
- ⚠️ 缺少 CSRF 保护
- ⚠️ 缺少 XSS 防护头

#### 运维功能 (50%)
- ⚠️ 缺少监控和指标
- ⚠️ 缺少健康检查端点
- ⚠️ 缺少优雅关机
- ⚠️ 缺少备份策略文档
- ⚠️ 缺少容量规划指南

#### 前端开发 (10%)
- ⚠️ 仅有占位符 HTML
- ⚠️ 缺少完整的前端应用
- ⚠️ 缺少用户界面

## 🔒 安全性评估

### 强项
1. **密码安全**: 使用 Argon2 进行密码哈希，业界最佳实践
2. **SQL 安全**: SQLx 提供编译时 SQL 检查，防止注入攻击
3. **认证机制**: JWT 令牌有效期控制
4. **网络隔离**: 数据库和后端服务不直接暴露给外网
5. **速率限制**: Nginx 层面的 API 速率限制

### 需要加强的安全措施

#### 高优先级
1. **HTTPS 启用**: 生产环境必须启用 HTTPS
   ```nginx
   # 需要配置 SSL 证书
   ssl_certificate /etc/nginx/certs/origin.pem;
   ssl_certificate_key /etc/nginx/certs/origin.key;
   ```

2. **密钥管理**: 
   - `.env.example` 中的默认值需要强制用户修改
   - 考虑使用密钥管理服务 (AWS Secrets Manager, HashiCorp Vault)

3. **安全响应头**: 
   ```nginx
   add_header X-Frame-Options "SAMEORIGIN";
   add_header X-Content-Type-Options "nosniff";
   add_header X-XSS-Protection "1; mode=block";
   add_header Strict-Transport-Security "max-age=31536000";
   ```

4. **请求大小限制**:
   ```nginx
   client_max_body_size 10M;
   ```

#### 中优先级
5. **密码策略**: 
   - 增加密码复杂度要求
   - 添加密码强度检查

6. **令牌刷新**: 实现刷新令牌机制

7. **审计日志**: 记录敏感操作

8. **输入清理**: 对用户输入进行 HTML/XSS 清理

#### 低优先级
9. **多因素认证**: 支持 2FA
10. **会话管理**: 实现会话撤销功能

## 📊 代码统计

- 总行数: ~3,584 行 Rust 代码
- 模块数: 10+ 个处理器模块
- 数据库表: 11 个表
- 迁移文件: 12 个
- API 端点: 30+ 个

## 🎯 后续开发建议

### 短期目标 (1-2 周)
1. ✍️ 完善项目文档
2. 🔒 启用 HTTPS 配置
3. 🧪 增加单元测试覆盖率到 60%
4. 📝 添加 API 使用示例
5. 🛡️ 添加安全响应头

### 中期目标 (1-2 月)
1. 🎨 开发完整前端应用
2. 📊 添加监控和指标收集
3. 🔐 实现令牌刷新机制
4. 📚 完善 API 文档和示例
5. 🧪 达到 80% 测试覆盖率

### 长期目标 (3-6 月)
1. 🌍 支持国际化 (i18n)
2. 📱 开发移动端应用
3. 🔍 全文搜索功能
4. 📊 数据分析和报表
5. 🤖 CI/CD 流水线自动化

## 🤝 贡献指南

欢迎贡献！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📄 许可证

[待添加许可证信息]

## 📧 联系方式

[待添加联系信息]

---

**注**: 本项目目前处于开发阶段，建议仅用于学习和开发环境。生产部署前需要完成安全加固和全面测试。
