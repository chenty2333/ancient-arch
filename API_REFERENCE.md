# Ancient Arch - API 参考文档

本文档详细描述了 Ancient Arch 后端提供的所有 API 接口、参数格式及响应结构。

## 1. 认证机制 (Authentication)

除 `/api/auth/login`, `/api/auth/register` 和部分公开读取接口（如查看帖子列表、查看建筑）外，大多数接口需要身份验证。

*   **Header**: `Authorization`
*   **Format**: `Bearer <your_jwt_token>`

---

## 3. 全局错误处理 (Global Error Handling)

所有 API 在发生错误时（如 400, 401, 404, 500），都会返回如下 JSON 格式：

```json
{
  "error": "Username 'myuser' already exists"
}
```

*   前端应统一判断 HTTP Status Code，并读取 `error` 字段进行提示。

---

## 2. API 详解

### 2.1 用户认证 (Auth)

#### 注册 (Register)
*   **URL**: `POST /api/auth/register`
*   **Body**:
    ```json
    {
      "username": "myuser",
      "password": "mypassword"
    }
    ```
    *   `username`: 3-50 字符。
    *   `password`: 4-128 字符。
*   **Response (201 Created)**:
    ```json
    {
      "id": 1,
      "username": "myuser",
      "role": "user",
      "is_verified": false,
      "created_at": "2025-12-20T10:00:00Z"
    }
    ```

#### 登录 (Login)
*   **URL**: `POST /api/auth/login`
*   **Body**:
    ```json
    {
      "username": "myuser",
      "password": "mypassword"
    }
    ```
    *   `username`: 1-50 字符。
    *   `password`: 1-128 字符。
*   **Response (200 OK)**:
    ```json
    {
      "token": "eyJhbGciOiJIUzI1Ni...",
      "type": "Bearer",
      "is_verified": false
    }
    ```

#### 生成资格考试 (Generate Qualification Exam)
*   **URL**: `GET /api/auth/qualification`
*   **Auth**: Required
*   **Response (200 OK)**:
    ```json
    {
      "questions": [
        {
          "id": 101,
          "type": "single", // 注意：输出时字段名为 "type"
          "content": "Which dynasty built the Forbidden City?",
          "options": ["Tang", "Ming", "Han", "Song"]
        }
      ],
      "exam_token": "eyJ...",  // 提交时必须携带此 Token
      "expires_in": 900
    }
    ```

#### 提交资格考试 (Submit Qualification Exam)
*   **URL**: `POST /api/auth/qualification/submit`
*   **Auth**: Required
*   **Body**:
    ```json
    {
      "exam_token": "<received_token>",
      "answers": {
        "101": "Ming",
        "102": "Option A"
      }
    }
    ```
*   **Response (200 OK)**:
    ```json
    {
      "score": 85.0,
      "correct_count": 17,
      "total_questions": 20,
      "passed": true,
      "message": "Verification successful!"
    }
    ```

---

### 2.2 古建筑知识库 (Architecture)

#### 获取建筑列表
*   **URL**: `GET /api/architectures`
*   **Query Params**:
    *   `category`: (Optional) 按分类筛选，例如 `?category=Palace`。
*   **Response (200 OK)**:
    ```json
    [
      {
        "id": 1,
        "category": "Palace",
        "name": "Forbidden City",
        "dynasty": "Ming",
        "location": "Beijing",
        "description": "...",
        "cover_img": "http://...",
        "carousel_imgs": ["http://...", "http://..."]
      }
    ]
    ```

#### 获取建筑详情
*   **URL**: `GET /api/architectures/{id}`
*   **Response (200 OK)**: 同上（单对象）。

---

### 2.3 社区论坛 (Community)

#### 获取帖子列表
*   **URL**: `GET /api/posts`
*   **Query Params**:
    *   `cursor`: (Optional) ISO Timestamp (上一次请求的最后一条记录的时间)，用于分页。
    *   `limit`: (Optional) 默认 20，最大 100。
    *   `sort`: (Optional) `new` (默认) 或 `hot`。
