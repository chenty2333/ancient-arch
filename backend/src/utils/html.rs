use ammonia;

/// Clean HTML content using the ammonia library.
/// 
/// This employs a whitelist-based sanitization strategy: it preserves safe tags 
/// (like <b>, <p>) while stripping dangerous tags (like <script>, <iframe>) 
/// and malicious attributes (like onclick).
/// 
/// Note: 
/// 1. This will remove the <script> tag and its entire content.
/// 2. If the goal is to display raw code, the frontend should use `textContent` 
///    or the backend should use HTML entity escaping instead of sanitization.
/// 3. This serves as a fail-safe against Stored XSS in admin panels or other clients.
pub fn clean_html(input: &str) -> String {
    ammonia::clean(input)
}
