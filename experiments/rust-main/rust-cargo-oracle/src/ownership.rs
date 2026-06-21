mod paths;
mod resolver;
#[cfg(test)]
mod tests;

pub(crate) use resolver::OwnershipResolver;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SpanClass {
    UserCode,
    Dependency,
    Generated,
    Unknown,
}