*   **Response (200 OK)**:
    ```json
    [
      {
        "id": 5,
        "user_id": 2,
        "title": "Discussion about Tang Roofs",
        "content": "...",
        "created_at": "...",
        "likes_count": 10,
        "comments_count": 5,
        "favorites_count": 2,
        "is_liked": false,      // 列表接口默认 false，仅详情接口会计算
        "is_favorited": false
      }
    ]
    ```

#### 创建帖子 (Verified User Only)
*   **URL**: `POST /api/posts`
*   **Auth**: Required + User MUST be Verified
*   **Body**:
    ```json
    {
      "title": "My New Discovery",
      "content": "Check this out..."
    }
    ```
    *   `title`: 1-100 字符。
    *   `content`: 1-10,000 字符。
*   **Response (201 Created)**:
    ```json
    { "id": 6 }
    ```

#### 获取帖子详情
*   **URL**: `GET /api/posts/{id}`
*   **Auth**: Optional (如果不传 Token，`is_liked` 为 false)
*   **Response (200 OK)**:
    ```json
    {
      "id": 5,
      // ... same as list item ...
      "is_liked": true,
      "is_favorited": false
    }
    ```

#### 删除帖子 (Author or Admin)
*   **URL**: `DELETE /api/posts/{id}`
*   **Auth**: Required
*   **Response**: 204 No Content

#### 点赞/取消点赞
*   **URL**: `POST /api/posts/{id}/like`
*   **Auth**: Required
*   **Response (200 OK)**:
    ```json
    { "liked": true } // true 表示现在是点赞状态，false 表示已取消
    ```

#### 收藏/取消收藏
*   **URL**: `POST /api/posts/{id}/favorite`
*   **Auth**: Required
*   **Response (200 OK)**:
    ```json
    { "favorited": true }
    ```

#### 获取评论列表
*   **URL**: `GET /api/posts/{id}/comments`
*   **Query Params**:
    *   `limit`: (Optional) 默认 50，最大 100。
    *   `offset`: (Optional) 默认 0。
*   **Response (200 OK)**:
    ```json
    [
      {
        "id": 10,
        "post_id": 5,
        "user_id": 3,
        "username": "commenter_one",
        "content": "Great post!",
        "root_id": null,      // 顶级评论
        "parent_id": null,
        "created_at": "..."
      },
      {
        "id": 11,
        "post_id": 5,
        "user_id": 2,
        "username": "author",
        "content": "Thanks!",
        "root_id": 10,        // 属于 ID 10 的子评论树
        "parent_id": 10       // 直接回复 ID 10
      }
    ]
    ```

#### 发表评论
*   **URL**: `POST /api/posts/{id}/comments`
*   **Auth**: Required
*   **Body**:
    ```json
    {
      "content": "I agree.",
      "parent_id": 10 // Optional. 如果回复某人，填其 Comment ID
    }
    ```
    *   `content`: 1-1,000 字符。
*   **Response (201 Created)**:
    ```json
    { "id": 12 }
    ```

---

### 2.4 个人资料 (Profile)

#### 获取我的信息
*   **URL**: `GET /api/profile/me`
*   **Auth**: Required
*   **Response (200 OK)**:
    ```json
    {
      "id": 1,
      "username": "myuser",
      "role": "user",
      "is_verified": true,
      "posts_count": 5,
      "total_likes_received": 20
    }
    ```

#### 获取我的帖子
*   **URL**: `GET /api/profile/posts`
*   **Response (200 OK)**: `[Post Objects]`

#### 获取我的收藏
*   **URL**: `GET /api/profile/favorites`
*   **Response (200 OK)**:
    ```json
    [
      {
        "post_id": 5,
        "title": "Post Title",
        "author_username": "other_user",
        "favorited_at": "..."
      }
    ]
    ```

#### 获取我的贡献记录
*   **URL**: `GET /api/profile/contributions`
*   **Response (200 OK)**:
    ```json
    [
      {
        "id": 1,
        "type": "architecture",
        "status": "pending",
        "created_at": "...",
        "data": { ... }
      }
    ]
    ```

