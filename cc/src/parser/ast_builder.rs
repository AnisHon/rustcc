// use crate::parser::ast::{AssignOp, Expression, ExpressionKind, ExternalDeclaration, FloatSize, FunctionDefinition, UnaryOp};
// use crate::parser::ast::{ASTNode, IntegerSize, StorageClass, Type};
// use crate::parser::cst::{ArgumentExpressionList, AssignmentOperator, Constant, StorageClassSpecifier, StructOrUnion, TypeQualifier, TypeSpecifier, UnaryOperator};
// use crate::parser::cst_folder::CSTFolder;
// use crate::parser::span::Span;
// use crate::types::symbol_table::SymbolTable;
// use crate::types::token::Token;
// 
// pub struct ASTBuilder {
//     symbol_table: SymbolTable,
// }
// 
// impl CSTFolder for ASTBuilder {
//     type Result = ASTNode;
// 
//     fn fold_translation_unit_multi(&mut self, translation_unit: Self::Result, external_declaration: Self::Result) -> Self::Result {
//         let mut translation_unit = translation_unit.into_translation_unit().unwrap();
//         let external_decl = external_declaration.into_external_declaration().unwrap();
// 
//         let span = match &external_decl {
//             ExternalDeclaration::Function(_, span) => span,
//             ExternalDeclaration::Variable(_, span) => span
//         };
// 
// 
//         translation_unit.span.merge(span);
//         translation_unit.ext_decls.push(external_decl);
// 
//         Self::Result::TranslationUnit(translation_unit)
//     }
// 
//     fn fold_function_definition_with(&mut self, decl_spec: Self::Result, declarator: Self::Result, decl_list: Vec<Self::Result>, body: Self::Result) -> Self::Result {
//         // Self::Result::FunctionDefinition(FunctionDefinition {})
//         todo!()
//     }
// 
//     fn fold_function_definition_without(&mut self, declarator: Self::Result, decl_list: Vec<Self::Result>, body: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_declaration_node(&mut self, decl_specifiers: Self::Result, init_decl: Vec<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_init_declarator_node(&mut self, declarator: Self::Result, initializer: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_declaration_specifiers_storage_class(&mut self, storage_class_specifier: Self::Result, decl_specifiers: Option<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_declaration_specifiers_type_specifier(&mut self, type_specifier: Self::Result, decl_specifiers: Option<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_declaration_specifiers_type_qualifier(&mut self, type_qualifier: Self::Result, decl_specifiers: Option<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_storage_class_specifier(&mut self, node: StorageClassSpecifier) -> Self::Result {
//         let result = match node {
//             StorageClassSpecifier::Typedef(x) => StorageClass::Typedef(Span::from_token(&x)),
//             StorageClassSpecifier::Extern(x) => StorageClass::Extern(Span::from_token(&x)),
//             StorageClassSpecifier::Static(x) => StorageClass::Static(Span::from_token(&x)),
//             StorageClassSpecifier::Auto(x) => StorageClass::Auto(Span::from_token(&x)),
//             StorageClassSpecifier::Register(x) => StorageClass::Register(Span::from_token(&x))
//         };
// 
//         Self::Result::StorageClass(result)
//     }
// 
//     fn fold_type_specifier(&mut self, node: TypeSpecifier) -> Self::Result {
//         let result = match node {
//             TypeSpecifier::Void(x) => Type::Void(Span::from_token(&x)),
//             TypeSpecifier::Char(x) =>
//                 Type::Integer { signed: true, size: IntegerSize::Char, span: Span::from_token(&x) },
//             TypeSpecifier::Short(x) =>
//                 Type::Integer { signed: true, size: IntegerSize::Short, span: Span::from_token(&x) },
//             TypeSpecifier::Int(x) =>
//                 Type::Integer { signed: true, size: IntegerSize::Int, span: Span::from_token(&x) },
//             TypeSpecifier::Long(x) =>
//                 Type::Integer { signed: true, size: IntegerSize::Short, span: Span::from_token(&x) },
//             TypeSpecifier::Signed(x) =>
//                 Type::Integer { signed: true, size: IntegerSize::Int, span: Span::from_token(&x) },
//             TypeSpecifier::Unsigned(x) =>
//                 Type::Integer { signed: false, size: IntegerSize::Short, span: Span::from_token(&x) },
//             TypeSpecifier::Float(x) =>
//                 Type::Floating { size: FloatSize::Float, span: Span::from_token(&x) },
//             TypeSpecifier::Double(x) =>
//                 Type::Floating { size: FloatSize::Double, span: Span::from_token(&x) },
//             TypeSpecifier::StructOrUnion(x) =>
//                 return self.fold_struct_or_union_specifier(x),
//             TypeSpecifier::Enum(x) =>
//                 return self.fold_enum_specifier(x),
//             TypeSpecifier::TypeName(x) => {
//                 let span = Span::from_token(&x);
//                 let name = x.value.into_string().unwrap();
//                 Type::NamedType { name, span }
// 
//             }
//         };
//         Self::Result::Type(result)
//     }
// 
//     fn fold_type_qualifier(&mut self, node: TypeQualifier) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_struct_or_union_specifier_defined(&mut self, kind: Self::Result, name: Option<Token>, fields: Vec<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_struct_or_union_specifier_named(&mut self, kind: Self::Result, name: Token) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_struct_or_union(&mut self, node: StructOrUnion) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_struct_declaration_node(&mut self, specifiers: Self::Result, declarators: Vec<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_struct_declarator_node(&mut self, declarator: Option<Self::Result>, constexpr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_enum_specifier_node(&mut self, name: Option<Self::Result>, enumerators: Vec<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_enumerator_node(&mut self, token: Token, constexpr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_declarator_node(&mut self, pointer: Option<Self::Result>, direct: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_pointer_node(&mut self, type_quals: Vec<Self::Result>, pointer: Option<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_direct_declarator_array(&mut self, direct_declarator: Self::Result, constexpr: Option<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_direct_declarator_func_params(&mut self, direct_declarator: Self::Result, parameter_type_list: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_direct_declarator_func_idents(&mut self, direct_declarator: Self::Result, idents: Vec<Token>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_labeled_statement_label(&mut self, token: Token, stmt: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_labeled_statement_case(&mut self, constexpr: Self::Result, stmt: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_compound_statement_node(&mut self, block_items: Vec<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_selection_statement_if(&mut self, cond: Self::Result, then_stmt: Self::Result, else_stmt: Option<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_selection_statement_switch(&mut self, cond: Self::Result, body: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_iteration_statement_while(&mut self, cond: Self::Result, body: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_iteration_statement_do_while(&mut self, cond: Self::Result, body: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_iteration_statement_for(&mut self, init: Option<Self::Result>, cond: Option<Self::Result>, step: Option<Self::Result>, body: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_jump_statement_return(&mut self, expr: Option<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_primary_expressing_string_literal(&mut self, x: Vec<Token>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_constant(&mut self, node: Constant) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_postfix_expression_array(&mut self, post_expr: Self::Result, expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_postfix_expression_call(&mut self, post_expr: Self::Result, arg_expr_list: Option<ArgumentExpressionList>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_postfix_expression_field(&mut self, post_expr: Self::Result, token: Token) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_postfix_expression_arrow(&mut self, post_expr: Self::Result, token: Token) -> Self::Result {
//         let expr = post_expr.into_expression().unwrap();
//         let span = expr.span.merge(&Span::from_token(&token));
// 
//         let field = token.value.into_string().unwrap();
//         let kind = ExpressionKind::Arrow {base: Box::new(expr), field };
// 
//         // Self::Result::Expression(Expression {
//         //     kind,
//         //     span
//         // })
//         todo!()
//     }
// 
//     fn fold_unary_expression_unary_op(&mut self, unary_op: Self::Result, cast_expr: Self::Result) -> Self::Result {
//         let cast_expr = cast_expr.into_expression().unwrap();
//         let unary_op = unary_op.into_unary_op().unwrap();
//         let op_span = unary_op.unwrap_span();
// 
// 
//         let span = op_span.merge(&cast_expr.span);
// 
// 
//         let ty = match &unary_op {
//             UnaryOp::AddressOf(_) => Type::Pointer(Box::new(cast_expr.ty.clone()), span),
//             UnaryOp::Deref(_) => deref_type(cast_expr.ty.clone()),
//             UnaryOp::Plus(_) => promote_type(cast_expr.ty.clone()),
//             UnaryOp::Minus(_) => promote_type(cast_expr.ty.clone()),
//             UnaryOp::BitNot(_) => promote_type(cast_expr.ty.clone()),
//             UnaryOp::LogicalNot(_) => Type::Integer {signed: true, size: IntegerSize::Int, span}
//         };
// 
//         let kind = ExpressionKind::Unary {
//             op: unary_op,
//             expr: Box::new(cast_expr),
//         };
// 
// 
//         Self::Result::Expression(Expression {
//             kind,
//             ty,
//             span
//         })
//     }
// 
//     fn fold_unary_operator(&mut self, node: UnaryOperator) -> Self::Result {
//         let result = match node {
//             UnaryOperator::AddressOf(x) => UnaryOp::AddressOf(Span::from_token(&x)),
//             UnaryOperator::Deref(x) => UnaryOp::Deref(Span::from_token(&x)),
//             UnaryOperator::Plus(x) => UnaryOp::Plus(Span::from_token(&x)),
//             UnaryOperator::Minus(x) => UnaryOp::Minus(Span::from_token(&x)),
//             UnaryOperator::BitNot(x) => UnaryOp::BitNot(Span::from_token(&x)),
//             UnaryOperator::Not(x) => UnaryOp::LogicalNot(Span::from_token(&x)),
//         };
//         Self::Result::UnaryOp(result)
//     }
// 
//     fn fold_cast_expression_cast(&mut self, type_name: Self::Result, cast_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_multiplicative_expression_mul(&mut self, mul_expr: Self::Result, cast_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_multiplicative_expression_div(&mut self, mul_expr: Self::Result, cast_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_multiplicative_expression_mod(&mut self, mul_expr: Self::Result, cast_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_additive_expression_add(&mut self, add_expr: Self::Result, mul_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_additive_expression_sub(&mut self, add_expr: Self::Result, mul_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_shift_expression_shl(&mut self, shift_expr: Self::Result, add_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_shift_expression_shr(&mut self, shift_expr: Self::Result, add_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_relational_expression_lt(&mut self, relation_expr: Self::Result, shift_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_relational_expression_gt(&mut self, relation_expr: Self::Result, shift_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_relational_expression_le(&mut self, relation_expr: Self::Result, shift_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_relational_expression_ge(&mut self, relation_expr: Self::Result, shift_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_equality_expression_eq(&mut self, eq_expr: Self::Result, relation_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_equality_expression_ne(&mut self, eq_expr: Self::Result, relation_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_and_expression_and(&mut self, and_expr: Self::Result, eq_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_exclusive_or_expression_xor(&mut self, exclusive_or_expr: Self::Result, and_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_inclusive_or_expression_or(&mut self, inclusive_or_expr: Self::Result, exclusive_or_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_logical_and_expression_and(&mut self, logical_and_expr: Self::Result, inclusive_or_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_logical_or_expression_or(&mut self, logical_or_expr: Self::Result, logical_and_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_conditional_expression_cond(&mut self, cond_expr: Self::Result, then_expr: Self::Result, else_expr: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_assignment_expression_assign(&mut self, lhs_expr: Self::Result, op: Self::Result, rhs: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_assignment_operator(&mut self, node: AssignmentOperator) -> Self::Result {
//         let result = match node {
//             AssignmentOperator::Assign(x) => AssignOp::Assign(Span::from_token(&x)),
//             AssignmentOperator::MulAssign(x) => AssignOp::MulAssign(Span::from_token(&x)),
//             AssignmentOperator::DivAssign(x) => AssignOp::DivAssign(Span::from_token(&x)),
//             AssignmentOperator::ModAssign(x) => AssignOp::ModAssign(Span::from_token(&x)),
//             AssignmentOperator::AddAssign(x) => AssignOp::AddAssign(Span::from_token(&x)),
//             AssignmentOperator::SubAssign(x) => AssignOp::SubAssign(Span::from_token(&x)),
//             AssignmentOperator::ShlAssign(x) => AssignOp::ShlAssign(Span::from_token(&x)),
//             AssignmentOperator::ShrAssign(x) => AssignOp::ShrAssign(Span::from_token(&x)),
//             AssignmentOperator::AndAssign(x) => AssignOp::AndAssign(Span::from_token(&x)),
//             AssignmentOperator::XorAssign(x) => AssignOp::XorAssign(Span::from_token(&x)),
//             AssignmentOperator::OrAssign(x) => AssignOp::OrAssign(Span::from_token(&x)),
//         };
//         Self::Result::AssignOp(result)
//     }
// 
//     /// expression
//     ///     : assignment_expression
//     ///     | expression COMMA assignment_expression
//     fn fold_expression_node(&mut self, expr: Self::Result, assign_expr: Self::Result) -> Self::Result {
//         let mut expr = expr.into_expression().unwrap();
//         let assign_expr = assign_expr.into_expression().unwrap();
// 
//         let exprs = expr.kind.as_comma_mut().unwrap();
// 
//         expr.span.merge_self(&assign_expr.span);
//         exprs.push(assign_expr);
// 
//         Self::Result::Expression(expr)
//     }
// 
//     fn fold_type_name_node(&mut self, specifiers: Self::Result, abs_declarator: Option<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_initializer_list_node(&mut self, node: Vec<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_abstract_declarator_direct(&mut self, pointer: Option<Self::Result>, direct: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_direct_abstract_declarator_array(&mut self, constexpr: Option<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_direct_abstract_declarator_array_nested(&mut self, direct_abs_declarator: Self::Result, constexpr: Option<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_direct_abstract_declarator_func(&mut self, param_type_list: Option<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_direct_abstract_declarator_func_nested(&mut self, direct_abs_declarator: Self::Result, param_type_list: Option<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_parameter_type_list_node(&mut self, param_decl: Vec<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_parameter_declaration_with_declarator(&mut self, specs: Self::Result, declarator: Self::Result) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_parameter_declaration_abstract(&mut self, specs: Self::Result, abs_decl: Option<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_specifier_qualifier_list_type_spec(&mut self, type_spec: Self::Result, spec_qual_list: Option<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_specifier_qualifier_list_type_qual(&mut self, type_qual: Self::Result, spec_qual_list: Option<Self::Result>) -> Self::Result {
//         todo!()
//     }
// 
//     fn fold_token(&mut self, node: Token) -> Self::Result {
//         Self::Result::Token(node)
//     }
// }
// 
// 
