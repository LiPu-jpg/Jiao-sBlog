pub mod jwt;

// 重新导出，方便外部使用
pub use jwt::{AuthError, Claims, JWTConfig, create_token, validate_token};