---

### 2.5 内容贡献 (Contribution)

#### 提交新内容
*   **URL**: `POST /api/contributions`
*   **Auth**: Verified Users Only
*   **Body**:
    ```json
    {
      "type": "architecture", // 或 "question"
      "data": {
        // 如果是 architecture:
        "category": "Temple",
        "name": "White Horse Temple",
        "dynasty": "Han",
        "location": "Luoyang",
        "description": "First Buddhist temple in China",
        "cover_img": "http://...",
        "carousel_imgs": []
        // 如果是 question:
        // "question_type": "single", // 注意：提交/创建时字段名必须为 "question_type"
        // "content": "...",
        // "options": [...],
        // "answer": "...",
        // "analysis": "..."
      }
    }
    ```
    *   `data`: JSON 对象，总大小限制约为 50KB。
*   **Note**: 每日限提交 1 次。

---

### 2.6 趣味测验 (Quiz)

#### 生成练习卷
*   **URL**: `GET /api/quiz/generate`
*   **Response**: 同 `GET /api/auth/qualification`。

#### 提交练习卷
*   **URL**: `POST /api/quiz/submit`
*   **Body**: 同 `POST /api/auth/qualification/submit`。

#### 排行榜
*   **URL**: `GET /api/quiz/leaderboard`
*   **Response (200 OK)**:
    ```json
    [
      {
        "username": "top_player",
        "score": 100,
        "created_at": "..."
      }
    ]
    ```

---

### 2.7 管理员 (Admin)

所有 Admin 接口需要 Header `Authorization` 且用户 `role="admin"`。

#### 用户管理 (Users)
*   **List**: `GET /api/admin/users`
*   **Create**: `POST /api/admin/users`
    *   **Body**: `{"username": "...", "password": "...", "role": "admin"}`
    *   `username`: 3-50 字符。
    *   `password`: 4-128 字符。
*   **Update**: `PUT /api/admin/users/{id}`
    *   **Body** (所有字段可选): `{"username": "newname", "role": "user", "password": "newpass"}`
    *   验证规则同上。
*   **Delete**: `DELETE /api/admin/users/{id}`

#### 建筑管理 (Architectures)
*   **Create**: `POST /api/admin/architectures`
    *   **Body**:
        ```json
        {
          "category": "...", "name": "...", "dynasty": "...", 
          "location": "...", "description": "...", 
          "cover_img": "...", "carousel_imgs": ["..."]
        }
        ```
    *   `category`: 1-50 | `name`: 1-100 | `dynasty`: 1-50 | `location`: 1-200
    *   `description`: 1-20,000 | `cover_img`: 1-500 | `carousel_imgs`: 每个 URL 1-500
*   **Update**: `PUT /api/admin/architectures/{id}`
    *   **Body**: 同上，所有字段均为 Option。
*   **Delete**: `DELETE /api/admin/architectures/{id}`

#### 题库管理 (Questions)
*   **Create**: `POST /api/admin/questions`
    *   **Body**:
        ```json
        {
          "question_type": "single", // 注意：字段名为 "question_type"
          "content": "What is...?",
          "options": ["A", "B"],
          "answer": "A",
          "analysis": "Because..."
        }
        ```
    *   `question_type`: 1-20 | `content`: 1-1,000 | `options`: 每个 1-500
    *   `answer`: 1-500 | `analysis`: 0-2,000
*   **Update**: `PUT /api/admin/questions/{id}`
    *   **Body**: 同上，所有字段均为 Option。
*   **Delete**: `DELETE /api/admin/questions/{id}`

#### 贡献审核 (Contributions)
*   **List Pending**: `GET /api/admin/contributions`
*   **Review**: `PUT /api/admin/contributions/{id}/review`
    *   **Body**:
        ```json
        {
          "status": "approved", // 或 "rejected"
          "admin_comment": "Good job."
        }
        ```