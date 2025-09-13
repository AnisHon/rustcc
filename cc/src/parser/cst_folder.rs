
//!
//! 废弃，无用
//!
use crate::parser::cst::*;
use crate::types::token::Token;

/// 解包+Optional转换的声明宏
macro_rules! unwrap_list_opt {
    ($opt:expr) => {
        $opt.map(|x| x.0).unwrap_or_else(Vec::new)
    };
}


pub trait CSTFolder {

    type Result;

    fn fold_translation_unit(&mut self, node: TranslationUnit) -> Self::Result {
        match node {
            TranslationUnit::Single(ext_decl) => self.fold_external_declaration(*ext_decl),
            TranslationUnit::Multi(unit, ext_decl, ) => {
                let unit = self.fold_translation_unit(*unit);
                let ext_decl = self.fold_external_declaration(*ext_decl);
                self.fold_translation_unit_multi(unit, ext_decl)
            }
        }
    }

    fn fold_translation_unit_multi(&mut self, translation_unit: Self::Result, external_declaration: Self::Result) -> Self::Result;

    fn fold_external_declaration(&mut self, node: ExternalDeclaration) -> Self::Result {
        match node {
            ExternalDeclaration::FunctionDefinition(func_def) => self.fold_function_definition(func_def),
            ExternalDeclaration::Declaration(decl) => self.fold_declaration(decl),
        }
    }

    fn fold_function_definition(&mut self, node: FunctionDefinition) -> Self::Result {
        match node {
            FunctionDefinition::WithSpecifiers {
                declaration_specifiers,
                declarator,
                declaration_list,
                body,
            } => {

                let declaration_list = unwrap_list_opt!(declaration_list);

                let decl_spec = self.fold_declaration_specifiers(declaration_specifiers);
                let declarator = self.fold_declarator(declarator);
                let decl_list: Vec<_> = declaration_list.into_iter()
                    .map(|x| self.fold_declaration(x))
                    .collect();
                let body = self.fold_compound_statement(body);
                self.fold_function_definition_with(decl_spec, declarator, decl_list, body)
            },
            FunctionDefinition::WithoutSpecifiers {
                declarator,
                declaration_list,
                body,
            } => {
                let declaration_list = declaration_list.unwrap_or_else(DeclarationList::empty).0;

                let declarator = self.fold_declarator(declarator);
                let decl_list: Vec<_> = declaration_list.into_iter()
                    .map(|x| self.fold_declaration(x))
                    .collect();
                let body = self.fold_compound_statement(body);

                self.fold_function_definition_without(declarator, decl_list, body)
            }
        }
    }

    fn fold_function_definition_with(&mut self, decl_spec: Self::Result, declarator: Self::Result, decl_list: Vec<Self::Result>, body: Self::Result) -> Self::Result;

    fn fold_function_definition_without(&mut self, declarator: Self::Result, decl_list: Vec<Self::Result>, body: Self::Result) -> Self::Result;

    fn fold_declaration(&mut self, node: Declaration) -> Self::Result {
        let init_declarators = unwrap_list_opt!(node.init_declarators);

        let decl_specifiers = self.fold_declaration_specifiers(node.specifiers);
        let init_decl: Vec<_> = init_declarators.into_iter().map(|x| self.fold_init_declarator(x)).collect();
        self.fold_declaration_node(decl_specifiers, init_decl)
    }

    fn fold_declaration_node(&mut self, decl_specifiers: Self::Result, init_decl: Vec<Self::Result>) -> Self::Result;

    // fn fold_init_declarator_list(&mut self, node: InitDeclaratorList) -> Self::Result {
    //     let declarator_list: Vec<_> = node.0.into_iter().map(|x| self.fold_init_declarator(x)).collect();
    //     self.fold_init_declarator_list_node(declarator_list)
    // }
    //
    // fn fold_init_declarator_list_node(&mut self, declarator_list: Vec<Self::Result>) -> Self::Result;

    fn fold_init_declarator(&mut self, node: InitDeclarator) -> Self::Result {
        match node {
            InitDeclarator::Plain(declarator) => self.fold_declarator(declarator),
            InitDeclarator::Initialized(declarator, initializer) => {
                let declarator = self.fold_declarator(declarator);
                let initializer = self.fold_initializer(initializer);
                self.fold_init_declarator_node(declarator, initializer)
            },
        }
    }

    fn fold_init_declarator_node(&mut self, declarator: Self::Result, initializer: Self::Result) -> Self::Result;

