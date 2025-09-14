#![allow(clippy::all)]

use enum_as_inner::EnumAsInner;
use crate::types::token::Token;


macro_rules! make_opt {
    ($field:ident, $variant:ident, $func_name:ident) => {
        pub fn $func_name(item: SemanticValue) -> SemanticValue {
            let item = item.$field().unwrap();
            SemanticValue::$variant(Some(item))
        }
    };
}

macro_rules! make_one {
    ($field:ident, $variant:ident, $variant_field:ident, $func_name:ident) => {
        pub fn $func_name(item: SemanticValue) -> SemanticValue {
            let item = item.$field().unwrap();
            SemanticValue::$variant($variant::$variant_field(item))
        }
    };
}

macro_rules! make_two {
    ($field:ident, $field2:ident, $variant:ident, $variant_field:ident, $func_name:ident) => {
        pub fn $func_name(item: SemanticValue, item2: SemanticValue) -> SemanticValue {
            let item = item.$field().unwrap();
            let item2 = item2.$field2().unwrap();
            SemanticValue::$variant($variant::$variant_field(item, item2))
        }
    };
}

macro_rules! make_struct_list {
    ($field:ident, $variant:ident, $func_name:ident) => {
        pub fn $func_name(item: SemanticValue) -> SemanticValue {
            let item = item.$field().unwrap();
            SemanticValue::$variant(Self(vec![item]))
        }
    };
}


macro_rules! make_struct_insert {
    ($list_field:ident, $field:ident, $variant:ident, $func_name:ident) => {
        pub fn $func_name(list: SemanticValue, decl: SemanticValue) -> SemanticValue {
            let list = list.$list_field().unwrap();
            let decl = decl.$field().unwrap();
            SemanticValue::$variant(Self(list_push(list.0, decl)))
        }
    };
}

macro_rules! make_list {
    ($field:ident, $variant:ident, $func_name:ident) => {
        pub fn $func_name(item: SemanticValue) -> SemanticValue {
            let item = item.$field().unwrap();
            SemanticValue::$variant(vec![item])
        }
    };
}


macro_rules! make_insert {
    ($list_field:ident, $field:ident, $variant:ident, $func_name:ident) => {
        pub fn $func_name(list: SemanticValue, decl: SemanticValue) -> SemanticValue {
            let list = list.$list_field().unwrap();
            let decl = decl.$field().unwrap();
            SemanticValue::$variant(list_push(list, decl))
        }
    };
}

macro_rules! make_zero {
    ($variant:ident, $variant_field:ident, $func_name:ident) => {
        pub fn $func_name(item: SemanticValue) -> SemanticValue {
            let token = item.into_token().unwrap();
            SemanticValue::$variant($variant::$variant_field(token))
        }
    };
}


/*
    pub fn make_decl_list(decl: SemanticValue) -> SemanticValue {
        let decl = decl.into_declaration().unwrap();
        SemanticValue::DeclarationList(Self(vec![decl]))
    }


    pub fn insert(list: SemanticValue, decl: SemanticValue) -> SemanticValue {
        let list = list.into_declaration_list().unwrap();
        let decl = decl.into_declaration().unwrap();
        SemanticValue::DeclarationList(Self(list_push(list.0, decl)))
    }
 */

// #[derive(EnumAsInner)]
#[derive(Debug, EnumAsInner, Default)]
pub enum SemanticValue {
    TranslationUnit(TranslationUnit),
    ExternalDeclaration(ExternalDeclaration),
    FunctionDefinition(FunctionDefinition),
    DeclarationListOpt(Option<DeclarationList>),
    DeclarationList(DeclarationList),
    Declaration(Declaration),
    InitDeclaratorListOpt(Option<InitDeclaratorList>),
    InitDeclaratorList(InitDeclaratorList),
    InitDeclarator(InitDeclarator),
    DeclarationSpecifiers(DeclarationSpecifiers),
    DeclarationSpecifiersOpt(Option<DeclarationSpecifiers>),
    StorageClassSpecifier(StorageClassSpecifier),
    TypeSpecifier(TypeSpecifier),
    TypeQualifier(TypeQualifier),
    StructOrUnionSpecifier(StructOrUnionSpecifier),
    StructOrUnion(StructOrUnion),
    IdentifierOpt(IdentifierOpt),
    StructDeclarationList(Vec<StructDeclaration>),
    StructDeclaration(StructDeclaration),
    SpecifierQualifierList(SpecifierQualifierList),
    SpecifierQualifierListOpt(Option<SpecifierQualifierList>),
    StructDeclaratorList(Vec<StructDeclarator>),
    StructDeclarator(StructDeclarator),
    EnumSpecifier(EnumSpecifier),
    EnumeratorList(Vec<Enumerator>),
    Enumerator(Enumerator),
    Declarator(Declarator),
    PointerOpt(Option<Pointer>),
    Pointer(Pointer),
    TypeQualifierList(Vec<TypeQualifier>),
    DirectDeclarator(DirectDeclarator),
    ConstantExpressionOpt(Option<ConstantExpression>),
    IdentifierListOpt(Option<IdentifierList>),
    IdentifierList(IdentifierList),
    ParameterTypeList(ParameterTypeList),
    ParameterList(Vec<ParameterDeclaration>),
    ParameterDeclaration(ParameterDeclaration),
    AbstractDeclaratorOpt(Option<AbstractDeclarator>),
    AbstractDeclarator(AbstractDeclarator),
    DirectAbstractDeclarator(DirectAbstractDeclarator),
    ParameterTypeListOpt(Option<ParameterTypeList>),
    Initializer(Initializer),
    InitializerList(InitializerList),
    Statement(Statement),
    LabeledStatement(LabeledStatement),
    CompoundStatement(CompoundStatement),
    BlockItemList(Vec<BlockItem>),
    BlockItem(BlockItem),
    ExpressionStatement(ExpressionStatement),
    SelectionStatement(SelectionStatement),
    IterationStatement(IterationStatement),
    ExpressionOpt(Option<Expression>),
    JumpStatement(JumpStatement),
    PrimaryExpression(PrimaryExpression),
    Constant(Constant),
    String(Vec<Token>),
    PostfixExpression(PostfixExpression),
    ArgumentExpressionListOpt(Option<ArgumentExpressionList>),
    ArgumentExpressionList(ArgumentExpressionList),
    UnaryExpression(UnaryExpression),
    UnaryOperator(UnaryOperator),
    CastExpression(CastExpression),
    MultiplicativeExpression(MultiplicativeExpression),
    AdditiveExpression(AdditiveExpression),
    ShiftExpression(ShiftExpression),
    RelationalExpression(RelationalExpression),
    EqualityExpression(EqualityExpression),
    AndExpression(AndExpression),
    ExclusiveOrExpression(ExclusiveOrExpression),
    InclusiveOrExpression(InclusiveOrExpression),
    LogicalAndExpression(LogicalAndExpression),
    LogicalOrExpression(LogicalOrExpression),
    ConditionalExpression(ConditionalExpression),
    AssignmentExpression(AssignmentExpression),
    AssignmentOperator(AssignmentOperator),
    Expression(Expression),
    ConstantExpression(ConstantExpression),
    TypeName(TypeName),
    Token(Token),
    #[default]
    None
}



/// ====== Grammar CST Types ======

#[derive(Debug, Clone)]
pub enum TranslationUnit {
    Single(Box<ExternalDeclaration>),
    Multi(Box<TranslationUnit>, Box<ExternalDeclaration>),
}

