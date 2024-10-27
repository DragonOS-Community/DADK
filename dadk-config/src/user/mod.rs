#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserCleanLevel {
    /// 清理所有用户程序构建缓存
    All,
    /// 只在用户程序源码目录下清理
    InSrc,
    /// 只清理用户程序输出目录
    Output,
}