    fn fold_declaration_specifiers(&mut self, node: DeclarationSpecifiers) -> Self::Result {
        match node {
            DeclarationSpecifiers::StorageClass(storage_class_specifier, decl_specifiers) => {
                let storage_class_specifier = self.fold_storage_class_specifier(storage_class_specifier);
                let decl_specifiers = decl_specifiers.map(|x| self.fold_declaration_specifiers(*x));
                self.fold_declaration_specifiers_storage_class(storage_class_specifier, decl_specifiers)
            }
            DeclarationSpecifiers::TypeSpecifier(type_specifier, decl_specifiers) => {
                let type_specifier = self.fold_type_specifier(type_specifier);
                let decl_specifiers = decl_specifiers.map(|x| self.fold_declaration_specifiers(*x));
                self.fold_declaration_specifiers_type_specifier(type_specifier, decl_specifiers)
            }
            DeclarationSpecifiers::TypeQualifier(type_qual, decl_specifiers) => {
                let type_qualifier = self.fold_type_qualifier(type_qual);
                let decl_specifiers = decl_specifiers.map(|x| self.fold_declaration_specifiers(*x));
                self.fold_declaration_specifiers_type_qualifier(type_qualifier, decl_specifiers)
            }
        }
    }

    fn fold_declaration_specifiers_storage_class(&mut self, storage_class_specifier: Self::Result, decl_specifiers: Option<Self::Result>) -> Self::Result;
    fn fold_declaration_specifiers_type_specifier(&mut self, type_specifier: Self::Result, decl_specifiers: Option<Self::Result>) -> Self::Result;
    fn fold_declaration_specifiers_type_qualifier(&mut self, type_qualifier: Self::Result, decl_specifiers: Option<Self::Result>) -> Self::Result;

    fn fold_storage_class_specifier(&mut self, node: StorageClassSpecifier) -> Self::Result;

    /// 虽然有非叶子节点，但是我选择不处理，因为太琐碎
    fn fold_type_specifier(&mut self, node: TypeSpecifier) -> Self::Result;

    fn fold_type_qualifier(&mut self, node: TypeQualifier) -> Self::Result;

    fn fold_struct_or_union_specifier(&mut self, node: StructOrUnionSpecifier) -> Self::Result {
        match node {
            StructOrUnionSpecifier::Defined { kind, name, fields } => {
                let kind = self.fold_struct_or_union(kind);
                let name = name as Option<Token>;
                let fields: Vec<_> = fields.into_iter().map(|x| self.fold_struct_declaration(x)).collect();
                self.fold_struct_or_union_specifier_defined(kind, name, fields)
            }
            StructOrUnionSpecifier::Named { kind, name } => {
                let kind = self.fold_struct_or_union(kind);
                self.fold_struct_or_union_specifier_named(kind, name)
            }
        }
    }
    fn fold_struct_or_union_specifier_defined(&mut self, kind: Self::Result, name: Option<Token>, fields: Vec<Self::Result>) -> Self::Result;

    fn fold_struct_or_union_specifier_named(&mut self, kind: Self::Result, name: Token) -> Self::Result;

    fn fold_struct_or_union(&mut self, node: StructOrUnion) -> Self::Result;

    fn fold_struct_declaration(&mut self, node: StructDeclaration) -> Self::Result {
        let specifiers = self.fold_specifier_qualifier_list(node.specifiers);
        let declarators: Vec<_> = node.declarators.into_iter().map(|x| self.fold_struct_declarator(x)).collect();
        self.fold_struct_declaration_node(specifiers, declarators)
    }

    fn fold_struct_declaration_node(&mut self, specifiers: Self::Result, declarators: Vec<Self::Result>) -> Self::Result;

    fn fold_struct_declarator(&mut self, node: StructDeclarator) -> Self::Result {
        match node {
            StructDeclarator::Declarator(declarator) => self.fold_declarator(declarator),
            StructDeclarator::Bitfield(declarator, constexpr) => {
                let declarator = declarator.map(|x| self.fold_declarator(x));
                let constexpr = self.fold_constant_expression(constexpr);
                self.fold_struct_declarator_node(declarator, constexpr)
            }
        }
    }

    fn fold_struct_declarator_node(&mut self, declarator: Option<Self::Result>, constexpr: Self::Result) -> Self::Result;

    fn fold_enum_specifier(&mut self, node: EnumSpecifier) -> Self::Result {
        match node {
            EnumSpecifier::Defined { name, enumerators } => {
                let name = name.map(|x| self.fold_token(x));
                let enumerators: Vec<_> = enumerators.into_iter().map(|x| self.fold_enumerator(x)).collect();
                self.fold_enum_specifier_node(name, enumerators)
            },
            EnumSpecifier::Named(name) => self.fold_token(name)
        }
    }

    fn fold_enum_specifier_node(&mut self, name: Option<Self::Result>, enumerators: Vec<Self::Result>) -> Self::Result;

    fn fold_enumerator(&mut self, node: Enumerator) -> Self::Result {
        match node {
            Enumerator::Plain(token) => self.fold_token(token),
            Enumerator::WithValue(token, constexpr) => {
                let constexpr = self.fold_constant_expression(constexpr);
                self.fold_enumerator_node(token, constexpr)
            }
        }
    }
    fn fold_enumerator_node(&mut self, token: Token, constexpr: Self::Result) -> Self::Result;

    fn fold_declarator(&mut self, node: Declarator) -> Self::Result {
        let pointer = node.pointer.map(|x| self.fold_pointer(x));
        let direct = self.fold_direct_declarator(node.direct);
        self.fold_declarator_node(pointer, direct)
    }

