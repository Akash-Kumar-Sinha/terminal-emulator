use akssora_terminal_config::{Color, Theme};
use akssora_terminal_parser::TerminalParser;
use akssora_terminal_screen::TerminalGrid;
use akssora_terminal_ui::{Header, HeaderButton};

#[test]
fn test_terminal_screen_and_parser_integration() {
    let mut grid = TerminalGrid::new(24, 80);
    let mut parser = TerminalParser::new();

    parser.advance(&mut grid, b"Hello, Akssora!\r\n");
    {
        let lines = grid.visible_lines();
        let first_line_chars: String = lines[0].iter().map(|c| c.ch).collect();
        assert!(first_line_chars.starts_with("Hello, Akssora!"));
    }

    parser.advance(&mut grid, b"\x1b[31mRed\x1b[0m");
    {
        let lines = grid.visible_lines();
        let second_line = lines[1];
        assert_eq!(second_line[0].ch, 'R');
        assert_eq!(second_line[0].fg, Color::Indexed(1));
        assert_eq!(second_line[1].ch, 'e');
        assert_eq!(second_line[2].ch, 'd');
    }
}

#[test]
fn test_header_ui_hit_test() {
    let header = Header::new("Akssora");
    assert_eq!(header.title, "Akssora");

    let window_w = 1000.0;
    let close_btn = header.hit_test(window_w - 10.0, 10.0, window_w);
    assert_eq!(close_btn, Some(HeaderButton::Close));

    assert!(header.in_drag_region(500.0, 10.0, window_w));
}

#[test]
fn test_theme_color_resolution() {
    let theme = Theme::modern_dark();
    assert_eq!(theme.resolve_fg(Color::Default, false), theme.foreground);
    assert_eq!(theme.resolve_fg(Color::Indexed(1), false), theme.ansi[1]);
    assert_eq!(theme.resolve_bg(Color::Default), None);
}
