struct ASTWalker<T: ASTVisitor> {
    visitor: T,
}

impl<T: ASTVisitor> ASTWalker<T> {
    fn new(visitor: T) -> Self {
        ASTWalker { visitor }
    }
}


trait ASTVisitor {

}