    fn fold_declarator_node(&mut self, pointer: Option<Self::Result>, direct: Self::Result) -> Self::Result;

    fn fold_pointer(&mut self, node: Pointer) -> Self::Result {
        match node { Pointer::Single(type_quals, pointer) => {
            let type_quals: Vec<_> = type_quals.into_iter().map(|x| self.fold_type_qualifier(x)).collect();
            let pointer = pointer.map(|x| self.fold_pointer(*x));
            self.fold_pointer_node(type_quals, pointer)
        } }
    }
    fn fold_pointer_node(&mut self, type_quals: Vec<Self::Result>, pointer: Option<Self::Result>) -> Self::Result;
    fn fold_direct_declarator(&mut self, node: DirectDeclarator) -> Self::Result {
        match node {
            DirectDeclarator::Id(token) => self.fold_token(token),
            DirectDeclarator::Paren(declarator) => self.fold_declarator(*declarator),
            DirectDeclarator::Array(direct_declarator, constexpr) => {
                let direct_declarator =self.fold_direct_declarator(*direct_declarator);
                let constexpr = constexpr.map(|x| self.fold_constant_expression(x));
                self.fold_direct_declarator_array(direct_declarator, constexpr)
            }
            DirectDeclarator::FuncParams(direct_declarator, parameter_type_list) => {
                let direct_declarator =self.fold_direct_declarator(*direct_declarator);
                let parameter_type_list = self.fold_parameter_type_list(parameter_type_list);
                self.fold_direct_declarator_func_params(direct_declarator, parameter_type_list)
            }
            DirectDeclarator::FuncIdentifiers(direct_declarator, idents) => {
                let direct_declarator =self.fold_direct_declarator(*direct_declarator);
                let idents = unwrap_list_opt!(idents);
                self.fold_direct_declarator_func_idents(direct_declarator, idents)
            }
        }
    }

    fn fold_direct_declarator_array(&mut self, direct_declarator: Self::Result, constexpr: Option<Self::Result>) -> Self::Result;
    fn fold_direct_declarator_func_params(&mut self, direct_declarator: Self::Result, parameter_type_list: Self::Result) -> Self::Result;
    fn fold_direct_declarator_func_idents(&mut self, direct_declarator: Self::Result, idents: Vec<Token>) -> Self::Result;

    fn fold_statement(&mut self, node: Statement) -> Self::Result {
        match node {
            Statement::Labeled(labeled_stmt) => self.fold_labeled_statement(labeled_stmt),
            Statement::Compound(compound_stmt) => self.fold_compound_statement(compound_stmt),
            Statement::Expression(expr_stmt) => self.fold_expression_statement(expr_stmt),
            Statement::Selection(selection_stmt) => self.fold_selection_statement(selection_stmt),
            Statement::Iteration(iter_stmt) => self.fold_iteration_statement(iter_stmt),
            Statement::Jump(jmp_stmt) => self.fold_jump_statement(jmp_stmt),
        }
    }

    fn fold_labeled_statement(&mut self, node: LabeledStatement) -> Self::Result {
        match node {
            LabeledStatement::Label(token, stmt) => {
                let stmt = self.fold_statement(*stmt);
                self.fold_labeled_statement_label(token, stmt)
            }
            LabeledStatement::Case(constexpr, stmt) => {
                let constexpr = self.fold_constant_expression(constexpr);
                let stmt = self.fold_statement(*stmt);
                self.fold_labeled_statement_case(constexpr, stmt)
            }
            LabeledStatement::Default(stmt) => self.fold_statement(*stmt)
        }
    }

    fn fold_labeled_statement_label(&mut self, token: Token, stmt: Self::Result) -> Self::Result;
    fn fold_labeled_statement_case(&mut self, constexpr: Self::Result, stmt: Self::Result) -> Self::Result;

    fn fold_compound_statement(&mut self, node: CompoundStatement) -> Self::Result {
        match node {
            CompoundStatement::Empty(x) => self.fold_token(x),
            CompoundStatement::Block(block_items) => {
                let block_items: Vec<_> = block_items.into_iter().map(|x| self.fold_block_item(x)).collect();
                self.fold_compound_statement_node(block_items)
            }
        }
    }

    fn fold_compound_statement_node(&mut self, block_items: Vec<Self::Result>) -> Self::Result;
    fn fold_block_item(&mut self, node: BlockItem) -> Self::Result {
        match node {
            BlockItem::Declaration(x) => self.fold_declaration(x),
            BlockItem::Statement(x) => self.fold_statement(x),
        }
    }

    fn fold_expression_statement(&mut self, node: ExpressionStatement) -> Self::Result {
        match node {
            ExpressionStatement::Empty(x) => self.fold_token(x),
            ExpressionStatement::Expr(x) => self.fold_expression(x),
        }
    }

