use super::super::ast::*;
use super::Sema;

impl Sema {
    pub(crate) fn alignof(&self, t: &CType) -> usize {
        self.layout(t).1
    }
    pub(crate) fn layout(&self, t: &CType) -> (usize, usize) {
        let bytes = |bits: u16| usize::from(bits.div_ceil(8));
        match &t.kind {
            TypeKind::Bool | TypeKind::Char { .. } => {
                let size = bytes(self.target.char_width);
                (size, size)
            }
            TypeKind::Short { .. } => {
                let size = bytes(self.target.short_width);
                (size, size)
            }
            TypeKind::Int { .. } | TypeKind::Float | TypeKind::Enum { .. } => {
                let size = bytes(self.target.int_width);
                (size, size)
            }
            TypeKind::Long { .. } => {
                let size = bytes(self.target.long_width);
                (size, size)
            }
            TypeKind::LongLong { .. } | TypeKind::Double => {
                let size = bytes(self.target.long_long_width);
                (size, size)
            }
            TypeKind::Pointer(_) => (
                bytes(self.target.pointer_width),
                bytes(self.target.pointer_align),
            ),
            TypeKind::LongDouble => (16, 16),
            TypeKind::Complex(inner) | TypeKind::Imaginary(inner) => {
                let (size, align) = self.layout(inner);
                (size * 2, align)
            }
            TypeKind::Array { element, size } => {
                let (element_size, align) = self.layout(element);
                let count = match size {
                    ArraySize::Constant(size) => *size,
                    _ => 0,
                };
                (element_size * count, align)
            }
            TypeKind::Struct {
                fields: Some(fields),
                ..
            } => {
                let mut offset = 0;
                let mut aggregate_align = 1;
                for field in fields {
                    let (size, align) = self.layout(&field.ty);
                    aggregate_align = aggregate_align.max(align);
                    offset = align_up(offset, align);
                    offset += size;
                }
                (align_up(offset, aggregate_align), aggregate_align)
            }
            TypeKind::Union {
                fields: Some(fields),
                ..
            } => {
                let size = fields
                    .iter()
                    .map(|field| self.layout(&field.ty).0)
                    .max()
                    .unwrap_or(0);
                let align = fields
                    .iter()
                    .map(|field| self.layout(&field.ty).1)
                    .max()
                    .unwrap_or(1);
                (align_up(size, align), align)
            }
            _ => (0, 1),
        }
    }
}

fn align_up(value: usize, alignment: usize) -> usize {
    value.div_ceil(alignment) * alignment
}
