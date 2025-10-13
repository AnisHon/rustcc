use crate::err::parser_error::ParserResult;
use crate::lex::types::token_kind::{Keyword, TokenKind};
use crate::parser::parser_core::Parser;
use crate::parser::types::common::Ident;
use crate::parser::types::sema::decl_spec::DeclSpecChunk;

impl Parser {


    pub(crate) fn parse_decl(&mut self, chunks: Vec<DeclSpecChunk>, declarator: ()) -> ParserResult<()> {
        let init_list = self.parse_init_declarator_list(declarator)?;
        let semi = self.expect(TokenKind::Semi)?;
        todo!()
    }

    pub(crate) fn parse_decl_spec(&mut self) -> ParserResult<Vec<DeclSpecChunk>> {
        let mut decl_specs = Vec::new();
        loop {
            let token = self.stream.peek();
            let chunk = if self.is_storage_spec(token) {
                self.parse_storage_spec()?
            } else if self.is_type_spec(token) {
                self.parse_type_spec()?
            } else if self.is_type_qual(token) {
                self.parse_type_qual()?
            } else if self.check_keyword(Keyword::Inline) {
                // inline
                self.parse_function_spec()?
            } else {
                break
            };
            decl_specs.push(chunk)
        }
        Ok(decl_specs)
    }

    fn parse_storage_spec(&mut self) -> ParserResult<DeclSpecChunk> {
        todo!()
    }
    fn parse_type_spec(&mut self) -> ParserResult<DeclSpecChunk> {
        todo!()
    }
    fn parse_type_qual(&mut self) -> ParserResult<DeclSpecChunk> {
        todo!()
    }

    fn parse_function_spec(&mut self) -> ParserResult<DeclSpecChunk> {
        todo!()
    }

    fn parse_init_declarator_list(&mut self, declarator: ()) -> ParserResult<Vec<()>> {
        let init = self.parse_init_declarator(Some(declarator))?;
        let mut init_list = vec![init];

        while let Some(comma) = self.consume(TokenKind::Comma) {
            let init = self.parse_init_declarator(None)?;
            init_list.push(init);
        }
        Ok(init_list)
    }

    /// 已经预先解析一个declarator了
    fn parse_init_declarator(&mut self, declarator: Option<()>) -> ParserResult<()> {
        let declarator = match declarator {
            None => self.parse_declarator()?,
            Some(x) => x
        };
        let assign;
        let init;
        if let Some(assign_token) = self.consume(TokenKind::Assign) {
            assign = Some(assign_token.span);
            init = Some(self.parse_initializer()?);
        } else {
            assign = None;
            init = None;
        }
        todo!()
    }

    fn parse_struct_or_union_spec(&mut self) -> ParserResult<()> {
        let kw = self.expect_keyword_pair(Keyword::Struct, Keyword::Union)?;
        if let Some(lbrace) = self.consume(TokenKind::LBrace) {
            let rbrace = self.expect(TokenKind::RBrace)?;
        } else if let Some(ident) = self.consume_ident() {

        } else {
            // return Err()
        }

        todo!()
    }

    fn parse_struct_decl_list(&mut self) -> ParserResult<Vec<()>> {
        let mut decls = Vec::new();
        if self.check(TokenKind::RBrace) {
            return Ok(decls);
        }

        loop {
            let decl = self.parse_struct_decl()?;
            decls.push(decl);
            if self.check(TokenKind::RBrace) {
                break;
            }
        }
        Ok(decls)
    }

    fn parse_struct_decl(&mut self) -> ParserResult<()> {
        todo!()
    }

    fn parse_spec_qual_list(&mut self) -> ParserResult<Vec<DeclSpecChunk>> {
        let mut decl_specs = Vec::new();
        loop {
            let token = self.stream.peek();
            let chunk = if self.is_type_spec(token) {
                self.parse_type_spec()?
            } else if self.is_type_qual(token) {
                self.parse_type_qual()?
            } else {
                break
            };
            decl_specs.push(chunk)
        }
        Ok(decl_specs)
    }

    fn parse_enum_spec(&mut self) -> ParserResult<()> {
        let kw = self.expect_keyword(Keyword::Enum)?;
        if let Some(lbrace) = self.consume(TokenKind::LBrace) {
            let enumerator = self.parse_enumerator_list();
            let rbrace = self.expect(TokenKind::RBrace)?;
        } else if let Some(ident) = self.consume_ident() {

        } else {
            // return Err()
        }
        todo!()
    }

    fn parse_enumerator_list(&mut self) -> ParserResult<Vec<()>> {
        todo!()
    }

    fn parse_enumerator(&mut self) -> ParserResult<Vec<()>> {
        let ident = self.expect_ident()?;
        let assign;
        let expr;
        if let Some(assign_token) = self.consume(TokenKind::Assign) {
            assign = Some(assign_token);
            expr = Some(self.parse_expr()?);
        } else {
            assign = None;
            expr = None;
        }

        todo!()
    }

    pub(crate) fn parse_declarator(&mut self) -> ParserResult<()> {
        todo!()
    }

    fn parse_direct_declarator(&mut self) -> ParserResult<()> {
        todo!()
    }

    fn parse_pointer(&mut self) -> ParserResult<()> {
        todo!()
    }

    fn parse_type_qual_list(&mut self) -> ParserResult<Vec<()>> {
        todo!()
    }

    fn parse_parameter_type_list(&mut self) -> ParserResult<()> {
        todo!()
    }

    fn parse_parameter_decl(&mut self) -> ParserResult<()> {
        todo!()
    }

    fn parse_ident_list(&mut self) -> ParserResult<Vec<Ident>> {
        todo!()
    }

    pub(crate) fn parse_type_name(&mut self) -> ParserResult<()> {
        // todo
        self.stream.next();
        Ok(())
    }

    fn parse_abstract_declarator(&mut self) -> ParserResult<()> {
        todo!()
    }

    fn parse_direct_abstract_declarator(&mut self) -> ParserResult<()> {
        todo!()
    }

    fn parse_initializer(&mut self) -> ParserResult<()> {
        todo!()
    }
}