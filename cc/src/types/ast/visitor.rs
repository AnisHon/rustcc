#[allow(dead_code)]
struct ASTWalker<T: ASTVisitor> {
    visitor: T,
}

#[allow(dead_code)]
impl<T: ASTVisitor> ASTWalker<T> {
    fn new(visitor: T) -> Self {
        ASTWalker { visitor }
    }
}

#[allow(dead_code)]
trait ASTVisitor {

}

