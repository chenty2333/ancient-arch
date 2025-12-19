# 安全策略 (Security Policy)

## 🔒 安全概述

本文档描述 Ancient Arch 项目的安全实践、已知安全特性和建议的安全改进措施。

## ✅ 已实现的安全特性

### 1. 身份认证与授权

#### 密码安全
- **哈希算法**: Argon2 (2015年密码哈希竞赛冠军)
  - 自动生成随机盐值 (128位)
  - 内存困难算法，抵抗 GPU 破解
  - 默认安全参数配置
  
```rust
// 密码哈希实现 (backend/src/utils/hash.rs)
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        // ...
}
```

#### JWT 令牌
- **签名算法**: HMAC-SHA256
- **令牌过期**: 可配置 (默认 3600 秒)
- **用户信息**: 包含用户 ID 和角色
- **验证**: 每次请求验证签名和过期时间

```rust
// JWT 实现 (backend/src/utils/jwt.rs)
pub fn sign_jwt(
    id: i64,
    _username: &str,
    role: &str,
    secret: &str,
    expiration_seconds: u64,
) -> Result<String, AppError>
```

#### 权限控制
- **角色**: `user` (普通用户), `admin` (管理员)
- **中间件**: 认证中间件 + 管理员中间件
- **验证者系统**: `VerifiedUser` 提取器检查用户验证状态

### 2. 数据库安全

#### SQL 注入防护
- **预编译查询**: 使用 SQLx 的宏和参数化查询
- **编译时检查**: `sqlx::query!` 宏在编译时验证 SQL 语法
- **类型安全**: 自动映射到 Rust 类型

```rust
// 安全的参数化查询示例
sqlx::query!(
    "INSERT INTO users (username, password) VALUES ($1, $2)",
    username,
    hashed_password
)
```

#### 数据完整性
- **外键约束**: 使用 `FOREIGN KEY` 和 `ON DELETE CASCADE`
- **唯一约束**: 用户名、邮箱等字段的唯一性
- **索引**: 提高查询性能并确保数据一致性

### 3. 网络安全

#### 反向代理 (Nginx)
- **速率限制**: 10请求/秒，突发20
  ```nginx
  limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
  limit_req zone=api_limit burst=20 nodelay;
  ```
- **代理头**: 传递真实 IP 和协议信息
- **网络隔离**: 数据库和后端不直接暴露

#### CORS 配置
- **允许来源**: 仅限配置的前端域名
- **允许方法**: GET, POST, PUT, DELETE
- **允许头**: Authorization, Content-Type

```rust
let cors = CorsLayer::new()
    .allow_origin(origins)
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);
```

### 4. 输入验证

