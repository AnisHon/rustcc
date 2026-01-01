/// parser模块设计较为冗杂，总体是按照 C 的文法进行递归下降解析的，sema 与 parser 交织进行
/// parser_* 模块纯按照文法递归下降，与文法一一对应。
/// 该模块最终将输出一个 Semantic AST（已经进行 表达式折叠 符号分析 类型推导 ...）
/// # Contents
/// - `parser_core`: 虽然是 core 但是只是提供必要函数，因为 clippy 不推荐与模块同名
/// - `parser_decl`: 解析 declaration 相关，文法参考 resources/declaration.txt
/// - `parser_expr`: 解析 expression 相关，文法参考 resources/expression.txt
/// - `parser_function`: 解析 declaration 相关，文法参考 resources/function.txt
/// - `parser_stmt`: 解析 statement 相关，文法参考 resources/statement.txt
/// - `semantic`: 核心模块定义了 编译上下文 `CompCtx`,  类型上下文 `TypeCtx`, 作用域管理器 `ScopeMgr`
pub mod parser_core;
mod parser_decl;
mod parser_expr;
mod parser_extern;
mod parser_stmt;
mod semantic;

pub use crate::parser::semantic::{ast, common, comp_ctx};
