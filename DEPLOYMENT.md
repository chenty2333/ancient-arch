# 部署指南 (Deployment Guide)

本文档详细说明如何在不同环境中部署 Ancient Arch 应用。

## 📋 目录

- [环境要求](#环境要求)
- [开发环境部署](#开发环境部署)
- [生产环境部署](#生产环境部署)
- [配置说明](#配置说明)
- [常见问题](#常见问题)

## 🔧 环境要求

### 最小要求
- **CPU**: 2 核心
- **内存**: 2 GB RAM
- **存储**: 10 GB 可用空间
- **操作系统**: Linux (推荐 Ubuntu 20.04+), macOS, Windows (WSL2)

### 推荐配置
- **CPU**: 4 核心
- **内存**: 4 GB RAM
- **存储**: 20 GB SSD
- **操作系统**: Ubuntu 22.04 LTS

### 软件要求
- Docker 20.10+
- Docker Compose 2.0+
- Git 2.30+
- (可选) Rust 1.75+

## 🚀 开发环境部署

### 使用 Docker Compose (推荐)

1. **克隆仓库**
```bash
git clone https://github.com/chenty2333/ancient-arch.git
cd ancient-arch
```

2. **配置环境变量**
```bash
cp .env.example .env
```

编辑 `.env` 文件：
```bash
# Database Configuration
POSTGRES_USER=devuser
POSTGRES_PASSWORD=devpass123
POSTGRES_DB=ancient_arch_dev

# Application Configuration
DATABASE_URL=postgres://devuser:devpass123@db:5432/ancient_arch_dev
JWT_SECRET=your_dev_jwt_secret_at_least_32_chars
JWT_EXPIRATION=3600
RUST_LOG=debug

# Admin User
ADMIN_USERNAME=admin
ADMIN_PASSWORD=admin123
```

3. **启动服务**
```bash
docker-compose up -d
```

4. **查看日志**
```bash
docker-compose logs -f app
```

5. **访问应用**
- API: http://localhost:8080/api
- Swagger 文档: http://localhost:8080/api/doc

6. **停止服务**
```bash
docker-compose down
```

7. **清理数据**
```bash
docker-compose down -v  # 删除数据卷
```

### 本地 Rust 开发

1. **安装 Rust**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **启动 PostgreSQL**
```bash
docker run -d \
  --name ancient_arch_postgres \
  -e POSTGRES_USER=devuser \
  -e POSTGRES_PASSWORD=devpass123 \
  -e POSTGRES_DB=ancient_arch_dev \
  -p 5432:5432 \
  postgres:16-alpine
```

3. **配置环境变量**
```bash
export DATABASE_URL="postgres://devuser:devpass123@localhost:5432/ancient_arch_dev"
export JWT_SECRET="your_dev_jwt_secret"
export RUST_LOG=debug
```

4. **安装 SQLx CLI**
```bash
cargo install sqlx-cli --no-default-features --features postgres
```

5. **运行迁移**
```bash
cd backend
sqlx migrate run
```

6. **启动应用**
```bash
cargo run
```

## 🏭 生产环境部署

### 前置准备

1. **服务器准备**
   - 确保服务器有固定 IP 或域名
   - 配置 DNS 记录指向服务器
   - 开放端口: 80 (HTTP), 443 (HTTPS)

2. **安装 Docker**
```bash
# Ubuntu/Debian
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER
```

3. **安装 Docker Compose**
```bash
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

### SSL/TLS 证书配置

#### 使用 Let's Encrypt (免费)

1. **安装 Certbot**
```bash
sudo apt update
sudo apt install certbot
```

2. **获取证书**
```bash
sudo certbot certonly --standalone -d yourdomain.com
```

3. **复制证书**
```bash
sudo mkdir -p nginx/certs
sudo cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem nginx/certs/origin.pem
sudo cp /etc/letsencrypt/live/yourdomain.com/privkey.pem nginx/certs/origin.key
sudo chown -R $USER:$USER nginx/certs
```

#### 使用自签名证书 (测试)

```bash
mkdir -p nginx/certs
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout nginx/certs/origin.key \
  -out nginx/certs/origin.pem \
  -subj "/CN=localhost"
```

### 生产配置

1. **创建生产环境变量**
```bash
cp .env.example .env.production
```

编辑 `.env.production`：
```bash
# 使用强密码和长密钥！
POSTGRES_USER=produser
POSTGRES_PASSWORD=$(openssl rand -base64 32)
POSTGRES_DB=ancient_arch

DATABASE_URL=postgres://produser:${POSTGRES_PASSWORD}@db:5432/ancient_arch
JWT_SECRET=$(openssl rand -base64 64)
JWT_EXPIRATION=3600
RUST_LOG=info

# 更改默认管理员密码
ADMIN_USERNAME=admin
ADMIN_PASSWORD=$(openssl rand -base64 16)
```

2. **启用 HTTPS**

编辑 `nginx/default.conf`，取消注释 HTTPS 部分：
```nginx
server {
    listen 443 ssl;
    listen [::]:443 ssl;
    server_name yourdomain.com;
    
    ssl_certificate /etc/nginx/certs/origin.pem;
    ssl_certificate_key /etc/nginx/certs/origin.key;
    
    # SSL 安全配置
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers on;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    
    # 安全头
    add_header Strict-Transport-Security "max-age=31536000" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    
    location /api {
        limit_req zone=api_limit burst=20 nodelay;
        proxy_pass http://app:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

# HTTP -> HTTPS 重定向
server {
    listen 80;
    server_name yourdomain.com;
    return 301 https://$host$request_uri;
}
```

3. **启动生产服务**
```bash
# 使用生产配置
docker-compose --env-file .env.production up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f
```

### 数据库备份

1. **手动备份**
```bash
# 备份数据库
docker exec ancient_arch_db pg_dump -U produser ancient_arch > backup_$(date +%Y%m%d_%H%M%S).sql

# 恢复数据库
docker exec -i ancient_arch_db psql -U produser ancient_arch < backup_20240101_120000.sql
```

2. **自动备份 (Cron)**
```bash
# 创建备份脚本
cat > /opt/ancient-arch/backup.sh << 'EOF'
#!/bin/bash
BACKUP_DIR=/opt/ancient-arch/backups
mkdir -p $BACKUP_DIR
docker exec ancient_arch_db pg_dump -U produser ancient_arch | gzip > $BACKUP_DIR/backup_$(date +%Y%m%d_%H%M%S).sql.gz
# 保留最近 7 天的备份
find $BACKUP_DIR -name "backup_*.sql.gz" -mtime +7 -delete
EOF

chmod +x /opt/ancient-arch/backup.sh

# 添加到 crontab (每天凌晨 2 点)
(crontab -l 2>/dev/null; echo "0 2 * * * /opt/ancient-arch/backup.sh") | crontab -
```

## ⚙️ 配置说明

### 环境变量详解

| 变量 | 描述 | 默认值 | 必需 |
|------|------|--------|------|
| `POSTGRES_USER` | PostgreSQL 用户名 | user | 是 |
| `POSTGRES_PASSWORD` | PostgreSQL 密码 | password | 是 |
| `POSTGRES_DB` | 数据库名称 | ancient_arch | 是 |
| `DATABASE_URL` | 完整数据库连接字符串 | - | 是 |
| `JWT_SECRET` | JWT 签名密钥 (至少 32 字符) | - | 是 |
| `JWT_EXPIRATION` | JWT 有效期 (秒) | 3600 | 否 |
| `RUST_LOG` | 日志级别 | info | 否 |
| `ADMIN_USERNAME` | 初始管理员用户名 | admin | 否 |
| `ADMIN_PASSWORD` | 初始管理员密码 | - | 否 |

### Nginx 配置调优

编辑 `nginx/default.conf`:

```nginx
# 工作进程数 (等于 CPU 核心数)
worker_processes auto;

# 连接数
events {
    worker_connections 1024;
}

# 速率限制
limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
limit_req_zone $binary_remote_addr zone=auth_limit:10m rate=5r/m;

# 请求大小限制
client_max_body_size 10M;
client_body_buffer_size 128k;

# 超时设置
proxy_connect_timeout 60s;
proxy_send_timeout 60s;
proxy_read_timeout 60s;
```

## 🔍 监控和维护

### 健康检查

```bash
# 检查服务状态
docker-compose ps

# 检查数据库连接
docker exec ancient_arch_db pg_isready -U produser

# 检查 API 健康
curl http://localhost:8080/api/architectures
```

### 日志管理

```bash
# 查看应用日志
docker-compose logs -f app

# 查看数据库日志
docker-compose logs -f db

# 查看 Nginx 日志
docker-compose logs -f nginx

# 清理旧日志
docker-compose exec app sh -c "find logs/ -mtime +30 -delete"
```

### 更新部署

```bash
# 拉取最新代码
git pull origin main

# 重新构建并启动
docker-compose up -d --build

# 查看变更
docker-compose logs -f app
```

## ❓ 常见问题

### Q: 数据库连接失败
**A**: 检查 `DATABASE_URL` 配置，确保数据库容器已启动且健康。

### Q: 内存不足
**A**: 调整 Docker 资源限制或增加服务器内存。

### Q: 端口冲突
**A**: 修改 `docker-compose.yml` 中的端口映射。

### Q: SSL 证书过期
**A**: 使用 Certbot 自动续期：
```bash
sudo certbot renew
```

### Q: 性能问题
**A**: 
1. 增加数据库连接池大小
2. 启用 Nginx 缓存
3. 添加 CDN
4. 垂直/水平扩展

## 📞 支持

如有问题，请：
1. 查看 [常见问题](#常见问题)
2. 检查日志文件
3. 创建 GitHub Issue

---

**注意**: 生产部署前，请务必阅读 [SECURITY.md](SECURITY.md) 并完成所有安全检查。
