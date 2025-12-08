-- 创建 users 表
-- id: 主键，自增
-- username: 用户名，必须唯一，用于登录
-- password: 存储 Argon2 哈希后的密码 (不是明文!)
-- role: 权限控制，默认是普通用户 'user'，管理员为 'admin'
-- created_at: 记录注册时间，默认为当前时间

CREATE TABLE IF NOT EXISTS users (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    username    TEXT NOT NULL UNIQUE,
    password    TEXT NOT NULL,
    role        TEXT NOT NULL DEFAULT 'user',
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);