impl TranslationUnit {
    pub fn make_single(external: SemanticValue) -> SemanticValue {
        let external = external.into_external_declaration().unwrap();
        SemanticValue::TranslationUnit(TranslationUnit::Single(Box::new(external)))
    }
    pub fn make_multi(trans: SemanticValue, external: SemanticValue) -> SemanticValue {
        let trans = trans.into_translation_unit().unwrap();
        let external = external.into_external_declaration().unwrap();
        SemanticValue::TranslationUnit(TranslationUnit::Multi(Box::new(trans), Box::new(external)))
    }
}

#[derive(Debug, Clone)]
pub enum ExternalDeclaration {
    FunctionDefinition(FunctionDefinition),
    Declaration(Declaration),
}

impl ExternalDeclaration {
    make_one!(into_function_definition, ExternalDeclaration, FunctionDefinition, make_function_definition);
    make_one!(into_declaration, ExternalDeclaration, Declaration, make_declaration);
}

#[derive(Debug, Clone)]
pub enum FunctionDefinition {
    WithSpecifiers {
        declaration_specifiers: DeclarationSpecifiers,
        declarator: Declarator,
        declaration_list: Option<DeclarationList>,
        body: CompoundStatement,
    },
    WithoutSpecifiers {
        declarator: Declarator,
        declaration_list: Option<DeclarationList>,
        body: CompoundStatement,
    },
}

impl FunctionDefinition {
    pub fn make_with_specifiers(decl_spec: SemanticValue, decl: SemanticValue, decl_list_opt: SemanticValue, body: SemanticValue) -> SemanticValue {
        let decl_spec = decl_spec.into_declaration_specifiers().unwrap();
        let decl = decl.into_declarator().unwrap();
        let decl_list_opt = decl_list_opt.into_declaration_list_opt().unwrap();
        let body = body.into_compound_statement().unwrap();
        SemanticValue::FunctionDefinition(Self::WithSpecifiers {
            declaration_specifiers: decl_spec,
            declarator: decl,
            declaration_list: decl_list_opt,
            body
        })
    }