    fn fold_selection_statement(&mut self, node: SelectionStatement) -> Self::Result {
        match node {
            SelectionStatement::If { cond, then_stmt, else_stmt } => {
                let cond = self.fold_expression(cond);
                let then_stmt = self.fold_statement(*then_stmt);
                let else_stmt = else_stmt.map(|x| self.fold_statement(*x));
                self.fold_selection_statement_if(cond, then_stmt, else_stmt)
            }
            SelectionStatement::Switch { cond, body } => {
                let cond = self.fold_expression(cond);
                let body = self.fold_statement(*body);
                self.fold_selection_statement_switch(cond, body)
            }
        }
    }

    fn fold_selection_statement_if(&mut self, cond: Self::Result, then_stmt: Self::Result, else_stmt: Option<Self::Result>) -> Self::Result;

    fn fold_selection_statement_switch(&mut self, cond: Self::Result, body: Self::Result) -> Self::Result;

    fn fold_iteration_statement(&mut self, node: IterationStatement) -> Self::Result {
        match node {
            IterationStatement::While { cond, body } => {
                let cond = self.fold_expression(cond);
                let body = self.fold_statement(*body);
                self.fold_iteration_statement_while(cond, body)
            }
            IterationStatement::DoWhile { body, cond } => {
                let cond = self.fold_expression(cond);
                let body = self.fold_statement(*body);
                self.fold_iteration_statement_do_while(cond, body)
            }
            IterationStatement::For { init, cond, step, body } => {
                let init = init.map(|x| self.fold_expression(x));
                let cond = cond.map(|x| self.fold_expression(x));
                let step = step.map(|x| self.fold_expression(x));
                let body = self.fold_statement(*body);
                self.fold_iteration_statement_for(init, cond, step, body)
            }
        }
    }

    fn fold_iteration_statement_while(&mut self, cond: Self::Result, body: Self::Result) -> Self::Result;

    fn fold_iteration_statement_do_while(&mut self, cond: Self::Result, body: Self::Result) -> Self::Result;

    fn fold_iteration_statement_for(&mut self, init: Option<Self::Result>, cond: Option<Self::Result>, step: Option<Self::Result>, body: Self::Result) -> Self::Result;

    fn fold_jump_statement(&mut self, node: JumpStatement) -> Self::Result {
        match node {
            JumpStatement::Goto(x) => self.fold_token(x),
            JumpStatement::Continue(x) => self.fold_token(x),
            JumpStatement::Break(x) => self.fold_token(x),
            JumpStatement::Return(expr) => {
                let expr = expr.map(|x| self.fold_expression(x));
                self.fold_jump_statement_return(expr)
            }
        }
    }

    fn fold_jump_statement_return(&mut self, expr: Option<Self::Result>) -> Self::Result;

    fn fold_primary_expression(&mut self, node: PrimaryExpression) -> Self::Result {
        match node {
            PrimaryExpression::Id(x) => self.fold_token(x),
            PrimaryExpression::Constant(x) => self.fold_constant(x),
            PrimaryExpression::StringLiteral(x) => self.fold_primary_expressing_string_literal(x),
            PrimaryExpression::Paren(x) => self.fold_expression(*x),
        }
    }

    fn fold_primary_expressing_string_literal(&mut self, x: Vec<Token>) -> Self::Result;

    fn fold_constant(&mut self, node: Constant) -> Self::Result;

    fn fold_postfix_expression(&mut self, node: PostfixExpression) -> Self::Result {
        match node {
            PostfixExpression::Primary(x) => self.fold_primary_expression(x),
            PostfixExpression::Array(post_expr, expr) => {
                let post_expr = self.fold_postfix_expression(*post_expr);
                let expr = self.fold_expression(*expr);
                self.fold_postfix_expression_array(post_expr, expr)
            }
            PostfixExpression::Call(post_expr, arg_expr_list) => {
                let post_expr = self.fold_postfix_expression(*post_expr);
                self.fold_postfix_expression_call(post_expr, arg_expr_list)
            }
            PostfixExpression::Field(post_expr, token) => {
                let post_expr = self.fold_postfix_expression(*post_expr);
                self.fold_postfix_expression_field(post_expr, token)
            }
            PostfixExpression::Arrow(post_expr, token) => {
                let post_expr = self.fold_postfix_expression(*post_expr);
                self.fold_postfix_expression_arrow(post_expr, token)
            }
            PostfixExpression::Inc(post_expr) => self.fold_postfix_expression(*post_expr),
            PostfixExpression::Dec(post_expr) => self.fold_postfix_expression(*post_expr),
        }
    }

    fn fold_postfix_expression_array(&mut self, post_expr: Self::Result, expr: Self::Result) -> Self::Result;
    fn fold_postfix_expression_call(&mut self, post_expr: Self::Result, arg_expr_list: Option<ArgumentExpressionList>) -> Self::Result;
    fn fold_postfix_expression_field(&mut self, post_expr: Self::Result, token: Token) -> Self::Result;
    fn fold_postfix_expression_arrow(&mut self, post_expr: Self::Result, token: Token) -> Self::Result;

