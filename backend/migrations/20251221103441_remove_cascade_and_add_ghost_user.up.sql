-- 1. 插入幽灵用户 (如果不存在)
-- 密码设置为无效字符串，确保无法登录
INSERT INTO users (username, password, role, is_verified)
VALUES ('ghost', 'ACCOUNT_DISABLED_NO_LOGIN_PERMITTED', 'user', true)
ON CONFLICT (username) DO NOTHING;

-- 2. 移除 posts 表的级联删除
ALTER TABLE posts DROP CONSTRAINT IF EXISTS posts_user_id_fkey;
ALTER TABLE posts ADD CONSTRAINT posts_user_id_fkey 
    FOREIGN KEY (user_id) REFERENCES users(id); -- 默认是 RESTRICT (禁止删除还有帖子的用户)

-- 3. 移除 comments 表的级联删除 (如果有的话)
-- 之前的 interaction 迁移里，comments 也是 CASCADE，我们要改掉
ALTER TABLE comments DROP CONSTRAINT IF EXISTS comments_user_id_fkey;
ALTER TABLE comments ADD CONSTRAINT comments_user_id_fkey 
    FOREIGN KEY (user_id) REFERENCES users(id);
