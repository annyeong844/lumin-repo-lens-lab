mod declarations;
mod normalized;
mod type_escape;
mod warning;

pub(in crate::prewrite) use declarations::{
    DependencyDeclaration, NameDeclaration, RefactorSource, ShapeIntent,
};
pub(in crate::prewrite) use normalized::{LoadedIntent, NormalizedIntent};
pub(in crate::prewrite) use type_escape::PlannedTypeEscape;
pub(in crate::prewrite) use warning::{IntentKey, IntentWarning};
