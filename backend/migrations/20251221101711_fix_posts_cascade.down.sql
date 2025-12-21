-- 回滚操作：移除级联删除，恢复默认约束
ALTER TABLE posts DROP CONSTRAINT IF EXISTS posts_user_id_fkey;

ALTER TABLE posts 
    ADD CONSTRAINT posts_user_id_fkey 
    FOREIGN KEY (user_id) 
    REFERENCES users(id);