    fn fold_unary_expression(&mut self, node: UnaryExpression) -> Self::Result {
        match node {
            UnaryExpression::Postfix(x) => self.fold_postfix_expression(x),
            UnaryExpression::PreInc(x) => self.fold_unary_expression(*x),
            UnaryExpression::PreDec(x) => self.fold_unary_expression(*x),
            UnaryExpression::UnaryOp(unary_op, cast_expr) => {
                let unary_op = self.fold_unary_operator(unary_op);
                let cast_expr = self.fold_cast_expression(*cast_expr);
                self.fold_unary_expression_unary_op(unary_op, cast_expr)
            }
            UnaryExpression::SizeofExpr(x) => self.fold_unary_expression(*x),
            UnaryExpression::SizeofType(x) => self.fold_type_name(*x),
        }
    }

    fn fold_unary_expression_unary_op(&mut self, unary_op: Self::Result, cast_expr: Self::Result) -> Self::Result;

    fn fold_unary_operator(&mut self, node: UnaryOperator) -> Self::Result;

    fn fold_cast_expression(&mut self, node: CastExpression) -> Self::Result {
        match node {
            CastExpression::Cast(type_name, cast_expr) => {
                let type_name = self.fold_type_name(*type_name);
                let cast_expr = self.fold_cast_expression(*cast_expr);
                self.fold_cast_expression_cast(type_name, cast_expr)
            }
            CastExpression::Unary(unary_expr) => self.fold_unary_expression(unary_expr),
        }
    }

    fn fold_cast_expression_cast(&mut self, type_name: Self::Result, cast_expr: Self::Result) -> Self::Result;

    fn fold_multiplicative_expression(&mut self, node: MultiplicativeExpression) -> Self::Result {
        match node {
            MultiplicativeExpression::Mul(mul_expr, cast_expr) => {
                let mul_expr = self.fold_multiplicative_expression(*mul_expr);
                let cast_expr = self.fold_cast_expression(cast_expr);
                self.fold_multiplicative_expression_mul(mul_expr, cast_expr)
            }
            MultiplicativeExpression::Div(mul_expr, cast_expr) => {
                let mul_expr = self.fold_multiplicative_expression(*mul_expr);
                let cast_expr = self.fold_cast_expression(cast_expr);
                self.fold_multiplicative_expression_div(mul_expr, cast_expr)
            }
            MultiplicativeExpression::Mod(mul_expr, cast_expr) => {
                let mul_expr = self.fold_multiplicative_expression(*mul_expr);
                let cast_expr = self.fold_cast_expression(cast_expr);
                self.fold_multiplicative_expression_mod(mul_expr, cast_expr)
            }
            MultiplicativeExpression::Cast(x) => self.fold_cast_expression(x),
        }
    }

    fn fold_multiplicative_expression_mul(&mut self, mul_expr: Self::Result, cast_expr: Self::Result) -> Self::Result;

    fn fold_multiplicative_expression_div(&mut self, mul_expr: Self::Result, cast_expr: Self::Result) -> Self::Result;

    fn fold_multiplicative_expression_mod(&mut self, mul_expr: Self::Result, cast_expr: Self::Result) -> Self::Result;

    fn fold_additive_expression(&mut self, node: AdditiveExpression) -> Self::Result {
        match node {
            AdditiveExpression::Add(add_expr, mul_expr) => {
                let add_expr = self.fold_additive_expression(*add_expr);
                let mul_expr = self.fold_multiplicative_expression(mul_expr);
                self.fold_additive_expression_add(add_expr, mul_expr)
            }
            AdditiveExpression::Sub(add_expr, mul_expr) => {
                let add_expr = self.fold_additive_expression(*add_expr);
                let mul_expr = self.fold_multiplicative_expression(mul_expr);
                self.fold_additive_expression_sub(add_expr, mul_expr)
            }
            AdditiveExpression::Mul(mul_expr) => self.fold_multiplicative_expression(mul_expr),
        }
    }

    fn fold_additive_expression_add(&mut self, add_expr: Self::Result, mul_expr: Self::Result) -> Self::Result;

    fn fold_additive_expression_sub(&mut self, add_expr: Self::Result, mul_expr: Self::Result) -> Self::Result;

    fn fold_shift_expression(&mut self, node: ShiftExpression) -> Self::Result {
        match node {
            ShiftExpression::Shl(shift_expr, add_expr) => {
                let shift_expr = self.fold_shift_expression(*shift_expr);
                let add_expr = self.fold_additive_expression(add_expr);
                self.fold_shift_expression_shl(shift_expr, add_expr)
            }
            ShiftExpression::Shr(shift_expr, add_expr) => {
                let shift_expr = self.fold_shift_expression(*shift_expr);
                let add_expr = self.fold_additive_expression(add_expr);
                self.fold_shift_expression_shr(shift_expr, add_expr)
            }
            ShiftExpression::Add(add_expr) => self.fold_additive_expression(add_expr),
        }
    }

    fn fold_shift_expression_shl(&mut self, shift_expr: Self::Result, add_expr: Self::Result) -> Self::Result;

    fn fold_shift_expression_shr(&mut self, shift_expr: Self::Result, add_expr: Self::Result) -> Self::Result;

