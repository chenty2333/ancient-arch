-- 移除旧的外键约束
ALTER TABLE posts DROP CONSTRAINT IF EXISTS posts_user_id_fkey;

-- 添加带有 ON DELETE CASCADE 的新约束
ALTER TABLE posts 
    ADD CONSTRAINT posts_user_id_fkey 
    FOREIGN KEY (user_id) 
    REFERENCES users(id) 
    ON DELETE CASCADE;