    pub fn make_without_specifiers(decl: SemanticValue, decl_list_opt: SemanticValue, body: SemanticValue) -> SemanticValue {
        let decl = decl.into_declarator().unwrap();
        let decl_list_opt = decl_list_opt.into_declaration_list_opt().unwrap();
        let body = body.into_compound_statement().unwrap();
        SemanticValue::FunctionDefinition(Self::WithoutSpecifiers {
            declarator: decl,
            declaration_list: decl_list_opt,
            body
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeclarationList(pub Vec<Declaration>);

impl DeclarationList {
    pub fn make_decl_list(decl: SemanticValue) -> SemanticValue {
        let decl = decl.into_declaration().unwrap();
        SemanticValue::DeclarationList(Self(vec![decl]))
    }


    pub fn insert(list: SemanticValue, decl: SemanticValue) -> SemanticValue {
        let list = list.into_declaration_list().unwrap();
        let decl = decl.into_declaration().unwrap();
        SemanticValue::DeclarationList(Self(list_push(list.0, decl)))
    }

    /// 返回一个空的DeclarationList
    pub fn empty() -> DeclarationList {
        DeclarationList(Vec::new())
    }

}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub specifiers: DeclarationSpecifiers,
    pub init_declarators: Option<InitDeclaratorList>,
}

impl Declaration {
    pub fn make_declaration(spec: SemanticValue, init_decl: SemanticValue) -> SemanticValue {
        let spec = spec.into_declaration_specifiers().unwrap();
        let init_decl = init_decl.into_init_declarator_list_opt().unwrap();
        SemanticValue::Declaration(Self {
            specifiers: spec,
            init_declarators: init_decl,
        })
    }
}

#[derive(Debug, Clone)]
pub struct InitDeclaratorList(pub Vec<InitDeclarator>);

impl InitDeclaratorList {
    make_struct_list!(into_init_declarator, InitDeclaratorList, make_init_decl_list);
    make_struct_insert!(into_init_declarator_list, into_init_declarator, InitDeclaratorList, insert);

}

#[derive(Debug, Clone)]
pub enum InitDeclarator {
    Plain(Declarator),
    Initialized(Declarator, Initializer),
}

impl InitDeclarator {

    make_one!(into_declarator, InitDeclarator, Plain, make_plain);
    make_two!(into_declarator, into_initializer, InitDeclarator, Initialized, make_initialized);
}

#[derive(Debug, Clone)]
pub enum DeclarationSpecifiers {
    StorageClass(StorageClassSpecifier, Option<Box<DeclarationSpecifiers>>),
    TypeSpecifier(TypeSpecifier, Option<Box<DeclarationSpecifiers>>),
    TypeQualifier(TypeQualifier, Option<Box<DeclarationSpecifiers>>),
}

impl DeclarationSpecifiers {
    pub fn make_storage_class(item: SemanticValue, decl_spec_opt: SemanticValue) -> SemanticValue {
        let item = item.into_storage_class_specifier().unwrap();
        let decl_spec_opt = decl_spec_opt.into_declaration_specifiers_opt().unwrap()
            .map(Box::new);
        SemanticValue::DeclarationSpecifiers(Self::StorageClass(item, decl_spec_opt))
    }
    pub fn make_type_specifier(item: SemanticValue, decl_spec_opt: SemanticValue) -> SemanticValue {
        let item = item.into_type_specifier().unwrap();
        let decl_spec_opt = decl_spec_opt.into_declaration_specifiers_opt().unwrap()
            .map(Box::new);
        SemanticValue::DeclarationSpecifiers(Self::TypeSpecifier(item, decl_spec_opt))
    }
    pub fn make_type_qualifier(item: SemanticValue, decl_spec_opt: SemanticValue) -> SemanticValue {
        let item = item.into_type_qualifier().unwrap();
        let decl_spec_opt = decl_spec_opt.into_declaration_specifiers_opt().unwrap()
            .map(Box::new);
        SemanticValue::DeclarationSpecifiers(Self::TypeQualifier(item, decl_spec_opt))
    }
}

#[derive(Debug, Clone)]
pub enum StorageClassSpecifier {
    Typedef(Token),
    Extern(Token),
    Static(Token),
    Auto(Token),
    Register(Token),
}

impl StorageClassSpecifier {
    make_zero!(StorageClassSpecifier, Typedef, make_typedef);
    make_zero!(StorageClassSpecifier, Extern, make_extern);
    make_zero!(StorageClassSpecifier, Static, make_static);
    make_zero!(StorageClassSpecifier, Auto, make_auto);
    make_zero!(StorageClassSpecifier, Register, make_register);
}

#[derive(Debug, Clone)]
pub enum TypeSpecifier {
    Void(Token),
    Char(Token),
    Short(Token),
    Int(Token),
    Long(Token),
    Signed(Token),
    Unsigned(Token),
    Float(Token),
    Double(Token),
    StructOrUnion(StructOrUnionSpecifier),
    Enum(EnumSpecifier),
    TypeName(Token), // typedef name resolved by lexer
}

impl TypeSpecifier {

    make_zero!(TypeSpecifier, Void, make_void);
    make_zero!(TypeSpecifier, Char, make_char);
    make_zero!(TypeSpecifier, Short, make_short);
    make_zero!(TypeSpecifier, Int, make_int);
    make_zero!(TypeSpecifier, Long, make_long);
    make_zero!(TypeSpecifier, Signed, make_signed);
    make_zero!(TypeSpecifier, Unsigned, make_unsigned);
    make_zero!(TypeSpecifier, Float, make_float);
    make_zero!(TypeSpecifier, Double, make_double);

    make_one!(into_struct_or_union_specifier, TypeSpecifier, StructOrUnion, make_struct);
    make_one!(into_enum_specifier, TypeSpecifier, Enum, make_enum);
    make_one!(into_token, TypeSpecifier, TypeName, make_type_name);
}

#[derive(Debug, Clone)]
pub enum TypeQualifier {
    Const(Token),
    Volatile(Token),
}

impl TypeQualifier {
    make_zero!(TypeQualifier, Const, make_const);
    make_zero!(TypeQualifier, Volatile, make_volatile);
}

#[derive(Debug, Clone)]
pub enum StructOrUnionSpecifier {
    Defined {
        kind: StructOrUnion,
        name: IdentifierOpt,
        fields: Vec<StructDeclaration>,
    },
    Named {
        kind: StructOrUnion,
        name: Token,
    },
}

impl StructOrUnionSpecifier {
    pub fn make_defined(kind: SemanticValue, name: SemanticValue, fields: SemanticValue) -> SemanticValue {
        let kind = kind.into_struct_or_union().unwrap();
        let name = name.into_identifier_opt().unwrap();
        let fields = fields.into_struct_declaration_list().unwrap();
        SemanticValue::StructOrUnionSpecifier(Self::Defined {
            kind,
            name,
            fields
        })
    }

    pub fn make_named(kind: SemanticValue, name: SemanticValue) -> SemanticValue {
        let kind = kind.into_struct_or_union().unwrap();
        let name = name.into_token().unwrap();
        SemanticValue::StructOrUnionSpecifier(Self::Named {
            kind,
            name,
        })
    }


}

#[derive(Debug, Clone)]
pub enum StructOrUnion {
    Struct(Token),
    Union(Token),
}

impl StructOrUnion {
    make_zero!(StructOrUnion, Struct, make_struct);
    make_zero!(StructOrUnion, Union, make_union);
}

pub type IdentifierOpt = Option<Token>;


#[derive(Debug, Clone)]
pub struct StructDeclaration {
    pub specifiers: SpecifierQualifierList,
    pub declarators: Vec<StructDeclarator>,
}

impl StructDeclaration {
    pub fn make_struct_declaration(specifiers: SemanticValue, declarators: SemanticValue) -> SemanticValue {
        let specifiers = specifiers.into_specifier_qualifier_list().unwrap();
        let declarators = declarators.into_struct_declarator_list().unwrap();
        SemanticValue::StructDeclaration(Self {
            specifiers,
            declarators
        })
    }
}

#[derive(Debug, Clone)]
pub enum StructDeclarator {
    Declarator(Declarator),
    Bitfield(Option<Declarator>, ConstantExpression),
}

impl StructDeclarator {

    make_one!(into_declarator, StructDeclarator, Declarator, make_declarator);
    pub fn make_bitfield(decl_opt: SemanticValue, constexpr: SemanticValue) -> SemanticValue {
        let decl_opt = match decl_opt {
            SemanticValue::Declarator(x) => Some(x),
            SemanticValue::None => None,
            _ => unreachable!()
        };
        let constexpr = constexpr.into_constant_expression().unwrap();

        SemanticValue::StructDeclarator(Self::Bitfield(decl_opt, constexpr))
    }
}

#[derive(Debug, Clone)]
pub enum EnumSpecifier {
    Defined {
        name: IdentifierOpt,
        enumerators: Vec<Enumerator>,
    },
    Named(Token),
}

impl EnumSpecifier {
    pub fn make_defined(name: SemanticValue, enumerators: SemanticValue) -> SemanticValue {
        let name = name.into_identifier_opt().unwrap();
        let enumerators = enumerators.into_enumerator_list().unwrap();
        SemanticValue::EnumSpecifier(Self::Defined {
            name,
            enumerators
        })
    }

    make_one!(into_token ,EnumSpecifier, Named, make_named);
}

#[derive(Debug, Clone)]
pub enum Enumerator {
    Plain(Token),
    WithValue(Token, ConstantExpression),
}

impl Enumerator {
    make_one!(into_token ,Enumerator, Plain, make_plain);

    make_two!(into_token, into_constant_expression, Enumerator, WithValue, make_with_value);
}

// declarators
#[derive(Debug, Clone)]
pub struct Declarator {
    pub pointer: Option<Pointer>,
    pub direct: DirectDeclarator,
}

impl Declarator {
    pub fn make_declarator(pointer_opt: SemanticValue, direct: SemanticValue) -> SemanticValue {
        let pointer = pointer_opt.into_pointer_opt().unwrap();
        let direct = direct.into_direct_declarator().unwrap();
        SemanticValue::Declarator(Self { pointer, direct })
    }
}

#[derive(Debug, Clone)]
pub enum Pointer {
    Single(Vec<TypeQualifier>, Option<Box<Pointer>>),
}

impl Pointer {
    pub fn make_pointer(type_qual_list: SemanticValue, pointer: SemanticValue) -> SemanticValue {
        let type_qual_list = match type_qual_list {
            SemanticValue::None => vec![],
            SemanticValue::TypeQualifierList(x) => x,
            _ => unreachable!()
        };
        let pointer = match pointer {
            SemanticValue::None => None,
            SemanticValue::Pointer(x) => Some(Box::new(x)),
            _ => unreachable!()
        };
        SemanticValue::Pointer(Pointer::Single(type_qual_list, pointer))
    }
}

#[derive(Debug, Clone)]
pub enum DirectDeclarator {
    Id(Token),
    Paren(Box<Declarator>),
    Array(Box<DirectDeclarator>, Option<ConstantExpression>),
    FuncParams(Box<DirectDeclarator>, ParameterTypeList),
    FuncIdentifiers(Box<DirectDeclarator>, Option<IdentifierList>),
}

impl DirectDeclarator {

    make_one!(into_token, DirectDeclarator, Id, make_id);

    pub fn make_paren(item: SemanticValue) -> SemanticValue {
        let item = item.into_declarator().unwrap();
        SemanticValue::DirectDeclarator(DirectDeclarator::Paren(Box::new(item)))
    }

    pub fn make_array(direct: SemanticValue, constexpr_opt: SemanticValue) -> SemanticValue {
        let direct = direct.into_direct_declarator().unwrap();
        let constexpr_opt = constexpr_opt.into_constant_expression_opt().unwrap();
        SemanticValue::DirectDeclarator(DirectDeclarator::Array(Box::new(direct), constexpr_opt))
    }

    pub fn make_func_params(direct: SemanticValue, param_type_list: SemanticValue) -> SemanticValue {
        let direct = direct.into_direct_declarator().unwrap();
        let param_type_list = param_type_list.into_parameter_type_list().unwrap();
        SemanticValue::DirectDeclarator(DirectDeclarator::FuncParams(Box::new(direct), param_type_list))
    }

    pub fn make_func_identifiers(direct: SemanticValue, tokens: SemanticValue) -> SemanticValue {
        let direct = direct.into_direct_declarator().unwrap();
        let tokens = tokens.into_identifier_list_opt().unwrap();
        SemanticValue::DirectDeclarator(DirectDeclarator::FuncIdentifiers(Box::new(direct), tokens))
    }
}




/* ===================== Statements ===================== */

#[derive(Debug, Clone)]
pub enum Statement {
    Labeled(LabeledStatement),
    Compound(CompoundStatement),
    Expression(ExpressionStatement),
    Selection(SelectionStatement),
    Iteration(IterationStatement),
    Jump(JumpStatement),
}

impl Statement {
    make_one!(into_labeled_statement, Statement, Labeled, make_labeled);
    make_one!(into_compound_statement, Statement, Compound, make_compound);
    make_one!(into_expression_statement, Statement, Expression, make_expression);
    make_one!(into_selection_statement, Statement, Selection, make_selection);
    make_one!(into_iteration_statement, Statement, Iteration, make_iteration);
    make_one!(into_jump_statement, Statement, Jump, make_jump);
}

#[derive(Debug, Clone)]
pub enum LabeledStatement {
    Label(Token, Box<Statement>),
    Case(ConstantExpression, Box<Statement>),
    Default(Box<Statement>),
}

impl LabeledStatement {
    pub fn make_label(token: SemanticValue, statement: SemanticValue) -> SemanticValue {
        let token = token.into_token().unwrap();
        let statement = statement.into_statement().unwrap();
        SemanticValue::LabeledStatement(Self::Label(token, Box::new(statement)))
    }

    pub fn make_case(constexpr: SemanticValue, statement: SemanticValue) -> SemanticValue {
        let constexpr = constexpr.into_constant_expression().unwrap();
        let statement = statement.into_statement().unwrap();
        SemanticValue::LabeledStatement(Self::Case(constexpr, Box::new(statement)))
    }

    pub fn make_default(statement: SemanticValue) -> SemanticValue {
        let statement = statement.into_statement().unwrap();
        SemanticValue::LabeledStatement(Self::Default(Box::new(statement)))
    }
}

#[derive(Debug, Clone)]
pub enum CompoundStatement {
    Empty(Token),
    Block(Vec<BlockItem>),
}

impl CompoundStatement {

    make_zero!(CompoundStatement, Empty, make_empty);
    make_one!(into_block_item_list, CompoundStatement, Block, make_expr);
}

#[derive(Debug, Clone)]
pub enum BlockItem {
    Declaration(Declaration),
    Statement(Statement),
}

impl BlockItem {
    make_one!(into_declaration, BlockItem, Declaration, make_declaration);
    make_one!(into_statement, BlockItem, Statement, make_statement);
}

#[derive(Debug, Clone)]
pub enum ExpressionStatement {
    Empty(Token),
    Expr(Expression),
}

impl ExpressionStatement {
    make_zero!(ExpressionStatement, Empty, make_empty);
    make_one!(into_expression, ExpressionStatement, Expr, make_expr);
}


#[derive(Debug, Clone)]
pub enum SelectionStatement {
    If {
        cond: Expression,
        then_stmt: Box<Statement>,
        else_stmt: Option<Box<Statement>>,
    },
    Switch {
        cond: Expression,
        body: Box<Statement>,
    },
}

impl SelectionStatement {
    pub fn make_if(cond: SemanticValue, then_stmt: SemanticValue, else_stmt: SemanticValue) -> SemanticValue {
        let cond = cond.into_expression().unwrap();
        let then_stmt = then_stmt.into_statement().unwrap();
        let else_stmt = match else_stmt {
            SemanticValue::Statement(x) => Some(Box::new(x)),
            SemanticValue::None => None,
            _ => unreachable!()
        };
        SemanticValue::SelectionStatement(Self::If {
            cond,
            then_stmt: Box::new(then_stmt),
            else_stmt
        })
    }

    pub fn make_switch(cond: SemanticValue, body: SemanticValue) -> SemanticValue {
        let cond = cond.into_expression().unwrap();
        let body = body.into_statement().unwrap();
        SemanticValue::SelectionStatement(Self::Switch {
            cond,
            body: Box::new(body),
        })
    }
}

#[derive(Debug, Clone)]
pub enum IterationStatement {
    While {
        cond: Expression,
        body: Box<Statement>,
    },
    DoWhile {
        body: Box<Statement>,
        cond: Expression,
    },
    For {
        init: Option<Expression>,
        cond: Option<Expression>,
        step: Option<Expression>,
        body: Box<Statement>,
    },
}

impl IterationStatement {
    pub fn make_while(cond: SemanticValue, body: SemanticValue) -> SemanticValue {
        let cond = cond.into_expression().unwrap();
        let body = body.into_statement().unwrap();
        SemanticValue::IterationStatement(Self::While {
            cond,
            body: Box::new(body)
        })
    }

    pub fn make_do_while(body: SemanticValue, cond: SemanticValue) -> SemanticValue {
        let body = body.into_statement().unwrap();
        let cond = cond.into_expression().unwrap();
        SemanticValue::IterationStatement(Self::DoWhile {
            body: Box::new(body),
            cond
        })
    }

    pub fn make_for(init: SemanticValue, cond: SemanticValue, step: SemanticValue, body: SemanticValue) -> SemanticValue {
        let init = init.into_expression_opt().unwrap();
        let cond = cond.into_expression_opt().unwrap();
        let step = step.into_expression_opt().unwrap();
        let body = body.into_statement().unwrap();
        SemanticValue::IterationStatement(Self::For {
            init,
            cond,
            step,
            body: Box::new(body)
        })
    }
}

#[derive(Debug, Clone)]
pub enum JumpStatement {
    Goto(Token),
    Continue(Token),
    Break(Token),
    Return(Option<Expression>),
}

impl JumpStatement {
    make_one!(into_token, JumpStatement, Goto, make_goto);
    make_zero!(JumpStatement, Continue, make_continue);
    make_zero!(JumpStatement, Break, make_break);
    pub fn make_return(item: SemanticValue) -> SemanticValue {
        let item = match item {
            SemanticValue::None => None,
            SemanticValue::Expression(expr) => Some(expr),
            _ => unreachable!()
        };
        SemanticValue::JumpStatement(JumpStatement::Return(item))
    }


}

/* ===================== Expressions ===================== */

#[derive(Debug, Clone)]
pub enum PrimaryExpression {
    Id(Token),
    Constant(Constant),
    StringLiteral(Vec<Token>), // adjacent string concat
    Paren(Box<Expression>),
}

impl PrimaryExpression {
    make_one!(into_token, PrimaryExpression, Id, make_id);
    make_one!(into_constant, PrimaryExpression, Constant, make_constant);
    make_one!(into_string, PrimaryExpression, StringLiteral, make_string_literal);

    pub fn make_paren(expr: SemanticValue) -> SemanticValue {
        let expr = expr.into_expression().unwrap();
        SemanticValue::PrimaryExpression(Self::Paren(Box::new(expr)))
    }
}

#[derive(Debug, Clone)]
pub enum Constant {
    Int(Token),   // could refine later
    Float(Token),
    Char(Token),
}

impl Constant {
    make_zero!(Constant, Int, make_int);
    make_zero!(Constant, Float, make_float);
    make_zero!(Constant, Char, make_char);
}

#[derive(Debug, Clone)]
pub enum PostfixExpression {
    Primary(PrimaryExpression),
    Array(Box<PostfixExpression>, Box<Expression>),
    Call(Box<PostfixExpression>, Option<ArgumentExpressionList>),
    Field(Box<PostfixExpression>, Token),
    Arrow(Box<PostfixExpression>, Token),
    Inc(Box<PostfixExpression>),
    Dec(Box<PostfixExpression>),
}

impl PostfixExpression {
    pub fn make_primary(primary: SemanticValue) -> SemanticValue {
        let primary = primary.into_primary_expression().unwrap();
        SemanticValue::PostfixExpression(Self::Primary(primary))
    }

    pub fn make_array(postfix: SemanticValue, expr: SemanticValue) -> SemanticValue {
        let postfix = postfix.into_postfix_expression().unwrap();
        let expr = expr.into_expression().unwrap();
        SemanticValue::PostfixExpression(Self::Array(Box::new(postfix), Box::new(expr)))
    }

    pub fn make_call(postfix: SemanticValue, arg: SemanticValue) -> SemanticValue {
        let postfix = postfix.into_postfix_expression().unwrap();
        let arg = arg.into_argument_expression_list_opt().unwrap();
        SemanticValue::PostfixExpression(Self::Call(Box::new(postfix), arg))
    }

    pub fn make_field(postfix: SemanticValue, id: SemanticValue) -> SemanticValue {
        let postfix = postfix.into_postfix_expression().unwrap();
        let id = id.into_token().unwrap();
        SemanticValue::PostfixExpression(Self::Field(Box::new(postfix), id))
    }

    pub fn make_arrow(postfix: SemanticValue, id: SemanticValue) -> SemanticValue {
        let postfix = postfix.into_postfix_expression().unwrap();
        let id = id.into_token().unwrap();
        SemanticValue::PostfixExpression(Self::Arrow(Box::new(postfix), id))
    }

    pub fn make_inc(postfix: SemanticValue) -> SemanticValue {
        let postfix = postfix.into_postfix_expression().unwrap();
        SemanticValue::PostfixExpression(Self::Inc(Box::new(postfix)))
    }

    pub fn make_dec(postfix: SemanticValue) -> SemanticValue {
        let postfix = postfix.into_postfix_expression().unwrap();
        SemanticValue::PostfixExpression(Self::Dec(Box::new(postfix)))
    }
}

pub type ArgumentExpressionList = Vec<AssignmentExpression>;


#[derive(Debug, Clone)]
pub enum UnaryExpression {
    Postfix(PostfixExpression),
    PreInc(Box<UnaryExpression>),
    PreDec(Box<UnaryExpression>),
    UnaryOp(UnaryOperator, Box<CastExpression>),
    SizeofExpr(Box<UnaryExpression>),
    SizeofType(Box<TypeName>), //  KEYWORD_SIZEOF LPAREN type_name RPAREN
}

impl UnaryExpression {
    pub fn make_postfix(postfix: SemanticValue) -> SemanticValue {
        let postfix = postfix.into_postfix_expression().unwrap();
        SemanticValue::UnaryExpression(Self::Postfix(postfix))
    }

    pub fn make_pre_inc(unary: SemanticValue) -> SemanticValue {
        let unary = unary.into_unary_expression().unwrap();
        SemanticValue::UnaryExpression(Self::PreInc(Box::new(unary)))
    }

    pub fn make_pre_dec(unary: SemanticValue) -> SemanticValue {
        let unary = unary.into_unary_expression().unwrap();
        SemanticValue::UnaryExpression(Self::PreInc(Box::new(unary)))
    }

    pub fn make_unary_op(unary: SemanticValue, cast: SemanticValue) -> SemanticValue {
        let unary = unary.into_unary_operator().unwrap();
        let cast = cast.into_cast_expression().unwrap();
        SemanticValue::UnaryExpression(Self::UnaryOp(unary, Box::new(cast)))
    }

    pub fn make_sizeof_expr(unary: SemanticValue) -> SemanticValue {
        let unary = unary.into_unary_expression().unwrap();
        SemanticValue::UnaryExpression(Self::SizeofExpr(Box::new(unary)))
    }

    pub fn make_sizeof_type(type_name: SemanticValue) -> SemanticValue {
        let type_name = type_name.into_type_name().unwrap();
        SemanticValue::UnaryExpression(Self::SizeofType(Box::new(type_name)))
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    AddressOf(Token),
    Deref(Token),
    Plus(Token),
    Minus(Token),
    BitNot(Token),
    Not(Token),
}

impl UnaryOperator {
    make_zero!(UnaryOperator, AddressOf, address_of);
    make_zero!(UnaryOperator, Deref, deref);
    make_zero!(UnaryOperator, Plus, plus);
    make_zero!(UnaryOperator, Minus, minus);
    make_zero!(UnaryOperator, BitNot, bit_not);
    make_zero!(UnaryOperator, Not, not);
}

#[derive(Debug, Clone)]
pub enum CastExpression {
    Cast(Box<TypeName>, Box<CastExpression>),
    Unary(UnaryExpression),
}

impl CastExpression {
    pub fn make_cast(type_name: SemanticValue, cast: SemanticValue) -> SemanticValue {
        let type_name = type_name.into_type_name().unwrap();
        let cast = cast.into_cast_expression().unwrap();
        SemanticValue::CastExpression(Self::Cast(Box::new(type_name), Box::new(cast)))
    }

    pub fn make_unary(unary: SemanticValue) -> SemanticValue {
        let unary = unary.into_unary_expression().unwrap();
        SemanticValue::CastExpression(Self::Unary(unary))
    }
}

#[derive(Debug, Clone)]
pub enum MultiplicativeExpression {
    Mul(Box<MultiplicativeExpression>, CastExpression),
    Div(Box<MultiplicativeExpression>, CastExpression),
    Mod(Box<MultiplicativeExpression>, CastExpression),
    Cast(CastExpression),
}

impl MultiplicativeExpression {
    pub fn make_mul(multi: SemanticValue, cast: SemanticValue) -> SemanticValue {
        let multi = multi.into_multiplicative_expression().unwrap();
        let cast = cast.into_cast_expression().unwrap();
        SemanticValue::MultiplicativeExpression(Self::Mul(Box::new(multi), cast))
    }

    pub fn make_div(multi: SemanticValue, cast: SemanticValue) -> SemanticValue {
        let multi = multi.into_multiplicative_expression().unwrap();
        let cast = cast.into_cast_expression().unwrap();
        SemanticValue::MultiplicativeExpression(Self::Div(Box::new(multi), cast))
    }

    pub fn make_mod(multi: SemanticValue, cast: SemanticValue) -> SemanticValue {
        let multi = multi.into_multiplicative_expression().unwrap();
        let cast = cast.into_cast_expression().unwrap();
        SemanticValue::MultiplicativeExpression(Self::Mod(Box::new(multi), cast))
    }
    pub fn make_cast(cast: SemanticValue) -> SemanticValue {
        let cast = cast.into_cast_expression().unwrap();
        SemanticValue::MultiplicativeExpression(Self::Cast(cast))
    }

}

#[derive(Debug, Clone)]
pub enum AdditiveExpression {
    Add(Box<AdditiveExpression>, MultiplicativeExpression),
    Sub(Box<AdditiveExpression>, MultiplicativeExpression),
    Mul(MultiplicativeExpression),
}

impl AdditiveExpression {
    pub fn make_add(additive: SemanticValue, multi: SemanticValue) -> SemanticValue {
        let multi = multi.into_multiplicative_expression().unwrap();
        let additive = additive.into_additive_expression().unwrap();
        SemanticValue::AdditiveExpression(Self::Add(Box::new(additive), multi))
    }

    pub fn make_sub(additive: SemanticValue, multi: SemanticValue) -> SemanticValue {
        let multi = multi.into_multiplicative_expression().unwrap();
        let additive = additive.into_additive_expression().unwrap();
        SemanticValue::AdditiveExpression(Self::Add(Box::new(additive), multi))
    }

    pub fn make_mul(multi: SemanticValue) -> SemanticValue {
        let multi = multi.into_multiplicative_expression().unwrap();
        SemanticValue::AdditiveExpression(Self::Mul(multi))
    }
}

#[derive(Debug, Clone)]
pub enum ShiftExpression {
    Shl(Box<ShiftExpression>, AdditiveExpression),
    Shr(Box<ShiftExpression>, AdditiveExpression),
    Add(AdditiveExpression),
}

impl ShiftExpression {
    pub fn make_shl(shift: SemanticValue, additive: SemanticValue) -> SemanticValue {
        let shift = shift.into_shift_expression().unwrap();
        let additive = additive.into_additive_expression().unwrap();
        SemanticValue::ShiftExpression(Self::Shl(Box::new(shift), additive))
    }

    pub fn make_shr(shift: SemanticValue, additive: SemanticValue) -> SemanticValue {
        let shift = shift.into_shift_expression().unwrap();
        let additive = additive.into_additive_expression().unwrap();
        SemanticValue::ShiftExpression(Self::Shr(Box::new(shift), additive))
    }

    pub fn make_add(additive: SemanticValue) -> SemanticValue {
        let additive = additive.into_additive_expression().unwrap();
        SemanticValue::ShiftExpression(Self::Add(additive))
    }
}

#[derive(Debug, Clone)]
pub enum RelationalExpression {
    Lt(Box<RelationalExpression>, ShiftExpression),
    Gt(Box<RelationalExpression>, ShiftExpression),
    Le(Box<RelationalExpression>, ShiftExpression),
    Ge(Box<RelationalExpression>, ShiftExpression),
    Shift(ShiftExpression),
}

impl RelationalExpression {
    pub fn make_lt(relation: SemanticValue, shift: SemanticValue) -> SemanticValue {
        let relation = relation.into_relational_expression().unwrap();
        let shift = shift.into_shift_expression().unwrap();
        SemanticValue::RelationalExpression(Self::Lt(Box::new(relation), shift))
    }
    pub fn make_gt(relation: SemanticValue, shift: SemanticValue) -> SemanticValue {
        let relation = relation.into_relational_expression().unwrap();
        let shift = shift.into_shift_expression().unwrap();
        SemanticValue::RelationalExpression(Self::Lt(Box::new(relation), shift))
    }
    pub fn make_le(relation: SemanticValue, shift: SemanticValue) -> SemanticValue {
        let relation = relation.into_relational_expression().unwrap();
        let shift = shift.into_shift_expression().unwrap();
        SemanticValue::RelationalExpression(Self::Lt(Box::new(relation), shift))
    }
    pub fn make_ge(relation: SemanticValue, shift: SemanticValue) -> SemanticValue {
        let relation = relation.into_relational_expression().unwrap();
        let shift = shift.into_shift_expression().unwrap();
        SemanticValue::RelationalExpression(Self::Lt(Box::new(relation), shift))
    }
    pub fn make_shift(shift: SemanticValue) -> SemanticValue {
        let shift = shift.into_shift_expression().unwrap();
        SemanticValue::RelationalExpression(Self::Shift(shift))
    }
}

#[derive(Debug, Clone)]
pub enum EqualityExpression {
    Eq(Box<EqualityExpression>, RelationalExpression),
    Ne(Box<EqualityExpression>, RelationalExpression),
    Rel(RelationalExpression),
}

impl EqualityExpression {
    pub fn make_eq(equ: SemanticValue, relation: SemanticValue) -> SemanticValue {
        let equ = equ.into_equality_expression().unwrap();
        let relation = relation.into_relational_expression().unwrap();
        SemanticValue::EqualityExpression(Self::Eq(Box::new(equ), relation))
    }

    pub fn make_ne(equ: SemanticValue, relation: SemanticValue) -> SemanticValue {
        let equ = equ.into_equality_expression().unwrap();
        let relation = relation.into_relational_expression().unwrap();
        SemanticValue::EqualityExpression(Self::Ne(Box::new(equ), relation))
    }

    pub fn make_rel(relation: SemanticValue) -> SemanticValue {
        let relation = relation.into_relational_expression().unwrap();
        SemanticValue::EqualityExpression(Self::Rel(relation))
    }
}

#[derive(Debug, Clone)]
pub enum AndExpression {
    And(Box<AndExpression>, EqualityExpression),
    Eq(EqualityExpression),
}

impl AndExpression {
    pub fn make_and(and: SemanticValue, equ: SemanticValue) -> SemanticValue {
        let and = and.into_and_expression().unwrap();
        let equ = equ.into_equality_expression().unwrap();
        SemanticValue::AndExpression(Self::And(Box::new(and), equ))
    }

    pub fn make_eq(equ: SemanticValue) -> SemanticValue {
        let equ = equ.into_equality_expression().unwrap();
        SemanticValue::AndExpression(Self::Eq(equ))
    }
}

#[derive(Debug, Clone)]
pub enum ExclusiveOrExpression {
    Xor(Box<ExclusiveOrExpression>, AndExpression),
    And(AndExpression),
}
impl ExclusiveOrExpression {
    pub fn make_xor(or: SemanticValue, and: SemanticValue) -> SemanticValue {
        let or = or.into_exclusive_or_expression().unwrap();
        let and = and.into_and_expression().unwrap();
        SemanticValue::ExclusiveOrExpression(Self::Xor(Box::new(or), and))
    }

    pub fn make_and(and: SemanticValue) -> SemanticValue {
        let and = and.into_and_expression().unwrap();
        SemanticValue::ExclusiveOrExpression(Self::And(and))
    }
}

#[derive(Debug, Clone)]
pub enum InclusiveOrExpression {
    Or(Box<InclusiveOrExpression>, ExclusiveOrExpression),
    Xor(ExclusiveOrExpression),
}

impl InclusiveOrExpression {
    pub fn make_or(or1: SemanticValue, or2: SemanticValue) -> SemanticValue {
        let or1 = or1.into_inclusive_or_expression().unwrap();
        let or2 = or2.into_exclusive_or_expression().unwrap();
        SemanticValue::InclusiveOrExpression(Self::Or(Box::new(or1), or2))
    }

    pub fn make_xor(or: SemanticValue) -> SemanticValue {
        let or = or.into_exclusive_or_expression().unwrap();
        SemanticValue::InclusiveOrExpression(InclusiveOrExpression::Xor(or))
    }
}

#[derive(Debug, Clone)]
pub enum LogicalAndExpression {
    And(Box<LogicalAndExpression>, InclusiveOrExpression),
    Or(InclusiveOrExpression),
}
impl LogicalAndExpression {
    pub fn make_and(and: SemanticValue, or: SemanticValue) -> SemanticValue {
        let and = and.into_logical_and_expression().unwrap();
        let or = or.into_inclusive_or_expression().unwrap();
        SemanticValue::LogicalAndExpression(Self::And(Box::new(and), or))
    }

    pub fn make_or(or: SemanticValue) -> SemanticValue {
        let or = or.into_inclusive_or_expression().unwrap();
        SemanticValue::LogicalAndExpression(LogicalAndExpression::Or(or))
    }
}

#[derive(Debug, Clone)]
pub enum LogicalOrExpression {
    Or(Box<LogicalOrExpression>, LogicalAndExpression),
    And(LogicalAndExpression),
}

impl LogicalOrExpression {
    pub fn make_or(or: SemanticValue, and: SemanticValue) -> SemanticValue {
        let or = or.into_logical_or_expression().unwrap();
        let and = and.into_logical_and_expression().unwrap();
        SemanticValue::LogicalOrExpression(Self::Or(Box::new(or), and))
    }

    pub fn make_and(and: SemanticValue) -> SemanticValue {
        let and = and.into_logical_and_expression().unwrap();
        SemanticValue::LogicalOrExpression(Self::And(and))
    }
}


#[derive(Debug, Clone)]
pub enum ConditionalExpression {
    Cond {
        cond: LogicalOrExpression,
        then_expr: Box<Expression>,
        else_expr: Box<ConditionalExpression>,
    },
    Or(LogicalOrExpression),
}

impl ConditionalExpression {
    pub fn make_cond(cond: SemanticValue, then_expr: SemanticValue, else_expr: SemanticValue) -> SemanticValue {
        let cond = cond.into_logical_or_expression().unwrap();
        let then_expr = then_expr.into_expression().unwrap();
        let else_expr = else_expr.into_conditional_expression().unwrap();
        SemanticValue::ConditionalExpression(Self::Cond{
            cond,
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr),
        })
    }

    pub fn make_or(or: SemanticValue) -> SemanticValue {
        let or = or.into_logical_or_expression().unwrap();
        SemanticValue::ConditionalExpression(Self::Or(or))
    }
}

#[derive(Debug, Clone)]
pub enum AssignmentExpression {
    Conditional(ConditionalExpression),
    Assign {
        lhs: UnaryExpression,
        op: AssignmentOperator,
        rhs: Box<AssignmentExpression>,
    },
}

impl AssignmentExpression {
    pub fn make_conditional(cond: SemanticValue) -> SemanticValue {
        let cond = cond.into_conditional_expression().unwrap();
        SemanticValue::AssignmentExpression(Self::Conditional(cond))
    }

    pub fn make_assign(lhs: SemanticValue, op: SemanticValue, rhs: SemanticValue) -> SemanticValue {
        let lhs = lhs.into_unary_expression().unwrap();
        let op = op.into_assignment_operator().unwrap();
        let rhs = rhs.into_assignment_expression().unwrap();
        SemanticValue::AssignmentExpression(Self::Assign {
            lhs,
            op,
            rhs: Box::new(rhs)
        })
    }
}

#[derive(Debug, Clone)]
pub enum AssignmentOperator {
    Assign(Token),
    MulAssign(Token),
    DivAssign(Token),
    ModAssign(Token),
    AddAssign(Token),
    SubAssign(Token),
    ShlAssign(Token),
    ShrAssign(Token),
    AndAssign(Token),
    XorAssign(Token),
    OrAssign(Token),
}

impl AssignmentOperator {
    make_zero!(AssignmentOperator, Assign, assign);
    make_zero!(AssignmentOperator, MulAssign, mul_assign);
    make_zero!(AssignmentOperator, DivAssign, div_assign);
    make_zero!(AssignmentOperator, ModAssign, mod_assign);
    make_zero!(AssignmentOperator, AddAssign, add_assign);
    make_zero!(AssignmentOperator, SubAssign, sub_assign);
    make_zero!(AssignmentOperator, ShlAssign, shl_assign);
    make_zero!(AssignmentOperator, ShrAssign, shr_assign);
    make_zero!(AssignmentOperator, AndAssign, and_assign);
    make_zero!(AssignmentOperator, XorAssign, xor_assign);
    make_zero!(AssignmentOperator, OrAssign, or_assign);
}

#[derive(Debug, Clone)]
pub enum Expression {
    Single(AssignmentExpression),
    Comma(Box<Expression>, AssignmentExpression), // expression COMMA assignment_expression
}

impl Expression {
    make_one!(into_assignment_expression, Expression, Single, make_single);
    pub fn make_comma(expr: SemanticValue, assign: SemanticValue) -> SemanticValue {
        let expr = expr.into_expression().unwrap();
        let assign = assign.into_assignment_expression().unwrap();
        SemanticValue::Expression(Expression::Comma(Box::new(expr), assign))
    }
}

#[derive(Debug, Clone)]
pub struct ConstantExpression(pub ConditionalExpression);

impl ConditionalExpression {
    pub fn make_constant(cond: SemanticValue) -> SemanticValue {
        let cond = cond.into_conditional_expression().unwrap();
        SemanticValue::ConstantExpression(ConstantExpression(cond))
    }
}

/* ===================== Type Name ===================== */

#[derive(Debug, Clone)]
pub struct TypeName {
    pub specifiers: SpecifierQualifierList,
    pub abstract_declarator: Option<AbstractDeclarator>,
}

impl TypeName {

    pub fn make_type_name(specifiers: SemanticValue, abstract_declarator: SemanticValue) -> SemanticValue {
        let specifiers = specifiers.into_specifier_qualifier_list().unwrap();
        let abstract_declarator = abstract_declarator.into_abstract_declarator_opt().unwrap();
        SemanticValue::TypeName(Self{
            specifiers,
            abstract_declarator,
        })
    }

}

/* ===================== Initializers ===================== */

#[derive(Debug, Clone)]
pub enum Initializer {
    Assignment(AssignmentExpression),
    List(InitializerList), // { init1, init2, ... }
}

impl Initializer {
    make_one!(into_assignment_expression, Initializer, Assignment, make_assignment);

    pub fn make_list(item: SemanticValue) -> SemanticValue {
        let item = item.into_initializer_list().unwrap();
        SemanticValue::Initializer(Self::List(item))
    }
}

#[derive(Debug, Clone)]
pub struct InitializerList(pub Vec<Initializer>);

impl InitializerList {

    make_struct_list!(into_initializer, InitializerList, make);
    make_struct_insert!(into_initializer_list, into_initializer, InitializerList, insert);

}

/* ===================== Abstract Declarators ===================== */

#[derive(Debug, Clone)]
pub enum AbstractDeclarator {
    Pointer(Pointer),
    Direct {
        pointer: Option<Pointer>,
        direct: DirectAbstractDeclarator,
    },
}

impl AbstractDeclarator {

    make_one!(into_pointer, AbstractDeclarator, Pointer, make_pointer);

    pub fn make_direct(pointer: SemanticValue, direct: SemanticValue) -> SemanticValue {
        let pointer = pointer.into_pointer_opt().unwrap();
        let direct = direct.into_direct_abstract_declarator().unwrap();
        SemanticValue::AbstractDeclarator(AbstractDeclarator::Direct {pointer, direct})
    }
}

#[derive(Debug, Clone)]
pub enum DirectAbstractDeclarator {
    Paren(Box<AbstractDeclarator>),
    Array(Option<ConstantExpression>),
    ArrayNested(Box<DirectAbstractDeclarator>, Option<ConstantExpression>),
    Func(Option<ParameterTypeList>),
    FuncNested(Box<DirectAbstractDeclarator>, Option<ParameterTypeList>),
}

impl DirectAbstractDeclarator {
    pub fn make_paren(decl: SemanticValue) -> SemanticValue {
        let decl = decl.into_abstract_declarator().unwrap();
        SemanticValue::DirectAbstractDeclarator(DirectAbstractDeclarator::Paren(Box::new(decl)))
    }

    pub fn make_array(constexpr: SemanticValue) -> SemanticValue {
        let constexpr = constexpr.into_constant_expression_opt().unwrap();
        SemanticValue::DirectAbstractDeclarator(DirectAbstractDeclarator::Array(constexpr))
    }

    pub fn make_array_nested(decl: SemanticValue, constexpr: SemanticValue) -> SemanticValue {
        let decl = decl.into_direct_abstract_declarator().unwrap();
        let constexpr = constexpr.into_constant_expression_opt().unwrap();
        SemanticValue::DirectAbstractDeclarator(DirectAbstractDeclarator::ArrayNested(Box::new(decl), constexpr))
    }

    pub fn make_func(param_type_list: SemanticValue) -> SemanticValue {
        let decl = param_type_list.into_parameter_type_list_opt().unwrap();
        SemanticValue::DirectAbstractDeclarator(DirectAbstractDeclarator::Func(decl))
    }

    pub fn make_func_nested(decl: SemanticValue, param_type_list_opt: SemanticValue) -> SemanticValue {
        let decl = decl.into_direct_abstract_declarator().unwrap();
        let param_type_list_opt = param_type_list_opt.into_parameter_type_list_opt().unwrap();
        SemanticValue::DirectAbstractDeclarator(DirectAbstractDeclarator::FuncNested(Box::new(decl), param_type_list_opt))
    }
}

/* ===================== Parameter Declarations ===================== */

#[derive(Debug, Clone)]
pub enum ParameterTypeList {
    Params(Vec<ParameterDeclaration>),
    Variadic(Vec<ParameterDeclaration>), // with ...
}

impl ParameterTypeList {

    make_one!(into_parameter_list, ParameterTypeList, Params, make_params);
    make_one!(into_parameter_list, ParameterTypeList, Variadic, make_variadic);
}


#[derive(Debug, Clone)]
pub enum ParameterDeclaration {
    WithDeclarator {
        specifiers: DeclarationSpecifiers,
        declarator: Declarator,
    },
    Abstract {
        specifiers: DeclarationSpecifiers,
        abstract_declarator: Option<AbstractDeclarator>,
    },
}

impl ParameterDeclaration {
    pub fn make_declarator(specifiers: SemanticValue, declarator: SemanticValue) -> SemanticValue {
        let specifiers = specifiers.into_declaration_specifiers().unwrap();
        let declarator = declarator.into_declarator().unwrap();
        SemanticValue::ParameterDeclaration(Self::WithDeclarator {
            specifiers,
            declarator
        })
    }
    pub fn make_abstract(specifiers: SemanticValue, abstract_declarator: SemanticValue) -> SemanticValue {
        let specifiers = specifiers.into_declaration_specifiers().unwrap();
        let abstract_declarator = abstract_declarator.into_abstract_declarator_opt().unwrap();
        SemanticValue::ParameterDeclaration(Self::Abstract {
            specifiers,
            abstract_declarator
        })
    }
}

/* ===================== Specifier Qualifier List ===================== */

#[derive(Debug, Clone)]
pub enum SpecifierQualifierList {
    TypeSpecifier(TypeSpecifier, Option<Box<SpecifierQualifierList>>),
    TypeQualifier(TypeQualifier, Option<Box<SpecifierQualifierList>>),
}

impl SpecifierQualifierList {
    pub fn make_type_specifier(type_specifier: SemanticValue, speci_qual_list_opt: SemanticValue) -> SemanticValue {
        let type_specifier = type_specifier.into_type_specifier().unwrap();
        let speci_qual_list_opt = speci_qual_list_opt.into_specifier_qualifier_list_opt()
            .unwrap().map(Box::new);
        SemanticValue::SpecifierQualifierList(Self::TypeSpecifier(type_specifier, speci_qual_list_opt))
    }
    pub fn make_type_qualifier(type_qualifier: SemanticValue, speci_qual_list_opt: SemanticValue) -> SemanticValue {
        let type_specifier = type_qualifier.into_type_qualifier().unwrap();
        let speci_qual_list_opt = speci_qual_list_opt.into_specifier_qualifier_list_opt()
            .unwrap().map(Box::new);
        SemanticValue::SpecifierQualifierList(Self::TypeQualifier(type_specifier, speci_qual_list_opt))
    }
}

/* ===================== Helpers ===================== */

#[derive(Debug, Clone)]
pub struct IdentifierList(pub Vec<Token>);

impl IdentifierList {
    make_struct_list!(into_token, IdentifierList, make_list);
    make_struct_insert!(into_identifier_list, into_token, IdentifierList, insert);
}

pub fn list_push<T>(mut list: Vec<T>, item: T) -> Vec<T> {
    list.push(item);
    list
}

make_list!(into_struct_declaration, StructDeclarationList, make_struct_declaration_list);
make_insert!(into_struct_declaration_list, into_struct_declaration, StructDeclarationList, insert_struct_declaration_list);

make_list!(into_type_qualifier, TypeQualifierList, make_type_qualifier_list);
make_insert!(into_type_qualifier_list, into_type_qualifier, TypeQualifierList, insert_type_qualifier_list);


make_list!(into_struct_declarator, StructDeclaratorList, make_struct_declarator_list);
make_insert!(into_struct_declarator_list, into_struct_declarator, StructDeclaratorList, insert_struct_declarator_list);

make_list!(into_enumerator, EnumeratorList, make_enumerator_list);
make_insert!(into_enumerator_list, into_enumerator, EnumeratorList, insert_enumerator_list);

make_list!(into_parameter_declaration, ParameterList, make_parameter_list);
make_insert!(into_parameter_list, into_parameter_declaration, ParameterList, insert_parameter_list);

make_list!(into_block_item, BlockItemList, make_block_item);
make_insert!(into_block_item_list, into_block_item, BlockItemList, insert_block_item);

make_list!(into_token, String, make_string);
make_insert!(into_string, into_token, String, insert_string);

make_list!(into_assignment_expression, ArgumentExpressionList, makeargument_expression_list);
make_insert!(into_argument_expression_list, into_assignment_expression, ArgumentExpressionList, insert_argument_expression_list);

make_opt!(into_declaration_list, DeclarationListOpt, make_declaration_list_opt);
make_opt!(into_init_declarator_list, InitDeclaratorListOpt, make_init_declarator_list_opt);
make_opt!(into_declaration_specifiers, DeclarationSpecifiersOpt, make_declarator_list_opt);
make_opt!(into_pointer, PointerOpt, make_pointer_opt);
make_opt!(into_token, IdentifierOpt, make_identifier_opt);
make_opt!(into_specifier_qualifier_list, SpecifierQualifierListOpt, make_specifier_qualifier_list_opt);
make_opt!(into_constant_expression, ConstantExpressionOpt, make_constant_expression_opt);
make_opt!(into_identifier_list, IdentifierListOpt, make_identifier_list_opt);
make_opt!(into_abstract_declarator, AbstractDeclaratorOpt, make_abstract_declarator_opt);
make_opt!(into_parameter_type_list, ParameterTypeListOpt, make_parameter_type_list_opt);
make_opt!(into_expression, ExpressionOpt, make_expression_opt);
make_opt!(into_argument_expression_list, ArgumentExpressionListOpt, make_argument_expression_list_opt);

