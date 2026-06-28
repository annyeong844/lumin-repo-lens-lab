use crate::protocol::AstVisibility;

use ra_ap_syntax::{ast, AstNode};

pub(in crate::analyzer) fn visibility_for(visibility: Option<ast::Visibility>) -> AstVisibility {
    let Some(visibility) = visibility else {
        return AstVisibility::Private;
    };
    let text = visibility.syntax().text().to_string();
    match text.as_str() {
        "pub" => AstVisibility::Public,
        "pub(crate)" => AstVisibility::Crate,
        value if value.starts_with("pub(") => AstVisibility::Restricted,
        _ => AstVisibility::Unknown,
    }
}
