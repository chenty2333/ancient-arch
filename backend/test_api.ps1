# test_api.ps1

$BaseUrl = "http://localhost:3000/api"

function Test-Step {
    param($Name, $Command)
    Write-Host "[$Name]..." -NoNewline
    try {
        $Result = Invoke-Expression $Command
        Write-Host " OK" -ForegroundColor Green
        return $Result
    } catch {
        Write-Host " FAILED" -ForegroundColor Red
        Write-Host $_
        return $null
    }
}

# 1. 注册 Admin
# 注意：如果已注册过会报错 409，这里忽略错误继续
Write-Host "--- 1. Registering Users ---"
try {
    Invoke-RestMethod -Method Post -Uri "$BaseUrl/auth/register" -ContentType "application/json" -Body '{"username": "admin_script", "password": "password"}'
} catch {}
try {
    Invoke-RestMethod -Method Post -Uri "$BaseUrl/auth/register" -ContentType "application/json" -Body '{"username": "user_script", "password": "password"}'
} catch {}

# 2. 数据库提权
Write-Host "--- 2. Hacking Admin Role ---"
sqlite3 data.db "UPDATE users SET role='admin' WHERE username='admin_script';"

# 3. 登录获取 Token
Write-Host "--- 3. Logging in ---"
$AdminLogin = Invoke-RestMethod -Method Post -Uri "$BaseUrl/auth/login" -ContentType "application/json" -Body '{"username": "admin_script", "password": "password"}'
$AdminToken = $AdminLogin.token
Write-Host "Admin Token acquired." -ForegroundColor Cyan

$UserLogin = Invoke-RestMethod -Method Post -Uri "$BaseUrl/auth/login" -ContentType "application/json" -Body '{"username": "user_script", "password": "password"}'
$UserToken = $UserLogin.token
Write-Host "User Token acquired." -ForegroundColor Cyan

# 4. 测试 Admin 接口
Write-Host "--- 4. Testing Admin API (List Users) ---"
try {
    $Users = Invoke-RestMethod -Method Get -Uri "$BaseUrl/admin/users" -Headers @{Authorization = "Bearer $AdminToken"}
    Write-Host "Success! Found $($Users.Count) users." -ForegroundColor Green
    $Users | Select-Object id, username, role | Format-Table
} catch {
    Write-Host "Admin Access Failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "--- 5. Testing 403 Forbidden (User accessing Admin) ---"
try {
    Invoke-RestMethod -Method Get -Uri "$BaseUrl/admin/users" -Headers @{Authorization = "Bearer $UserToken"}
    Write-Host "FAILED! User should NOT access admin API." -ForegroundColor Red
} catch {
    if ($_.Exception.Response.StatusCode -eq [System.Net.HttpStatusCode]::Forbidden) {
        Write-Host "Success! Access Forbidden as expected." -ForegroundColor Green
    } else {
        Write-Host "Unexpected Error: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# 5. 测试 Quiz
Write-Host "--- 6. Testing Quiz Generate ---"
$Paper = Invoke-RestMethod -Method Get -Uri "$BaseUrl/quiz/generate"
Write-Host "Generated $($Paper.Count) questions." -ForegroundColor Green

# 提取第一题 ID
$Q1 = $Paper[0]
$Q1_ID = $Q1.id
Write-Host "Answering QID: $Q1_ID"

Write-Host "--- 7. Testing Quiz Submit ---"
# 构造答案 JSON (手动构造字符串以避免 PowerShell 对象转换问题)
$SubmitBody = '{"answers": {"' + $Q1_ID + '": "Wrong Answer"}}'

Write-Host "Sending Body: $SubmitBody"

try {
    $Score = Invoke-RestMethod -Method Post -Uri "$BaseUrl/quiz/submit" `
        -ContentType "application/json" `
        -Headers @{Authorization = "Bearer $UserToken"} `
        -Body $SubmitBody
    
    Write-Host "Submitted! Score: $($Score.score)" -ForegroundColor Green
} catch {
    Write-Host "Submit Failed: $($_.Exception.Message)" -ForegroundColor Red
    # 打印详细错误响应
    $Stream = $_.Exception.Response.GetResponseStream()
    $Reader = New-Object System.IO.StreamReader($Stream)
    Write-Host "Server Response: $($Reader.ReadToEnd())" -ForegroundColor Red
}

Write-Host "--- 8. Testing Leaderboard ---"
$Board = Invoke-RestMethod -Method Get -Uri "$BaseUrl/quiz/leaderboard"
$Board | Format-Table username, score, created_at

Write-Host "=== TEST COMPLETED ==="