-- 1. 创建触发器函数
CREATE OR REPLACE FUNCTION prevent_system_user_deletion()
RETURNS TRIGGER AS $$
BEGIN
    -- 检查被删除的用户名
    IF OLD.username IN ('admin', 'ghost') THEN
        RAISE EXCEPTION 'Operation Denied: Cannot delete system protected account (%)', OLD.username;
    END IF;
    -- 如果不是受保护用户，允许操作
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

-- 2. 将函数绑定到 users 表的 DELETE 操作之前
CREATE TRIGGER trigger_protect_system_users
BEFORE DELETE ON users
FOR EACH ROW
EXECUTE FUNCTION prevent_system_user_deletion();
