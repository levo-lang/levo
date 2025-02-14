use super::cursor::Cursor;

#[test]
fn lexer_test() {
    let inputs = [
        "",
        "  ",
        "a + b",
        "a(c)",
        "+++*/",
        "/**/",
        "/***/",
        "/*!*/",
        "/*/**/*/",
        "/*",
        "/*/*",
        "/*/",
        "//",
        "// \n",
        "// a\na",
        "_abc234 + 0x2323_adcs / 0o32.3rd * 32.3e+2 + 32.3e-2 - 11e7",
        "0x + 1e + 1.a",
        "'a'",
        "\"bcsdcs\"xdcs",
        "\"\n\"",
        "' xsxscsc\r\n'",
    ];

    for (num, text) in inputs.into_iter().enumerate() {
        println!("===== {num} =====");
        let mut lexer = Cursor::new(text);
        std::iter::from_fn(|| lexer.next_token()).for_each(|token| println!("{token:?}"));
    }
}
