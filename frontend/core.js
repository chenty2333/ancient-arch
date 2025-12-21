/**
 * api.js - 核心逻辑与 API 封装
 */

const API_BASE = window.location.origin + "/api";

const state = {
    get token() { return localStorage.getItem("aa_token"); },
    set token(val) { localStorage.setItem("aa_token", val); },
    user: null
};

// --- 状态栏逻辑 ---
const statusBar = {
    el: null,
    timer: null,
    init() {
        if (this.el) return;
        this.el = document.createElement("div");
        this.el.id = "status-bar";
        document.body.appendChild(this.el);
    },
    show(message, type = "info", duration = 3000) {
        this.init();
        this.el.textContent = message;
        this.el.className = `visible ${type}`;
        clearTimeout(this.timer);
        if (duration > 0) {
            this.timer = setTimeout(() => this.el.classList.remove("visible"), duration);
        }
    },
    hide() {
        if (this.el) this.el.classList.remove("visible");
    }
};

/**
 * Escapes HTML characters to prevent XSS attacks.
 * @param {string} text - The input string to escape.
 * @returns {string} - The escaped string.
 */
function escapeHtml(text) {
    if (text === null || text === undefined) return "";
    return String(text)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
}

// --- API 请求封装 ---
async function request(endpoint, options = {}) {
    // 确保 endpoint 以 / 开头
    const cleanEndpoint = endpoint.startsWith("/") ? endpoint : `/${endpoint}`;
    const url = `${API_BASE}${cleanEndpoint}`;
    
    console.log(`[API Request] ${options.method || "GET"} ${url}`);

    const headers = {
        "Content-Type": "application/json",
        ...(state.token ? { "Authorization": `Bearer ${state.token}` } : {})
    };

    try {
        const res = await fetch(url, { ...options, headers });
        
        // 处理 204 No Content
        if (res.status === 204) return null;

        const data = await res.json().catch(() => ({}));

        if (!res.ok) {
            if (res.status === 401) {
                // 如果在非登录页发生 401，清除 token 并跳转
                if (!window.location.pathname.includes("login.html")) {
                    state.token = "";
                    window.location.href = "login.html";
                }
            }
            throw new Error(data.error || `请求失败: ${res.status}`);
        }
        return data;
    } catch (err) {
        statusBar.show(err.message, "error");
        throw err;
    }
}

// --- 通用 UI 处理 ---
function updateNavbar() {
    const authLinks = document.querySelectorAll(".auth-only");
    const guestLinks = document.querySelectorAll(".guest-only");
    
    if (state.token) {
        authLinks.forEach(el => el.classList.remove("hidden"));
        guestLinks.forEach(el => el.classList.add("hidden"));
    } else {
        authLinks.forEach(el => el.classList.add("hidden"));
        guestLinks.forEach(el => el.classList.remove("hidden"));
    }
}

function logout() {
    state.token = "";
    window.location.href = "index.html";
}

// 页面加载时初始化
document.addEventListener("DOMContentLoaded", () => {
    statusBar.init();
    updateNavbar();
});
