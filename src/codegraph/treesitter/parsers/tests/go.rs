#[cfg(test)]
mod tests {
    use std::fs::canonicalize;
    use std::path::PathBuf;

    use crate::codegraph::treesitter::language_id::LanguageId;
    use crate::codegraph::treesitter::parsers::AstLanguageParser;
    use crate::codegraph::treesitter::parsers::go::GoParser;
    use crate::codegraph::treesitter::parsers::tests::{base_declaration_formatter_test, base_parser_test, base_skeletonizer_test};

    const MAIN_GO_CODE: &str = include_str!("cases/go/main.go");
    const MAIN_GO_SYMBOLS: &str = include_str!("cases/go/main.go.json");

    const SHAPE_GO_CODE: &str = include_str!("cases/go/shape.go");
    const SHAPE_GO_SKELETON: &str = include_str!("cases/go/shape.go.skeleton");
    const SHAPE_GO_DECLS: &str = include_str!("cases/go/shape.go.decl_json");

    #[test]
    fn parser_test() {
        let mut parser: Box<dyn AstLanguageParser> = Box::new(GoParser::new().expect("GoParser::new"));
        let path = PathBuf::from("/main.go");
        base_parser_test(&mut parser, &path, MAIN_GO_CODE, MAIN_GO_SYMBOLS);
    }

    #[test]
    fn skeletonizer_test() {
        let mut parser: Box<dyn AstLanguageParser> = Box::new(GoParser::new().expect("GoParser::new"));
        let file = canonicalize(PathBuf::from(file!())).unwrap().parent().unwrap().join("cases/go/shape.go");
        assert!(file.exists());

        base_skeletonizer_test(&LanguageId::Go, &mut parser, &file, SHAPE_GO_CODE, SHAPE_GO_SKELETON);
    }

    #[test]
    fn declaration_formatter_test() {
        let mut parser: Box<dyn AstLanguageParser> = Box::new(GoParser::new().expect("GoParser::new"));
        let file = canonicalize(PathBuf::from(file!())).unwrap().parent().unwrap().join("cases/go/shape.go");
        assert!(file.exists());
        base_declaration_formatter_test(&LanguageId::Go, &mut parser, &file, SHAPE_GO_CODE, SHAPE_GO_DECLS);
    }
} 