    fn fold_relational_expression(&mut self, node: RelationalExpression) -> Self::Result {
        match node {
            RelationalExpression::Lt(relational_expr, shift_expr) => {
                let relation_expr = self.fold_relational_expression(*relational_expr);
                let shift_expr = self.fold_shift_expression(shift_expr);
                self.fold_relational_expression_lt(relation_expr, shift_expr)
            }
            RelationalExpression::Gt(relational_expr, shift_expr) => {
                let relation_expr = self.fold_relational_expression(*relational_expr);
                let shift_expr = self.fold_shift_expression(shift_expr);
                self.fold_relational_expression_gt(relation_expr, shift_expr)
            }
            RelationalExpression::Le(relational_expr, shift_expr) => {
                let relation_expr = self.fold_relational_expression(*relational_expr);
                let shift_expr = self.fold_shift_expression(shift_expr);
                self.fold_relational_expression_le(relation_expr, shift_expr)
            }
            RelationalExpression::Ge(relational_expr, shift_expr) => {
                let relation_expr = self.fold_relational_expression(*relational_expr);
                let shift_expr = self.fold_shift_expression(shift_expr);
                self.fold_relational_expression_ge(relation_expr, shift_expr)
            }
            RelationalExpression::Shift(x) => self.fold_shift_expression(x),
        }
    }

    fn fold_relational_expression_lt(&mut self, relation_expr: Self::Result, shift_expr: Self::Result) -> Self::Result;

    fn fold_relational_expression_gt(&mut self, relation_expr: Self::Result, shift_expr: Self::Result) -> Self::Result;

    fn fold_relational_expression_le(&mut self, relation_expr: Self::Result, shift_expr: Self::Result) -> Self::Result;

    fn fold_relational_expression_ge(&mut self, relation_expr: Self::Result, shift_expr: Self::Result) -> Self::Result;

    fn fold_equality_expression(&mut self, node: EqualityExpression) -> Self::Result {
        match node {
            EqualityExpression::Eq(eq_expr, relation_expr) => {
                let eq_expr = self.fold_equality_expression(*eq_expr);
                let relation_expr = self.fold_relational_expression(relation_expr);
                self.fold_equality_expression_eq(eq_expr, relation_expr)
            }
            EqualityExpression::Ne(eq_expr, relation_expr) => {
                let eq_expr = self.fold_equality_expression(*eq_expr);
                let relation_expr = self.fold_relational_expression(relation_expr);
                self.fold_equality_expression_ne(eq_expr, relation_expr)
            }
            EqualityExpression::Rel(relation_expr) => self.fold_relational_expression(relation_expr),
        }
    }

    fn fold_equality_expression_eq(&mut self, eq_expr: Self::Result, relation_expr: Self::Result) -> Self::Result;

    fn fold_equality_expression_ne(&mut self, eq_expr: Self::Result, relation_expr: Self::Result) -> Self::Result;

    fn fold_and_expression(&mut self, node: AndExpression) -> Self::Result {
        match node {
            AndExpression::And(and_expr, eq_expr) => {
                let and_expr = self.fold_and_expression(*and_expr);
                let eq_expr = self.fold_equality_expression(eq_expr);
                self.fold_and_expression_and(and_expr, eq_expr)
            }
            AndExpression::Eq(eq_expr) => self.fold_equality_expression(eq_expr)
        }
    }

    fn fold_and_expression_and(&mut self, and_expr: Self::Result, eq_expr: Self::Result) -> Self::Result;

    fn fold_exclusive_or_expression(&mut self, node: ExclusiveOrExpression) -> Self::Result {
        match node {
            ExclusiveOrExpression::Xor(exclusive_or_expr, and_expr) => {
                let exclusive_or_expr = self.fold_exclusive_or_expression(*exclusive_or_expr);
                let and_expr = self.fold_and_expression(and_expr);
                self.fold_exclusive_or_expression_xor(exclusive_or_expr, and_expr)
            }
            ExclusiveOrExpression::And(and_expr) => self.fold_and_expression(and_expr),
        }
    }

    fn fold_exclusive_or_expression_xor(&mut self, exclusive_or_expr: Self::Result, and_expr: Self::Result) -> Self::Result;

    fn fold_inclusive_or_expression(&mut self, node: InclusiveOrExpression) -> Self::Result {
        match node {
            InclusiveOrExpression::Or(inclusive_or_expr, exclusive_or_expr) => {
                let inclusive_or_expr = self.fold_inclusive_or_expression(*inclusive_or_expr);
                let exclusive_or_expr = self.fold_exclusive_or_expression(exclusive_or_expr);
                self.fold_inclusive_or_expression_or(inclusive_or_expr, exclusive_or_expr)
            }
            InclusiveOrExpression::Xor(x) => self.fold_exclusive_or_expression(x)
        }
    }

    fn fold_inclusive_or_expression_or(&mut self, inclusive_or_expr: Self::Result, exclusive_or_expr: Self::Result) -> Self::Result;

