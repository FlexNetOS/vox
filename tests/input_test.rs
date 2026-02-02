use vox::input::read_text;

#[test]
fn test_read_text_from_args() {
    let args = vec!["Hello".into(), "world".into()];
    let result = read_text(&args).unwrap();
    assert_eq!(result, "Hello world");
}

#[test]
fn test_read_text_single_arg() {
    let args = vec!["Bonjour".into()];
    let result = read_text(&args).unwrap();
    assert_eq!(result, "Bonjour");
}

#[test]
fn test_read_text_empty_args_is_error() {
    // With no args and a terminal stdin, this should error.
    // In test context stdin is a terminal, so empty args = error.
    let args: Vec<String> = vec![];
    let result = read_text(&args);
    assert!(result.is_err());
}

#[test]
fn test_read_text_whitespace_only_is_error() {
    let args = vec!["   ".into()];
    let result = read_text(&args);
    assert!(result.is_err());
}