#### 数据验证
- **使用**: validator 库进行数据验证
- **规则**:
  - 用户名: 3-20 字符
  - 密码: 4-20 字符 (建议提高到 8+)
  - 自定义验证规则

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 20))]
    pub username: String,
    #[validate(length(min = 4, max = 20))]
    pub password: String,
}
```

### 5. 日志和监控

#### 结构化日志
- **框架**: tracing + tracing-subscriber
- **级别**: 可配置 (info, debug, error)
- **输出**: 文件 + 控制台
- **敏感信息**: 不记录密码和令牌

### 6. 配置管理

#### 环境变量
- **密钥存储**: 通过环境变量配置
- **示例文件**: `.env.example` 提供模板
- **Git 忽略**: `.env` 文件不提交到仓库

## ⚠️ 已知安全问题和建议

### 🔴 高优先级

#### 1. HTTPS 未启用
**问题**: 生产环境 HTTPS 配置被注释
```nginx
# server {
#     listen 443 ssl;
#     ssl_certificate /etc/nginx/certs/origin.pem;
#     ssl_certificate_key /etc/nginx/certs/origin.key;
# }
```

**风险**: 
- 中间人攻击
- 凭证明文传输
- 会话劫持

**修复建议**:
1. 获取 SSL/TLS 证书 (Let's Encrypt 免费)
2. 取消注释 HTTPS 配置
3. 强制 HTTP 重定向到 HTTPS
4. 配置 HSTS 头

#### 2. 缺少安全响应头
**问题**: 未配置关键安全头

**风险**:
- XSS 攻击
- 点击劫持
- MIME 类型嗅探

**修复建议**: 在 Nginx 配置中添加
```nginx
add_header X-Frame-Options "SAMEORIGIN" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
add_header Content-Security-Policy "default-src 'self'" always;
add_header Referrer-Policy "strict-origin-when-cross-origin" always;
```

#### 3. 弱密码策略
**问题**: 密码最小长度仅 4 字符

**风险**: 容易被暴力破解

**修复建议**:
```rust
#[validate(length(
    min = 8,  // 提高到 8+
    max = 128,  // 增加最大长度
    message = "Password must be 8-128 characters"
))]
#[validate(custom(function = "validate_password_strength"))]
pub password: String,

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    // 检查: 大小写字母、数字、特殊字符
}
```

#### 4. JWT 密钥管理
**问题**: JWT_SECRET 可能使用弱密钥

**风险**: 令牌可能被伪造

**修复建议**:
1. 生成强随机密钥 (至少 256 位)
   ```bash
   openssl rand -base64 32
   ```
2. 使用密钥管理服务
3. 定期轮换密钥

#### 5. 缺少请求大小限制
**问题**: 未限制请求体大小

**风险**: DoS 攻击

**修复建议**:
```nginx
client_max_body_size 10M;
client_body_buffer_size 128k;
```

### 🟡 中优先级

#### 6. 缺少令牌刷新机制
**问题**: 无刷新令牌，长期令牌不安全

**修复建议**:
- 实现短期访问令牌 (15分钟) + 长期刷新令牌 (7天)
- 添加 `/api/auth/refresh` 端点

#### 7. 缺少审计日志
**问题**: 无法追踪敏感操作

**修复建议**:
- 记录所有管理员操作
- 记录登录/登出事件
- 记录权限变更

#### 8. 缺少 CSRF 保护
**问题**: 无 CSRF 令牌验证

**风险**: 跨站请求伪造

**修复建议**:
- 对状态变更操作添加 CSRF 令牌
- 使用 SameSite Cookie 属性

#### 9. 数据库连接池配置
**问题**: 固定 5 个连接可能不足

**修复建议**:
```rust
PgPoolOptions::new()
    .max_connections(20)  // 根据负载调整
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(3))
```

#### 10. 缺少输入清理
**问题**: 用户输入可能包含恶意 HTML/JS

**修复建议**:
- 使用 HTML 清理库 (如 ammonia)
- 对输出进行转义

### 🟢 低优先级

#### 11. 多因素认证 (2FA)
**建议**: 支持 TOTP/SMS 二次验证

#### 12. 账户锁定机制
**建议**: 失败登录次数过多后锁定账户

#### 13. 会话管理
**建议**: 实现会话撤销和设备管理

#### 14. 数据加密
**建议**: 对敏感字段进行数据库级加密

## 🛡️ 安全最佳实践

### 部署前检查清单

- [ ] 启用 HTTPS
- [ ] 配置安全响应头
- [ ] 更改所有默认密码和密钥
- [ ] 限制请求大小和速率
- [ ] 配置防火墙规则
- [ ] 启用数据库备份
- [ ] 配置日志收集和监控
- [ ] 进行安全扫描和渗透测试
- [ ] 审查并更新依赖包
- [ ] 配置错误页面 (避免泄露信息)

### 运维安全建议

1. **定期更新依赖**
   ```bash
   cd backend
   cargo update
   cargo audit
   ```

2. **监控日志**
   - 异常登录尝试
   - 异常 API 调用
   - 错误率突增

3. **数据库安全**
   - 使用强密码
   - 限制网络访问
   - 定期备份
   - 加密连接

4. **容器安全**
   - 使用非 root 用户运行
   - 定期更新基础镜像
   - 扫描镜像漏洞

## 🚨 漏洞报告

如果您发现安全漏洞，请**不要**创建公开 Issue。请通过以下方式私密报告：

1. 发送邮件至: [待添加安全联系邮箱]
2. 包含详细的漏洞描述和复现步骤
3. 如有可能，提供修复建议

我们承诺在 48 小时内响应安全报告。

## 📚 安全参考资料

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [CWE Top 25](https://cwe.mitre.org/top25/)
- [Axum Security Best Practices](https://docs.rs/axum/latest/axum/)

## 📝 变更日志

- 2024-12-19: 初始安全文档创建
- [待更新]

---

**免责声明**: 本项目目前处于开发阶段。在生产环境部署前，请务必完成本文档中提到的所有高优先级安全改进。