    fn fold_logical_and_expression(&mut self, node: LogicalAndExpression) -> Self::Result {
        match node {
            LogicalAndExpression::And(logical_and_expr, inclusive_or_expr) => {
                let logical_and_expr = self.fold_logical_and_expression(*logical_and_expr);
                let inclusive_or_expr = self.fold_inclusive_or_expression(inclusive_or_expr);
                self.fold_logical_and_expression_and(logical_and_expr, inclusive_or_expr)
            }
            LogicalAndExpression::Or(x) => self.fold_inclusive_or_expression(x)
        }
    }


    fn fold_logical_and_expression_and(&mut self, logical_and_expr: Self::Result, inclusive_or_expr: Self::Result) -> Self::Result;
    
    fn fold_logical_or_expression(&mut self, node: LogicalOrExpression) -> Self::Result {
        match node {
            LogicalOrExpression::Or(logical_or_expr, logical_and_expr) => {
                let logical_or_expr = self.fold_logical_or_expression(*logical_or_expr);
                let logical_and_expr = self.fold_logical_and_expression(logical_and_expr);
                self.fold_logical_or_expression_or(logical_or_expr, logical_and_expr)
            }
            LogicalOrExpression::And(x) => self.fold_logical_and_expression(x)
        }
    }

    fn fold_logical_or_expression_or(&mut self, logical_or_expr: Self::Result, logical_and_expr: Self::Result) -> Self::Result;

    fn fold_conditional_expression(&mut self, node: ConditionalExpression) -> Self::Result {
        match node {
            ConditionalExpression::Cond { cond, then_expr, else_expr } => {
                let cond_expr = self.fold_logical_or_expression(cond);
                let then_expr = self.fold_expression(*then_expr);
                let else_expr = self.fold_conditional_expression(*else_expr);
                self.fold_conditional_expression_cond(cond_expr, then_expr, else_expr)
            }
            ConditionalExpression::Or(x) => self.fold_logical_or_expression(x)
        }
    }

    fn fold_conditional_expression_cond(&mut self, cond_expr: Self::Result, then_expr: Self::Result, else_expr: Self::Result) -> Self::Result;

    fn fold_assignment_expression(&mut self, node: AssignmentExpression) -> Self::Result {
        match node {
            AssignmentExpression::Conditional(x) => self.fold_conditional_expression(x),
            AssignmentExpression::Assign { lhs, op, rhs } => {
                let lhs_expr = self.fold_unary_expression(lhs);
                let op = self.fold_assignment_operator(op);
                let rhs = self.fold_assignment_expression(*rhs);
                self.fold_assignment_expression_assign(lhs_expr, op, rhs)
            }
        }
    }

    fn fold_assignment_expression_assign(&mut self, lhs_expr: Self::Result, op: Self::Result, rhs: Self::Result) -> Self::Result;

    fn fold_assignment_operator(&mut self, node: AssignmentOperator) -> Self::Result;

    fn fold_expression(&mut self, node: Expression) -> Self::Result {
        match node {
            Expression::Single(x) => self.fold_assignment_expression(x),
            Expression::Comma(expr, assign_expr) => {
                let expr = self.fold_expression(*expr);
                let assign_expr = self.fold_assignment_expression(assign_expr);
                self.fold_expression_node(expr, assign_expr)
            }
        }
    }

    fn fold_expression_node(&mut self, expr: Self::Result, assign_expr: Self::Result) -> Self::Result;

    fn fold_constant_expression(&mut self, node: ConstantExpression) -> Self::Result {
        self.fold_conditional_expression(node.0)
    }

    fn fold_type_name(&mut self, node: TypeName) -> Self::Result {
        let specifiers = self.fold_specifier_qualifier_list(node.specifiers);
        let abs_declarator = node.abstract_declarator.map(|x| self.fold_abstract_declarator(x));
        self.fold_type_name_node(specifiers, abs_declarator)
    }

    fn fold_type_name_node(&mut self, specifiers: Self::Result, abs_declarator: Option<Self::Result>) -> Self::Result;

    fn fold_initializer(&mut self, node: Initializer) -> Self::Result {
        match node {
            Initializer::Assignment(x) => self.fold_assignment_expression(x),
            Initializer::List(x) => self.fold_initializer_list(x),
        }
    }

    fn fold_initializer_list(&mut self, node: InitializerList) -> Self::Result {
        let node: Vec<_> = node.0.into_iter().map(|x| self.fold_initializer(x)).collect();
        self.fold_initializer_list_node(node)
    }

    fn fold_initializer_list_node(&mut self, node: Vec<Self::Result>) -> Self::Result;

    fn fold_abstract_declarator(&mut self, node: AbstractDeclarator) -> Self::Result {
        match node {
            AbstractDeclarator::Pointer(x) => self.fold_pointer(x),
            AbstractDeclarator::Direct { pointer, direct } => {
                let pointer = pointer.map(|x| self.fold_pointer(x));
                let direct = self.fold_direct_abstract_declarator(direct);
                self.fold_abstract_declarator_direct(pointer, direct)
            }
        }
    }

    fn fold_abstract_declarator_direct(&mut self, pointer: Option<Self::Result>, direct: Self::Result) -> Self::Result;

