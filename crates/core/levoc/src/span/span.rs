#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CtxId(u32);

impl CtxId {
    pub const ROOT: CtxId = CtxId::new(0);

    pub const fn new(handle: u32) -> Self {
        Self(handle)
    }

    pub const fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn is_root(&self) -> bool {
        *self == Self::ROOT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    start: u32,
    len: u32,
    ctx: CtxId,
}

impl Span {
    pub fn new(start: u32, len: u32, ctx: CtxId) -> Self {
        Self { start, len, ctx }
    }
}

impl Span {
    pub fn start(&self) -> u32 {
        self.start
    }

    pub fn end(&self) -> u32 {
        self.start + self.len
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn ctx(&self) -> CtxId {
        self.ctx
    }
}