    fn fold_direct_abstract_declarator(&mut self, node: DirectAbstractDeclarator) -> Self::Result {
        match node {
            DirectAbstractDeclarator::Paren(x) => self.fold_abstract_declarator(*x),
            DirectAbstractDeclarator::Array(constexpr) => {
                let constexpr = constexpr.map(|x| self.fold_constant_expression(x));
                self.fold_direct_abstract_declarator_array(constexpr)
            },
            DirectAbstractDeclarator::ArrayNested(direct_abs_declarator, constexpr) => {
                let direct_abs_declarator = self.fold_direct_abstract_declarator(*direct_abs_declarator);
                let constexpr = constexpr.map(|x| self.fold_constant_expression(x));
                self.fold_direct_abstract_declarator_array_nested(direct_abs_declarator, constexpr)

            }
            DirectAbstractDeclarator::Func(param_type_list) => {
                let param_type_list = param_type_list.map(|x| self.fold_parameter_type_list(x));
                self.fold_direct_abstract_declarator_func(param_type_list)
            }
            DirectAbstractDeclarator::FuncNested(direct_abs_declarator, param_type_list) => {
                let direct_abs_declarator = self.fold_direct_abstract_declarator(*direct_abs_declarator);
                let param_type_list = param_type_list.map(|x| self.fold_parameter_type_list(x));
                self.fold_direct_abstract_declarator_func_nested(direct_abs_declarator, param_type_list)
            }
        }
    }

    fn fold_direct_abstract_declarator_array(&mut self, constexpr: Option<Self::Result>) -> Self::Result;

    fn fold_direct_abstract_declarator_array_nested(&mut self, direct_abs_declarator: Self::Result, constexpr: Option<Self::Result>) -> Self::Result;

    fn fold_direct_abstract_declarator_func(&mut self, param_type_list: Option<Self::Result>) -> Self::Result;

    fn fold_direct_abstract_declarator_func_nested(&mut self, direct_abs_declarator: Self::Result, param_type_list: Option<Self::Result>) -> Self::Result;

    fn fold_parameter_type_list(&mut self, node: ParameterTypeList) -> Self::Result {
        match node {
            ParameterTypeList::Params(param_decl) => {
                let param_decl: Vec<_> = param_decl.into_iter().map(|x| self.fold_parameter_declaration(x)).collect();
                self.fold_parameter_type_list_node(param_decl)
            }
            ParameterTypeList::Variadic(param_decl) => {
                let param_decl: Vec<_> = param_decl.into_iter().map(|x| self.fold_parameter_declaration(x)).collect();
                self.fold_parameter_type_list_node(param_decl)
            }
        }
    }

    fn fold_parameter_type_list_node(&mut self, param_decl: Vec<Self::Result>) -> Self::Result;

    fn fold_parameter_declaration(&mut self, node: ParameterDeclaration) -> Self::Result {
        match node {
            ParameterDeclaration::WithDeclarator { specifiers, declarator } => {
                let specs = self.fold_declaration_specifiers(specifiers);
                let declarator = self.fold_declarator(declarator);
                self.fold_parameter_declaration_with_declarator(specs, declarator)
            }
            ParameterDeclaration::Abstract { specifiers, abstract_declarator } => {
                let specs = self.fold_declaration_specifiers(specifiers);
                let abs_decl = abstract_declarator.map(|x| self.fold_abstract_declarator(x));
                self.fold_parameter_declaration_abstract(specs, abs_decl)
            }
        }
    }

    fn fold_parameter_declaration_with_declarator(&mut self, specs: Self::Result, declarator: Self::Result) -> Self::Result;
    fn fold_parameter_declaration_abstract(&mut self, specs: Self::Result, abs_decl: Option<Self::Result>) -> Self::Result;

    fn fold_specifier_qualifier_list(&mut self, node: SpecifierQualifierList) -> Self::Result {
        match node {
            SpecifierQualifierList::TypeSpecifier(type_spec, spec_qual_list) => {
                let type_spec = self.fold_type_specifier(type_spec);
                let spec_qual_list = spec_qual_list.map(|x| self.fold_specifier_qualifier_list(*x));
                self.fold_specifier_qualifier_list_type_spec(type_spec, spec_qual_list)
            }
            SpecifierQualifierList::TypeQualifier(type_qual, spec_qual_list) => {
                let type_qual = self.fold_type_qualifier(type_qual);
                let spec_qual_list = spec_qual_list.map(|x| self.fold_specifier_qualifier_list(*x));
                self.fold_specifier_qualifier_list_type_qual(type_qual, spec_qual_list)
            }
        }
    }

    fn fold_specifier_qualifier_list_type_spec(&mut self, type_spec: Self::Result, spec_qual_list: Option<Self::Result>) -> Self::Result;

    fn fold_specifier_qualifier_list_type_qual(&mut self, type_qual: Self::Result, spec_qual_list: Option<Self::Result>) -> Self::Result;

    fn fold_token(&mut self, node: Token) -> Self::Result;
}
