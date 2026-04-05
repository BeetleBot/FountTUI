use super::*;
use crate::app::{App, AppMode, BufferState};
use crate::types::LineType;

use ratatui::style::{Color, Modifier};
use crossterm::event::{KeyCode, KeyModifiers};

    fn create_empty_app() -> App {
        let mut app = App::new(crate::config::Cli::default());
        app.config = crate::config::Config::default();
        
        // Tests expect an initial empty buffer in Normal mode
        let buf = BufferState {
            lines: vec![String::new()],
            ..Default::default()
        };
        app.buffers.push(buf);
        app.switch_buffer(0);
        
        app.mode = AppMode::Normal;
        app.update_layout();
        app
    }


    #[test]
    fn test_app_initialization() {
        let app = create_empty_app();
        assert_eq!(app.lines.len(), 1);
        assert_eq!(app.cursor_y, 0);
        assert_eq!(app.cursor_x, 0);
        assert!(!app.dirty);
        assert!(app.mode == AppMode::Normal);
    }

    #[test]
    fn test_app_move_down() {
        let mut app = create_empty_app();
        app.lines = vec!["Line 1".to_string(), "Line 2".to_string()];
        app.parse_document();
        app.update_layout();
        app.move_down();
        assert_eq!(app.cursor_y, 1);
    }

    #[test]
    fn test_app_move_up() {
        let mut app = create_empty_app();
        app.lines = vec!["Line 1".to_string(), "Line 2".to_string()];
        app.cursor_y = 1;
        app.parse_document();
        app.update_layout();
        app.move_up();
        assert_eq!(app.cursor_y, 0);
    }

    #[test]
    fn test_app_move_right() {
        let mut app = create_empty_app();
        app.lines = vec!["123".to_string(), "456".to_string()];
        app.move_right();
        assert_eq!(app.cursor_x, 1);
        app.move_right();
        app.move_right();
        assert_eq!(app.cursor_x, 3);
        app.move_right();
        assert_eq!(app.cursor_y, 1);
        assert_eq!(app.cursor_x, 0);
    }

    #[test]
    fn test_app_move_left() {
        let mut app = create_empty_app();
        app.lines = vec!["123".to_string(), "456".to_string()];
        app.cursor_y = 1;
        app.cursor_x = 0;
        app.move_left();
        assert_eq!(app.cursor_y, 0);
        assert_eq!(app.cursor_x, 3);
        app.move_left();
        assert_eq!(app.cursor_x, 2);
    }

    #[test]
    fn test_app_move_word_right() {
        let mut app = create_empty_app();
        app.lines = vec!["Word one two".to_string()];
        app.move_word_right();
        assert_eq!(app.cursor_x, 4);
        app.move_word_right();
        assert_eq!(app.cursor_x, 8);
    }

    #[test]
    fn test_app_move_word_left() {
        let mut app = create_empty_app();
        app.lines = vec!["Word one two".to_string()];
        app.cursor_x = 9;
        app.move_word_left();
        assert_eq!(app.cursor_x, 5);
        app.move_word_left();
        assert_eq!(app.cursor_x, 0);
    }

    #[test]
    fn test_app_move_home_and_end() {
        let mut app = create_empty_app();
        app.lines = vec!["End of line".to_string()];
        app.move_end();
        assert_eq!(app.cursor_x, 11);
        app.move_home();
        assert_eq!(app.cursor_x, 0);
    }

    #[test]
    fn test_app_insert_char() {
        let mut app = create_empty_app();
        app.insert_char('A');
        assert_eq!(app.lines[0], "A");
        assert_eq!(app.cursor_x, 1);
        assert!(app.dirty);
    }

    #[test]
    fn test_app_insert_matching_parentheses() {
        let mut app = create_empty_app();
        app.insert_char('(');
        assert_eq!(app.lines[0], "()");
        assert_eq!(app.cursor_x, 1);
    }

    #[test]
    fn test_app_insert_matching_brackets() {
        let mut app = create_empty_app();
        app.insert_char('[');
        app.insert_char('[');
        assert_eq!(app.lines[0], "[[]]");
        assert_eq!(app.cursor_x, 2);
    }

    #[test]
    fn test_app_insert_matching_boneyard() {
        let mut app = create_empty_app();
        app.insert_char('/');
        app.insert_char('*');
        assert_eq!(app.lines[0], "/**/");
        assert_eq!(app.cursor_x, 2);
    }

    #[test]
    fn test_app_backspace() {
        let mut app = create_empty_app();
        app.lines = vec!["A".to_string()];
        app.cursor_x = 1;
        app.backspace();
        assert_eq!(app.lines[0], "");
        assert_eq!(app.cursor_x, 0);
    }

    #[test]
    fn test_app_backspace_matching_brackets() {
        let mut app = create_empty_app();
        app.lines = vec!["[[]]".to_string()];
        app.cursor_x = 2;
        app.backspace();
        assert_eq!(app.lines[0], "");
        assert_eq!(app.cursor_x, 0);
    }

    #[test]
    fn test_app_backspace_merge_lines() {
        let mut app = create_empty_app();
        app.lines = vec!["A".to_string(), "B".to_string()];
        app.cursor_y = 1;
        app.cursor_x = 0;
        app.backspace();
        assert_eq!(app.lines.len(), 1);
        assert_eq!(app.lines[0], "AB");
        assert_eq!(app.cursor_y, 0);
        assert_eq!(app.cursor_x, 1);
    }

    #[test]
    fn test_app_delete_forward() {
        let mut app = create_empty_app();
        app.lines = vec!["AB".to_string()];
        app.cursor_x = 0;
        app.delete_forward();
        assert_eq!(app.lines[0], "B");
        assert_eq!(app.cursor_x, 0);
    }

    #[test]
    fn test_app_delete_forward_merge_lines() {
        let mut app = create_empty_app();
        app.lines = vec!["A".to_string(), "B".to_string()];
        app.cursor_x = 1;
        app.delete_forward();
        assert_eq!(app.lines.len(), 1);
        assert_eq!(app.lines[0], "AB");
    }

    #[test]
    fn test_app_delete_word_back() {
        let mut app = create_empty_app();
        app.lines = vec!["One Two".to_string()];
        app.cursor_x = 7;
        app.delete_word_back();
        assert_eq!(app.lines[0], "One ");
        assert_eq!(app.cursor_x, 4);
    }

    #[test]
    fn test_app_delete_word_forward() {
        let mut app = create_empty_app();
        app.lines = vec!["One Two".to_string()];
        app.cursor_x = 0;
        app.delete_word_forward();
        assert_eq!(app.lines[0], " Two");
        assert_eq!(app.cursor_x, 0);
    }

    #[test]
    fn test_app_insert_newline() {
        let mut app = create_empty_app();
        app.lines = vec!["AB".to_string()];
        app.cursor_x = 1;
        app.insert_newline(false);
        assert_eq!(app.lines.len(), 2);
        assert_eq!(app.lines[0], "A");
        assert_eq!(app.lines[1], "B");
        assert_eq!(app.cursor_y, 1);
        assert_eq!(app.cursor_x, 0);
    }

    #[test]
    fn test_app_insert_newline_auto_paragraph_breaks() {
        let mut app = create_empty_app();
        app.config.auto_paragraph_breaks = true;
        app.lines = vec!["Action line.".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_x = 12;
        app.insert_newline(false);
        assert_eq!(app.lines.len(), 3);
        assert_eq!(app.lines[0], "Action line.");
        assert_eq!(app.lines[1], "");
        assert_eq!(app.lines[2], "");
        assert_eq!(app.cursor_y, 2);
    }

    #[test]
    fn test_app_insert_newline_smart_element_escape() {
        let mut app = create_empty_app();
        app.lines = vec!["CHARLOTTE".to_string()];
        app.types = vec![LineType::Character];
        app.cursor_x = 9;
        app.insert_newline(false);
        assert_eq!(app.lines.len(), 2);
        assert_eq!(app.lines[0], "CHARLOTTE");
        assert_eq!(app.lines[1], "");
        assert_eq!(app.cursor_y, 1);
    }

    #[test]
    fn test_app_undo_redo_stack() {
        let mut app = create_empty_app();
        app.lines = vec!["Initial".to_string()];
        app.save_state(true);
        app.lines = vec!["Changed".to_string()];
        app.undo();
        assert_eq!(app.lines[0], "Initial");
        app.redo();
        assert_eq!(app.lines[0], "Changed");
    }

    #[test]
    fn test_app_cut_and_paste() {
        let mut app = create_empty_app();
        app.lines = vec!["Line 1".to_string(), "Line 2".to_string()];
        app.cut_line();
        assert_eq!(app.lines.len(), 1);
        assert_eq!(app.lines[0], "Line 2");
        app.paste_line();
        assert_eq!(app.lines.len(), 2);
        assert_eq!(app.lines[0], "Line 1");
        assert_eq!(app.lines[1], "Line 2");
    }

    #[test]
    fn test_app_cut_append_buffer() {
        let mut app = create_empty_app();
        app.lines = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        app.cut_line();
        app.cut_line();
        assert_eq!(app.cut_buffer, Some("A\nB".to_string()));
    }

    #[test]
    fn test_app_search_forward() {
        let mut app = create_empty_app();
        app.lines = vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()];
        app.search_query = "eta".to_string();
        app.execute_search();
        assert_eq!(app.cursor_y, 1);
        assert_eq!(app.cursor_x, 1);
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn test_app_search_wrap_around() {
        let mut app = create_empty_app();
        app.lines = vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()];
        app.cursor_y = 2;
        app.search_query = "lph".to_string();
        app.execute_search();
        assert_eq!(app.cursor_y, 0);
        assert_eq!(app.cursor_x, 1);
    }

    #[test]
    fn test_app_search_regex_not_found() {
        let mut app = create_empty_app();
        app.lines = vec!["World".to_string()];
        app.search_query = "god".to_string();

        app.execute_search();

        assert_eq!(app.cursor_y, 0, "Cursor should not move");
        assert_eq!(app.status_msg.as_deref(), Some("\"god\" not found"));
        assert!(
            !app.show_search_highlight,
            "Highlight should be disabled if not found"
        );
    }

    #[test]
    fn test_app_tab_state_machine_empty_to_char() {
        let mut app = create_empty_app();
        app.lines = vec!["".to_string()];
        app.types = vec![LineType::Empty];
        app.handle_tab();
        assert_eq!(app.lines[0], "@");
        assert_eq!(app.cursor_x, 1);
    }

    #[test]
    fn test_app_tab_state_machine_char_to_scene() {
        let mut app = create_empty_app();
        app.lines = vec!["@".to_string()];
        app.types = vec![LineType::Character];
        app.cursor_x = 1;
        app.handle_tab();
        assert_eq!(app.lines[0], ".");
        assert_eq!(app.cursor_x, 1);
    }

    #[test]
    fn test_app_tab_state_machine_scene_to_transition() {
        let mut app = create_empty_app();
        app.lines = vec![".".to_string()];
        app.types = vec![LineType::SceneHeading];
        app.cursor_x = 1;
        app.handle_tab();
        assert_eq!(app.lines[0], ">");
        assert_eq!(app.cursor_x, 1);
    }

    #[test]
    fn test_app_tab_state_machine_transition_to_empty() {
        let mut app = create_empty_app();
        app.lines = vec![">".to_string()];
        app.types = vec![LineType::Transition];
        app.cursor_x = 1;
        app.handle_tab();
        assert_eq!(app.lines[0], "");
        assert_eq!(app.cursor_x, 0);
    }

    #[test]
    fn test_app_tab_state_machine_after_dialogue_is_paren() {
        let mut app = create_empty_app();
        app.lines = vec!["CHARLOTTE".to_string(), "".to_string()];
        app.types = vec![LineType::Character, LineType::Empty];
        app.cursor_y = 1;
        app.handle_tab();
        assert_eq!(app.lines[1], "()");
        assert_eq!(app.cursor_x, 1);
    }

    #[test]
    fn test_app_tab_dialogue_wrap() {
        let mut app = create_empty_app();
        app.lines = vec!["CHARLOTTE".to_string(), "speaking".to_string()];
        app.types = vec![LineType::Character, LineType::Dialogue];
        app.cursor_y = 1;
        app.handle_tab();
        assert_eq!(app.lines[1], "(speaking)");
    }

    #[test]
    fn test_app_tab_strip_forced_markers() {
        let mut app = create_empty_app();
        app.lines = vec!["!Force".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_x = 6;
        app.handle_tab();
        assert_eq!(app.lines[0], "Force");
        assert_eq!(app.cursor_x, 5);
    }

    #[test]
    fn test_app_autocomplete_character() {
        let mut app = create_empty_app();
        app.lines = vec!["@CHA".to_string()];
        app.cursor_y = 0;
        app.cursor_x = 4;
        app.characters.insert("CHARLOTTE C.".to_string());
        app.update_autocomplete();
        assert_eq!(app.suggestion, Some("RLOTTE C.".to_string()));
    }

    #[test]
    fn test_app_autocomplete_scene_heading() {
        let mut app = create_empty_app();
        app.lines = vec![
            "INT. BIG ROOM - DAY".to_string(),
            "".to_string(),
            "INT. BI".to_string(),
        ];
        app.cursor_y = 2;
        app.cursor_x = 7;
        app.parse_document();
        app.update_autocomplete();
        assert_eq!(app.suggestion, Some("G ROOM - DAY".to_string()));
    }

    #[test]
    fn test_app_utf8_cursor_navigation_and_deletion() {
        let mut app = create_empty_app();

        app.lines = vec!["Привет, мир!".to_string()];
        app.cursor_y = 0;
        app.cursor_x = 7;

        app.backspace();

        assert_eq!(app.lines[0], "Привет мир!");
        assert_eq!(app.cursor_x, 6);

        app.backspace();
        assert_eq!(app.lines[0], "Приве мир!");
        assert_eq!(app.cursor_x, 5);
    }

    #[test]
    fn test_app_word_navigation_utf8() {
        let mut app = create_empty_app();
        app.lines = vec!["Сценарий номер один".to_string()];
        app.cursor_y = 0;
        app.cursor_x = 0;

        app.move_word_right();
        assert_eq!(app.cursor_x, 8);

        app.move_word_right();
        assert_eq!(app.cursor_x, 14);

        app.move_word_left();
        assert_eq!(app.cursor_x, 9);
    }

    #[test]
    fn test_app_auto_title_page_enabled() {
        let mut cli = crate::config::Cli::default();
        cli.auto_title_page = true;
        // Logic only triggers if files are provided
        cli.files = vec![PathBuf::from("new_script.fountain")];

        let app = App::new(cli);
        assert!(
            app.lines.len() > 1,
            "Title page should generate multiple lines"
        );
        assert_eq!(
            app.lines[0], "Title: Untitled",
            "First line must be Title metadata"
        );
        assert!(
            app.dirty,
            "App should be marked dirty after generating title page"
        );
    }

    #[test]
    fn test_app_auto_title_page_disabled() {
        let mut cli = crate::config::Cli::default();
        cli.auto_title_page = false;
        cli.files = vec![PathBuf::from("new_script.fountain")];

        let app = App::new(cli);
        assert_eq!(
            app.lines.len(),
            1,
            "Should only have one line"
        );
    }

    #[test]
    fn test_app_autocomplete_disabled() {
        let mut app = create_empty_app();
        app.config.autocomplete = false;

        app.lines = vec!["@CHA".to_string()];
        app.cursor_y = 0;
        app.cursor_x = 4;
        app.characters.insert("CHARLOTTE C.".to_string());

        app.update_autocomplete();
        assert_eq!(
            app.suggestion, None,
            "Suggestion should be None when disabled"
        );
    }

    #[test]
    fn test_app_auto_paragraph_breaks_disabled() {
        let mut app = create_empty_app();
        app.config.auto_paragraph_breaks = false;

        app.lines = vec!["Action line.".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_x = 12;

        app.insert_newline(false);

        assert_eq!(app.lines.len(), 2, "Should only insert 1 newline");
        assert_eq!(app.lines[1], "");
        assert_eq!(app.cursor_y, 1);
    }

    #[test]
    fn test_draw_focus_mode_hides_panels() {
        use ratatui::{Terminal, backend::TestBackend};
        let mut app = create_empty_app();
        app.config.focus_mode = true;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| crate::app::ui::draw(f, &mut app)).unwrap();

        let mut content = String::new();
        let buffer = terminal.backend().buffer();
        for y in 0..24u16 {
            for x in 0..80u16 {
                content.push_str(buffer[(x, y)].symbol());
            }
        }

        assert!(
            !content.contains("fount"),
            "Top panel should be hidden in focus mode"
        );
        assert!(
            !content.contains("^X"),
            "Bottom panel should be hidden in focus mode"
        );
    }

    #[test]
    fn test_draw_focus_mode_shows_prompt() {
        use ratatui::{Terminal, backend::TestBackend};
        let mut app = create_empty_app();
        app.config.focus_mode = true;
        app.mode = AppMode::PromptSave;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| crate::app::ui::draw(f, &mut app)).unwrap();

        let mut content = String::new();
        let buffer = terminal.backend().buffer();
        for y in 0..24u16 {
            for x in 0..80u16 {
                content.push_str(buffer[(x, y)].symbol());
            }
        }

        assert!(
            content.contains("SAVE") && content.contains("MODIFIED"),
            "Prompt should appear even in focus mode. Content: {}", content
        );
    }

    #[test]
    fn test_draw_focus_mode_shows_status_msg() {
        use ratatui::{Terminal, backend::TestBackend};
        let mut app = create_empty_app();
        app.config.focus_mode = true;
        app.set_status("GNU Terry Pratchett");

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| crate::app::ui::draw(f, &mut app)).unwrap();

        let mut content = String::new();
        let buffer = terminal.backend().buffer();
        for y in 0..24u16 {
            for x in 0..80u16 {
                content.push_str(buffer[(x, y)].symbol());
            }
        }

        assert!(
            content.contains("GNU Terry Pratchett"),
            "Status message should appear even in focus mode"
        );
    }

    #[test]
    fn test_draw_no_formatting_page_numbers() {
        use ratatui::{Terminal, backend::TestBackend};
        let mut app = create_empty_app();
        app.config.no_formatting = true;

        app.lines = vec!["Action line".to_string()];
        app.types = vec![LineType::Action];
        app.update_layout();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| crate::app::ui::draw(f, &mut app)).unwrap();

        let buffer = terminal.backend().buffer();
        let mut found = false;
        for y in 0..23 {
            for x in 0..80 {
                let cell = &buffer[(x, y)];
                if cell.symbol() == "1" {
                    assert!(
                        !cell.modifier.contains(Modifier::BOLD),
                        "Page number should not be bold when no_formatting is true"
                    );
                    found = true;
                }
            }
        }
        assert!(found, "Page number not found");
    }

    #[test]
    fn test_draw_panel_style_resets_color() {
        use ratatui::{Terminal, backend::TestBackend};
        let mut app = create_empty_app();
        app.set_status("Test status");

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| crate::app::ui::draw(f, &mut app)).unwrap();

        let buffer = terminal.backend().buffer();
        let status_cell = &buffer[(0, 23)];
        assert_eq!(
            status_cell.fg,
            Color::LightBlue,
            "Panel should use mode background color for label"
        );
        assert!(status_cell.modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_ux_boundary_beginning_of_file() {
        let mut app = create_empty_app();
        app.lines = vec!["First".to_string()];
        app.cursor_y = 0;
        app.cursor_x = 0;

        app.move_up();
        app.move_left();
        app.move_word_left();
        app.backspace();

        assert_eq!(app.cursor_y, 0);
        assert_eq!(app.cursor_x, 0);
        assert_eq!(app.lines[0], "First");
    }

    #[test]
    fn test_ux_boundary_end_of_file() {
        let mut app = create_empty_app();
        app.lines = vec!["Last".to_string()];
        app.cursor_y = 0;
        app.cursor_x = 4;

        app.move_down();
        app.move_right();
        app.move_word_right();
        app.delete_forward();

        assert_eq!(app.cursor_y, 0);
        assert_eq!(app.cursor_x, 4);
        assert_eq!(app.lines[0], "Last");
    }

    #[test]
    fn test_ux_line_joining_backspace() {
        let mut app = create_empty_app();
        app.lines = vec!["Hello ".to_string(), "World".to_string()];
        app.cursor_y = 1;
        app.cursor_x = 0;

        app.backspace();

        assert_eq!(app.lines.len(), 1);
        assert_eq!(app.lines[0], "Hello World");
        assert_eq!(app.cursor_y, 0);
        assert_eq!(app.cursor_x, 6);
    }

    #[test]
    fn test_ux_line_joining_delete() {
        let mut app = create_empty_app();
        app.lines = vec!["Hello ".to_string(), "World".to_string()];
        app.cursor_y = 0;
        app.cursor_x = 6;

        app.delete_forward();

        assert_eq!(app.lines.len(), 1);
        assert_eq!(app.lines[0], "Hello World");
        assert_eq!(app.cursor_y, 0);
        assert_eq!(app.cursor_x, 6);
    }

    #[test]
    fn test_ux_line_splitting_enter() {
        let mut app = create_empty_app();
        app.lines = vec!["HelloWorld".to_string()];
        app.cursor_y = 0;
        app.cursor_x = 5;

        app.insert_newline(false);

        assert_eq!(app.lines.len(), 2);
        assert_eq!(app.lines[0], "Hello");
        assert_eq!(app.lines[1], "World");
        assert_eq!(app.cursor_y, 1);
        assert_eq!(app.cursor_x, 0);
    }

    #[test]
    fn test_ux_utf8_multibyte_safety() {
        let mut app = create_empty_app();

        app.lines = vec!["пути творчества".to_string()];
        app.cursor_y = 0;
        app.cursor_x = 15;

        app.delete_word_back();
        app.backspace();

        app.insert_char('н');
        app.insert_char(' ');
        app.insert_char('🦀');

        assert_eq!(app.lines[0], "путин 🦀");
        app.cursor_x = 7;

        app.backspace();
        assert_eq!(app.lines[0], "путин ", "backspace should delete emoji");
        assert_eq!(
            app.cursor_x, 6,
            "cursor should move back once after deleting emoji"
        );

        app.backspace();
        assert_eq!(
            app.lines[0], "путин",
            "backspace should delete trailing space"
        );
        assert_eq!(app.cursor_x, 5, "cursor should be at end of word");

        app.insert_char(' ');
        app.insert_char('х');
        app.insert_char('у');
        app.insert_char('й');
        app.insert_char('л');
        app.insert_char('о');
        assert_eq!(
            app.lines[0], "путин хуйло",
            "insert_char should append correctly"
        );
        assert_eq!(app.cursor_x, 11, "cursor should be at end after inserts");

        app.cursor_x = 0;
        for _ in 0..6 {
            app.delete_forward();
        }
        assert_eq!(
            app.lines[0], "хуйло",
            "delete_forward should remove first word char by char"
        );
        assert_eq!(app.cursor_x, 0, "cursor should stay at position 0");

        app.cursor_x = 5;
        app.backspace();
        app.backspace();
        assert_eq!(
            app.lines[0], "хуй",
            "delete_word_back should remove last two chars"
        );
        assert_eq!(app.cursor_x, 3, "cursor should be at end of remaining word");
    }

    #[test]
    fn test_ux_visual_up_down_inside_soft_wrapped_line() {
        let mut app = create_empty_app();
        let long_line = "A".repeat(100);
        app.lines = vec!["Short line".to_string(), long_line];
        app.types = vec![LineType::Action, LineType::Action];

        app.update_layout();

        app.cursor_y = 1;
        app.cursor_x = 80;
        app.target_visual_x = 20;

        app.move_up();

        assert_eq!(
            app.cursor_y, 1,
            "Cursor should stay on the same logical line"
        );
        assert_eq!(
            app.cursor_x, 20,
            "Cursor should move to the upper visual row of the soft-wrapped line"
        );

        app.move_down();
        assert_eq!(app.cursor_y, 1);
        assert_eq!(
            app.cursor_x, 80,
            "Cursor should return to the lower visual row"
        );
    }

    #[test]
    fn test_ux_smart_pairing_deletion() {
        let mut app = create_empty_app();
        app.lines = vec!["()".to_string()];
        app.cursor_y = 0;
        app.cursor_x = 1;

        app.backspace();
        assert_eq!(app.lines[0], "");
        assert_eq!(app.cursor_x, 0);
    }

    #[test]
    fn test_ux_undo_restores_cursor_position_perfectly() {
        let mut app = create_empty_app();
        app.lines = vec!["Some text".to_string()];
        app.cursor_y = 0;
        app.cursor_x = 5;

        app.save_state(true);

        app.insert_char('A');
        assert_eq!(app.cursor_x, 6);

        app.undo();

        assert_eq!(app.lines[0], "Some text");
        assert_eq!(app.cursor_x, 5);
    }

    #[test]
    fn test_ux_ghost_cursor_memory_target_x() {
        let mut app = create_empty_app();
        app.lines = vec!["a".repeat(20), "b".repeat(3), "c".repeat(20)];

        app.parse_document();

        app.cursor_y = 0;
        app.cursor_x = 15;
        app.update_layout();
        app.target_visual_x = app.current_visual_x();

        app.move_down();
        assert_eq!(app.cursor_y, 1);
        assert_eq!(app.cursor_x, 3);

        app.move_down();
        assert_eq!(app.cursor_y, 2);

        assert_eq!(
            app.cursor_x, 15,
            "Cursor forgot its target_visual_x memory!"
        );
    }

    #[test]
    fn test_ux_tab_state_machine_middle_of_line() {
        let mut app = create_empty_app();
        app.lines = vec!["Some text here".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_y = 0;
        app.cursor_x = 5;

        app.handle_tab();

        assert_eq!(app.lines[0], "@Some text here");
        assert_eq!(
            app.cursor_x, 6,
            "Cursor must shift right when a sigil is prepended!"
        );
    }

    #[test]
    fn test_draw_force_ascii_and_no_color_strips_ui_elements() {
        use ratatui::{Terminal, backend::TestBackend};

        let mut app = create_empty_app();
        app.config.force_ascii = true;
        app.config.no_color = true;

        app.lines = vec!["===".to_string(), "INT. TEST SCENE".to_string()];
        app.types = vec![LineType::PageBreak, LineType::SceneHeading];

        app.cursor_y = 1;

        app.update_layout();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| crate::app::ui::draw(f, &mut app)).unwrap();

        let mut content = String::new();
        let buffer = terminal.backend().buffer();
        for y in 0..24u16 {
            for x in 0..80u16 {
                content.push_str(buffer[(x, y)].symbol());
            }
            content.push('\n');
        }

        assert!(
            content.contains("------------------------------------------------------------"),
            "Page break should use ASCII '-' instead of Unicode '─'"
        );
        assert!(
            !content.contains("────────────────────────────────────────────────────────────"),
            "Page break should NOT contain Unicode '─' in force_ascii mode"
        );

        assert!(
            content.contains("INT. TEST SCENE"),
            "Standard text should be rendered"
        );
    }

    #[test]
    fn test_search_regex_basic_and_highlight_flag() {
        let mut app = create_empty_app();
        app.lines = vec!["Hello world".to_string(), "Line two".to_string()];
        app.search_query = "world".to_string();
        app.cursor_y = 0;
        app.cursor_x = 0;

        app.execute_search();

        assert_eq!(app.cursor_y, 0);
        assert_eq!(app.cursor_x, 6);
        assert!(
            app.show_search_highlight,
            "Highlight flag should be active after finding"
        );
    }

    #[test]
    fn test_search_regex_wrap_around() {
        let mut app = create_empty_app();
        app.lines = vec!["First target".to_string(), "Second line".to_string()];
        app.search_query = "target".to_string();
        app.cursor_y = 1;
        app.cursor_x = 0;

        app.execute_search();

        assert_eq!(app.cursor_y, 0, "Should wrap around to line 0");
        assert_eq!(app.cursor_x, 6, "Index of 't' in 'target'");
        assert_eq!(
            app.status_msg.as_deref(),
            Some("Search Wrapped"),
            "Should display wrapped status message"
        );
    }

    #[test]
    fn test_search_regex_utf8_multibyte_safety() {
        let mut app = create_empty_app();

        app.lines = vec!["путин 🦀 краб".to_string()];
        app.search_query = "краб".to_string();
        app.cursor_y = 0;
        app.cursor_x = 0;

        app.execute_search();

        assert_eq!(
            app.cursor_x, 8,
            "Search must correctly convert byte offsets to char offsets"
        );
    }

    #[test]
    fn test_search_highlight_cleared_on_escape() {
        use crossterm::event::{
            Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
        };

        let mut app = create_empty_app();
        app.lines = vec!["Target word".to_string()];
        app.search_query = "word".to_string();
        app.execute_search();

        assert!(app.show_search_highlight);

        let esc_event = Event::Key(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        });

        let mut update_x = false;
        let mut text_ch = false;
        let mut cur_moved = false;

        let _ = app
            .handle_event(esc_event, &mut update_x, &mut text_ch, &mut cur_moved)
            .unwrap();

        assert!(
            !app.show_search_highlight,
            "Highlight flag should be reset when pressing Escape"
        );
        assert!(
            text_ch,
            "Text changed flag should trigger redraw to clear highlights"
        );
    }

    #[test]
    fn test_move_page_down_and_up() {
        let mut app = create_empty_app();
        app.lines = (0..50).map(|i| format!("Line {}", i)).collect();
        app.parse_document();
        app.update_layout();
        app.visible_height = 10;

        app.move_page_down();
        assert_eq!(app.cursor_y, 10);

        app.move_page_up();
        assert_eq!(app.cursor_y, 0);
    }

    #[test]
    fn test_report_cursor_position_empty() {
        let mut app = create_empty_app();
        app.report_cursor_position();

        assert_eq!(
            app.status_msg.as_deref(),
            Some("line 1/1 (100%), col 1/1 (100%), char 1/1 (100%)"),
            "Empty document should report 100% for all metrics"
        );
    }

    #[test]
    fn test_report_cursor_position_basic_math() {
        let mut app = create_empty_app();
        app.lines = vec!["Hello".to_string()];
        app.types = vec![LineType::Action];
        app.update_layout();

        app.cursor_y = 0;
        app.cursor_x = 2;

        app.report_cursor_position();

        assert_eq!(
            app.status_msg.as_deref(),
            Some("line 1/1 (100%), col 3/6 (50%), char 3/6 (50%)")
        );
    }

    #[test]
    fn test_report_cursor_position_soft_wrap() {
        let mut app = create_empty_app();
        let long_line = "A".repeat(100);
        app.lines = vec![long_line];
        app.types = vec![LineType::Action];
        app.update_layout();

        app.cursor_y = 0;
        app.cursor_x = 70;

        app.report_cursor_position();

        assert_eq!(
            app.status_msg.as_deref(),
            Some("line 1/1 (100%), col 71/101 (70%), char 71/101 (70%)"),
            "Soft-wrapped lines count as one logical line"
        );
    }

    #[test]
    fn test_report_cursor_position_multi_line() {
        let mut app = create_empty_app();
        app.lines = vec!["One".to_string(), "Two".to_string(), "Three".to_string()];
        app.types = vec![LineType::Action, LineType::Action, LineType::Action];
        app.update_layout();

        app.cursor_y = 1;
        app.cursor_x = 1;

        app.report_cursor_position();

        assert_eq!(
            app.status_msg.as_deref(),
            Some("line 2/3 (66%), col 2/4 (50%), char 6/14 (42%)")
        );
    }

    #[test]
    fn test_report_cursor_position_utf8_multibyte() {
        let mut app = create_empty_app();

        app.lines = vec!["Дратути 👋".to_string()];
        app.types = vec![LineType::Action];
        app.update_layout();

        app.cursor_y = 0;
        app.cursor_x = 8;

        app.report_cursor_position();

        assert_eq!(
            app.status_msg.as_deref(),
            Some("line 1/1 (100%), col 9/10 (90%), char 9/10 (90%)"),
            "Cursor metrics should count UTF-8 chars, not raw bytes"
        );
    }

    #[test]
    fn test_report_cursor_position_end_of_file() {
        let mut app = create_empty_app();
        app.lines = vec!["123".to_string(), "45".to_string()];
        app.types = vec![LineType::Action, LineType::Action];
        app.update_layout();

        app.cursor_y = 1;
        app.cursor_x = 2;

        app.report_cursor_position();

        assert_eq!(
            app.status_msg.as_deref(),
            Some("line 2/2 (100%), col 3/3 (100%), char 7/7 (100%)"),
            "Should safely handle cursor being positioned at the absolute end of the line"
        );
    }

    #[test]
    fn test_draw_typewriter_mode_normal() {
        use ratatui::{
            Terminal,
            backend::{Backend, TestBackend},
        };
        let mut app = create_empty_app();
        app.config.typewriter_mode = true;
        app.config.strict_typewriter_mode = false;
        app.lines = vec!["Line 1".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_y = 0;
        app.update_layout();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| crate::app::ui::draw(f, &mut app)).unwrap();

        assert_eq!(app.scroll, 0);
        assert_eq!(terminal.backend_mut().get_cursor_position().unwrap().y, 0);
    }

    #[test]
    fn test_draw_typewriter_mode_strict() {
        use ratatui::{
            Terminal,
            backend::{Backend, TestBackend},
        };
        let mut app = create_empty_app();
        app.config.strict_typewriter_mode = true;
        app.lines = vec!["Line 1".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_y = 0;
        app.update_layout();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| crate::app::ui::draw(f, &mut app)).unwrap();

        assert_eq!(terminal.backend_mut().get_cursor_position().unwrap().y, 12);
    }

    fn send_key_press(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
        use crossterm::event::{Event, KeyEvent, KeyEventKind, KeyEventState};
        let mut update_target_x = false;
        let mut text_changed = false;
        let mut cursor_moved = false;

        let ev = Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        });

        let _ = app.handle_event(
            ev,
            &mut update_target_x,
            &mut text_changed,
            &mut cursor_moved,
        );
    }

    #[test]
    fn test_nano_multibuffer_indicator_persistence() {
        let mut app = create_empty_app();
        app.buffers = vec![BufferState::default(), BufferState::default()];
        app.has_multiple_buffers = true;
        app.current_buf_idx = 0;

        app.switch_next_buffer();
        assert_eq!(app.current_buf_idx, 1, "Failed to switch buffer");

        let _ = app.current_buf_idx; // check switch

        app.close_current_buffer();

        assert_eq!(app.buffers.len(), 1, "Buffer should be closed");
        assert!(
            app.has_multiple_buffers,
            "Multiple buffers flag must not be reset to false"
        );
    }

    #[test]
    fn test_buffer_state_isolation_on_switch() {
        let mut app = create_empty_app();

        app.buffers = vec![
            BufferState {
                lines: vec!["".to_string()],
                ..Default::default()
            },
            BufferState {
                lines: vec!["".to_string()],
                ..Default::default()
            },
        ];
        app.has_multiple_buffers = true;

        app.insert_char('A');
        assert_eq!(app.lines[0], "A");
        assert!(app.dirty);

        app.switch_next_buffer();
        assert_eq!(app.current_buf_idx, 1);
        assert_eq!(app.lines[0], "");
        assert!(!app.dirty);

        app.insert_char('B');
        app.insert_char('C');
        assert_eq!(app.cursor_x, 2);

        app.switch_next_buffer();
        assert_eq!(app.current_buf_idx, 0);

        assert_eq!(app.lines[0], "A");
        assert_eq!(app.cursor_x, 1);
        assert!(app.dirty);
    }

    #[test]
    fn test_nano_navigation_and_deletion_shortcuts() {
        let mut app = create_empty_app();
        app.buffers = vec![
            BufferState {
                lines: vec!["".to_string()],
                ..Default::default()
            },
            BufferState {
                lines: vec!["".to_string()],
                ..Default::default()
            },
        ];
        app.has_multiple_buffers = true;

        app.lines = vec!["one two three".to_string()];
        app.cursor_x = 4;

        send_key_press(&mut app, KeyCode::Right, KeyModifiers::CONTROL);
        assert_eq!(app.cursor_x, 7, "Ctrl+Right should trigger move_word_right");

        send_key_press(&mut app, KeyCode::Backspace, KeyModifiers::CONTROL);
        assert_eq!(
            app.cursor_x, 4,
            "Ctrl+Backspace should delete word backwards"
        );

        app.switch_next_buffer();
        assert_eq!(
            app.current_buf_idx, 1,
            "switch_next_buffer (bn) failed"
        );

        app.switch_prev_buffer();
        assert_eq!(
            app.current_buf_idx, 0,
            "switch_prev_buffer (bp) failed"
        );
    }

    #[test]
    fn test_app_vertical_movement_cursor_clamp() {
        let mut app = create_empty_app();
        app.lines = vec![
            "Short".to_string(),
            "A very long line indeed".to_string(),
            "Tiny".to_string(),
        ];
        app.types = vec![LineType::Action, LineType::Action, LineType::Action];
        app.update_layout();

        app.cursor_y = 1;
        app.cursor_x = 20;
        app.target_visual_x = 20;

        app.move_up();
        assert_eq!(app.cursor_y, 0);
        assert_eq!(
            app.cursor_x, 5,
            "Cursor should be clamped to the length of 'Short'"
        );

        app.cursor_y = 1;
        app.cursor_x = 20;

        app.move_down();
        assert_eq!(app.cursor_y, 2);
        assert_eq!(
            app.cursor_x, 4,
            "Cursor should be clamped to the length of 'Tiny'"
        );
    }

    #[test]
    fn test_app_deletion_out_of_bounds_cursor_clamp() {
        let mut app = create_empty_app();
        app.lines = vec!["Word".to_string()];
        app.cursor_y = 0;
        app.cursor_x = 100;

        app.backspace();
        assert_eq!(
            app.cursor_x, 3,
            "Cursor should jump to line end and delete last char"
        );
        assert_eq!(app.lines[0], "Wor");
    }

    #[test]
    fn test_app_delete_forward_out_of_bounds_cursor_clamp() {
        let mut app = create_empty_app();
        app.lines = vec!["Word".to_string(), "Next".to_string()];
        app.cursor_y = 0;
        app.cursor_x = 100;

        app.delete_forward();
        assert_eq!(app.cursor_x, 4);
        assert_eq!(app.lines[0], "WordNext");
        assert_eq!(app.lines.len(), 1);
    }

    #[test]
    fn test_app_autocomplete_forced_scene_heading() {
        let mut app = create_empty_app();
        app.lines = vec![
            ".KITCHEN - DAY".to_string(),
            "".to_string(),
            ".KIT".to_string(),
        ];
        app.cursor_y = 2;
        app.cursor_x = 4;
        app.parse_document();
        app.update_autocomplete();
        assert_eq!(app.suggestion, Some("CHEN - DAY".to_string()));
    }

    #[test]
    fn test_app_autocomplete_scene_heading_without_dot() {
        let mut app = create_empty_app();
        app.lines = vec![
            "INT BIG ROOM - DAY".to_string(),
            "".to_string(),
            "INT BI".to_string(),
        ];
        app.cursor_y = 2;
        app.cursor_x = 6;
        app.parse_document();
        app.update_autocomplete();
        assert_eq!(app.suggestion, Some("G ROOM - DAY".to_string()));
    }

    #[test]
    fn test_app_tab_autocomplete_character_without_at_symbol() {
        let mut app = create_empty_app();
        app.characters.insert("CHARLOTTE".to_string());
        app.characters.insert("RENÉ".to_string());

        app.lines = vec!["C".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_y = 0;
        app.cursor_x = 1;

        app.handle_tab();

        assert_eq!(
            app.lines[0], "C",
            "Should NOT prepend '@' when a valid character match exists"
        );
        assert_eq!(app.suggestion.as_deref(), Some("HARLOTTE"));

        app.update_autocomplete();

        assert_eq!(
            app.types[0],
            LineType::Character,
            "LineType must temporarily change to Character to center the text"
        );
        assert_eq!(
            app.suggestion.as_deref(),
            Some("HARLOTTE"),
            "Suggestion must survive the update_autocomplete cycle"
        );

        app.handle_tab();

        assert_eq!(app.lines[0], "CHARLOTTE");
        assert_eq!(app.suggestion, None);
        assert_eq!(app.cursor_x, 9);
    }

    #[test]
    fn test_app_tab_autocomplete_fallback_to_at_symbol_for_unknown() {
        let mut app = create_empty_app();
        app.characters.insert("CHARLOTTE".to_string());
        app.characters.insert("RENÉ".to_string());

        app.lines = vec!["X".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_y = 0;
        app.cursor_x = 1;

        app.handle_tab();

        assert_eq!(
            app.lines[0], "@X",
            "Must fallback to prepending '@' because 'X' matches no characters"
        );
        assert_eq!(app.suggestion, None);
        assert_eq!(app.cursor_x, 2);
    }

    #[test]
    fn test_app_no_ghost_text_while_typing_action_line() {
        let mut app = create_empty_app();
        app.characters.insert("CHARLOTTE".to_string());
        app.characters.insert("RENÉ".to_string());

        app.lines = vec!["C".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_y = 0;
        app.cursor_x = 1;

        app.update_autocomplete();

        assert_eq!(
            app.suggestion, None,
            "Typing on an Action line should NOT show ghost text unless Tab is pressed"
        );
        assert_eq!(
            app.types[0],
            LineType::Action,
            "LineType must remain Action during normal typing"
        );
    }

    #[test]
    fn test_app_tab_autocomplete_fixes_case_on_accept() {
        let mut app = create_empty_app();
        app.characters.insert("RENÉ".to_string());

        app.lines = vec!["re".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_y = 0;
        app.cursor_x = 2;

        app.handle_tab();
        app.update_autocomplete();
        app.handle_tab();

        assert_eq!(
            app.lines[0], "RENÉ",
            "The existing lowercase prefix must be uppercased upon accepting the suggestion"
        );
    }

    #[test]
    fn test_app_tab_fallback_strip_sigils_restored() {
        let mut app = create_empty_app();
        app.lines = vec!["~I get a strange magic".to_string()];
        app.types = vec![LineType::Empty];
        app.cursor_y = 0;
        app.cursor_x = 12;

        app.handle_tab();

        assert_eq!(
            app.lines[0], "I get a strange magic",
            "The fallback block at the end of handle_tab must strip the '~' sigil"
        );
        assert_eq!(
            app.cursor_x, 11,
            "Cursor should shift left by 1 after stripping the sigil"
        );
    }

    #[test]
    fn test_app_tab_autocomplete_cancellation_reverts_magic() {
        let mut app = create_empty_app();
        app.characters.insert("CHARLOTTE".to_string());

        app.lines = vec!["C".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_y = 0;
        app.cursor_x = 1;

        app.handle_tab();
        app.update_autocomplete();

        assert_eq!(
            app.types[0],
            LineType::Character,
            "Sanity check: magic applied"
        );
        assert!(app.suggestion.is_some(), "Sanity check: suggestion exists");

        app.insert_char('a');

        app.parse_document();
        app.update_autocomplete();

        assert_eq!(
            app.types[0],
            LineType::Action,
            "LineType must revert to Action after the user types a new lowercase character"
        );
        assert_eq!(
            app.suggestion, None,
            "Suggestion must be cleared when the user interrupts the autocomplete flow"
        );
    }

    #[test]
    fn test_app_tab_autocomplete_exact_match_prepends_at() {
        let mut app = create_empty_app();
        app.characters.insert("RENÉ".to_string());

        app.lines = vec!["RENÉ".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_y = 0;
        app.cursor_x = 4;

        app.handle_tab();

        assert_eq!(
            app.lines[0], "@RENÉ",
            "If the typed word exactly matches a character, Tab should force a character cue by prepending '@'"
        );
        assert_eq!(app.suggestion, None);
        assert_eq!(app.cursor_x, 5);
    }

    #[test]
    fn test_app_tab_autocomplete_interrupted_by_enter() {
        let mut app = create_empty_app();
        app.characters.insert("CHARLOTTE".to_string());

        app.lines = vec!["C".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_y = 0;
        app.cursor_x = 1;

        app.handle_tab();
        app.update_autocomplete();
        assert_eq!(app.types[0], LineType::Character, "Magic is active");

        app.suggestion = None;
        app.insert_newline(false);
        app.parse_document();
        app.update_autocomplete();

        assert_eq!(app.lines.len(), 2, "Newline should be inserted");
        assert_eq!(
            app.lines[0], "C",
            "Original line must remain unchanged (no ghost text applied)"
        );
        assert_eq!(app.lines[1], "", "New line should be empty");
        assert_eq!(
            app.types[0],
            LineType::Action,
            "The magic LineType::Character MUST revert to Action because 'C' is not a valid cue"
        );
        assert_eq!(app.suggestion, None, "Suggestion must be destroyed");
    }

    #[test]
    fn test_app_tab_autocomplete_cursor_in_middle_of_word() {
        let mut app = create_empty_app();
        app.characters.insert("RENÉ".to_string());

        app.lines = vec!["Rblablabla".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_y = 0;
        app.cursor_x = 1;

        app.handle_tab();

        assert_eq!(
            app.lines[0], "@Rblablabla",
            "Should prepend '@' because the entire trimmed line ('Rblablabla') does not match 'RENÉ'"
        );
        assert_eq!(
            app.cursor_x, 2,
            "Cursor should shift right by 1 due to the prepended '@'"
        );
    }

    #[test]
    fn test_app_tab_autocomplete_trailing_space() {
        let mut app = create_empty_app();
        app.characters.insert("CHARLOTTE".to_string());

        app.lines = vec!["C ".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_y = 0;
        app.cursor_x = 2;

        app.handle_tab();

        assert_eq!(
            app.lines[0], "@C ",
            "Should safely fallback to prepending '@' when there is a trailing space"
        );
        assert_eq!(
            app.suggestion, None,
            "Suggestion must NOT be created for strings with trailing spaces"
        );
        assert_eq!(app.cursor_x, 3, "Cursor shifts by 1 because of '@'");
    }

    #[test]
    fn test_app_deduplicate_files() {
        let mut cli = Cli::default();
        cli.files = vec![
            std::path::PathBuf::from("test.fountain"),
            std::path::PathBuf::from("test.fountain"),
        ];
        let app = App::new(cli);
        assert_eq!(app.buffers.len(), 1, "Duplicate files should be removed");
    }

    #[test]
    fn test_app_emergency_save() {
        let mut app = create_empty_app();
        app.lines = vec!["Test recovery data".to_string()];
        app.dirty = true;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("fount_test_recovery.fountain");
        app.file = Some(file_path.clone());

        app.emergency_save();

        let save_path = temp_dir.join("fount_test_recovery.fountain.save");
        assert!(save_path.exists());

        let _ = std::fs::remove_file(save_path);
    }

    #[test]
    fn test_app_save_command() {
        let mut app = create_empty_app();
        app.lines = vec!["Test save".to_string()];
        app.dirty = true;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("fount_test_save.fountain");
        app.file = Some(file_path.clone());

        assert!(app.save().is_ok());
        assert!(!app.dirty);
        assert!(file_path.exists());

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_app_mouse_scrolling() {
        use crossterm::event::{Event, MouseEvent, MouseEventKind};
        let mut app = create_empty_app();
        app.lines = vec!["1".to_string(), "2".to_string()];
        app.update_layout();

        let mut t1 = false;
        let mut t2 = false;
        let mut t3 = false;

        let scroll_down = Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: crossterm::event::KeyModifiers::empty(),
        });
        let _ = app
            .handle_event(scroll_down, &mut t1, &mut t2, &mut t3)
            .unwrap();
        assert_eq!(app.cursor_y, 1);

        let scroll_up = Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 0,
            modifiers: crossterm::event::KeyModifiers::empty(),
        });
        let _ = app
            .handle_event(scroll_up, &mut t1, &mut t2, &mut t3)
            .unwrap();
        assert_eq!(app.cursor_y, 0);
    }

    #[test]
    fn test_app_prompt_save_logic() {
        let mut app = create_empty_app();
        app.mode = AppMode::PromptSave;

        let temp_dir = std::env::temp_dir();
        app.file = Some(temp_dir.join("dummy.fountain"));

        send_key_press(&mut app, KeyCode::Char('y'), KeyModifiers::empty());
        assert_eq!(app.mode, AppMode::Normal);

        app.mode = AppMode::PromptSave;
        app.exit_after_save = true;
        let mut t1 = false;
        let mut t2 = false;
        let mut t3 = false;
        use crossterm::event::{Event, KeyEvent, KeyEventKind, KeyEventState};
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        });
        let result = app.handle_event(ev, &mut t1, &mut t2, &mut t3).unwrap();
        assert!(
            result,
            "Should return true (exit) when 'n' pressed and exit_after_save is true"
        );
    }

    #[test]
    fn test_app_prompt_filename_logic() {
        let mut app = create_empty_app();
        app.mode = AppMode::PromptFilename;
        app.filename_input = "i like trains".to_string();

        send_key_press(&mut app, KeyCode::Char('!'), KeyModifiers::empty());
        assert_eq!(app.filename_input, "i like trains!");

        send_key_press(&mut app, KeyCode::Backspace, KeyModifiers::empty());
        assert_eq!(app.filename_input, "i like trains");

        send_key_press(&mut app, KeyCode::Esc, KeyModifiers::empty());
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn test_app_close_last_buffer_returns_home() {
        let mut app = create_empty_app();
        assert_eq!(app.buffers.len(), 1);

        let should_exit = app.close_current_buffer();
        assert!(
            !should_exit,
            "Closing the last buffer should NOT exit the app"
        );
        assert_eq!(app.mode, AppMode::Home, "Should return to Home mode");
        assert!(app.buffers.is_empty(), "Buffers should be empty");
    }

    #[test]
    fn test_app_close_middle_buffer() {
        let mut app = create_empty_app();
        app.buffers = vec![
            BufferState {
                lines: vec!["Buf 0".to_string()],
                ..Default::default()
            },
            BufferState {
                lines: vec!["Buf 1".to_string()],
                ..Default::default()
            },
            BufferState {
                lines: vec!["Buf 2".to_string()],
                ..Default::default()
            },
        ];
        app.current_buf_idx = 1;
        app.has_multiple_buffers = true;

        let should_exit = app.close_current_buffer();

        assert!(!should_exit);
        assert_eq!(app.buffers.len(), 2);
        assert_eq!(app.current_buf_idx, 1);
        assert_eq!(app.lines[0], "Buf 2");
    }

    #[test]
    fn test_app_prompt_save_cancel_via_esc_and_ctrl_c() {
        let mut app = create_empty_app();

        app.mode = AppMode::PromptSave;
        send_key_press(&mut app, KeyCode::Esc, KeyModifiers::empty());
        assert_eq!(app.mode, AppMode::Normal);
        assert_eq!(app.status_msg.as_deref(), Some("Cancelled"));

        app.mode = AppMode::PromptSave;
        send_key_press(&mut app, KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(app.mode, AppMode::Normal);
        assert_eq!(app.status_msg.as_deref(), Some("Cancelled"));
    }

    #[test]
    fn test_app_prompt_filename_empty_input_cancels() {
        let mut app = create_empty_app();
        app.mode = AppMode::PromptFilename;
        app.filename_input = "   ".to_string();

        send_key_press(&mut app, KeyCode::Enter, KeyModifiers::empty());

        assert_eq!(app.mode, AppMode::Normal);
        assert_eq!(app.status_msg.as_deref(), Some("Cancelled"));
    }

    #[test]
    fn test_app_prompt_filename_save_error() {
        let mut app = create_empty_app();
        app.mode = AppMode::PromptFilename;
        app.filename_input =
            "/this/path/doesnt/exist/neither/does/the/meaning/of/life.fountain".to_string();

        send_key_press(&mut app, KeyCode::Enter, KeyModifiers::empty());

        assert_eq!(app.mode, AppMode::Normal);
        assert!(
            app.status_msg
                .as_deref()
                .unwrap_or("")
                .starts_with("Error saving:"),
            "An error saving message should appear"
        );
    }

    #[test]
    fn test_app_search_cancel_via_esc_and_ctrl_c() {
        let mut app = create_empty_app();

        app.mode = AppMode::Search;
        app.search_query = "something".to_string();
        app.show_search_highlight = true;

        send_key_press(&mut app, KeyCode::Esc, KeyModifiers::empty());

        assert_eq!(app.mode, AppMode::Normal);
        assert!(!app.show_search_highlight);
        assert!(app.search_query.is_empty(), "Query should be cleared");
        assert_eq!(app.status_msg.as_deref(), Some("Cancelled"));
    }

    #[test]
    fn test_app_search_backspace_to_empty_and_enter() {
        let mut app = create_empty_app();
        app.mode = AppMode::Search;
        app.search_query = "a".to_string();

        send_key_press(&mut app, KeyCode::Backspace, KeyModifiers::empty());
        assert!(app.search_query.is_empty());

        send_key_press(&mut app, KeyCode::Enter, KeyModifiers::empty());

        assert_eq!(app.mode, AppMode::Normal);
        assert_eq!(app.status_msg.as_deref(), Some("Cancelled"));
    }

    #[test]
    fn test_app_shift_enter_literal_newline() {
        let mut app = create_empty_app();
        app.lines = vec!["Action line.".to_string()];
        app.types = vec![LineType::Action];
        app.cursor_y = 0;
        app.cursor_x = 6;
        app.config.auto_paragraph_breaks = true;

        app.insert_newline(true);

        assert_eq!(
            app.lines.len(),
            2,
            "Should be exactly 2 lines; auto-paragraphs are ignored with Shift"
        );
        assert_eq!(app.lines[0], "Action");
        assert_eq!(app.lines[1], " line.");
    }

    #[test]
    fn test_app_undo_stack_limit_truncation() {
        let mut app = create_empty_app();
        app.lines = vec!["".to_string()];

        for _i in 0..650 {
            app.insert_char('a');
            app.save_state(true);
        }

        assert!(
            app.undo_stack.len() <= 640,
            "Undo stack should be truncated at 640 (...ought to be enough for anybody)"
        );
    }

    #[test]
    fn test_draw_metadata_key_dimming() {
        use ratatui::{Terminal, backend::TestBackend};
        let mut app = create_empty_app();
        app.config.no_color = false;
        app.config.strict_typewriter_mode = false;
        app.lines = vec!["Author: René".to_string()];
        app.types = vec![LineType::MetadataKey];
        app.update_layout();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| crate::app::ui::draw(f, &mut app)).unwrap();

        let buffer = terminal.backend().buffer();
        let mut found_gray_colon = false;

        for y in 0..5 {
            for x in 0..80 {
                let cell = &buffer[(x, y)];
                if cell.symbol() == ":" {
                    assert_eq!(
                        cell.fg,
                        Color::DarkGray,
                        "Metadata key should be rendered in gray"
                    );
                    found_gray_colon = true;
                }
            }
        }
        assert!(found_gray_colon, "Metadata colon not found on screen");
    }

    #[test]
    fn test_handle_event_ex_command_closes_app() {
        let mut app = create_empty_app();
        app.dirty = false;

        let (mut changed, mut moved, mut update) = (false, false, false);
        app.command_input = "ex".to_string();
        let result = app
            .execute_command(&mut changed, &mut moved, &mut update)
            .unwrap();

        assert!(result, "/ex command should return true to exit the application");
    }

    #[test]
    fn test_app_tab_autocomplete_character_edge_case_dots() {
        let mut app = create_empty_app();
        app.lines = vec![
            "@R.C.".to_string(),
            "Text".to_string(),
            "".to_string(),
            "R".to_string(),
        ];
        app.parse_document();
        app.cursor_y = 3;
        app.cursor_x = 1;

        app.handle_tab();
        assert_eq!(
            app.suggestion.as_deref(),
            Some(".C."),
            "First Tab should offer autocomplete suggestion"
        );
        assert_eq!(app.lines[3], "R", "Line content should not change yet");

        app.parse_document();
        app.update_autocomplete();

        app.handle_tab();

        assert_eq!(
            app.lines[3], "@R.C.",
            "Should force a character cue with '@' because R.C. ends with a dot"
        );
        assert_eq!(app.cursor_x, 5, "Cursor should be at end of line");
    }

    #[test]
    fn test_app_tab_autocomplete_normal_character_regression() {
        let mut app = create_empty_app();
        app.lines = vec![
            "RENÉ".to_string(),
            "Text".to_string(),
            "".to_string(),
            "RE".to_string(),
        ];
        app.parse_document();
        app.cursor_y = 3;
        app.cursor_x = 2;

        app.update_autocomplete();
        assert_eq!(app.suggestion.as_deref(), Some("NÉ"));

        app.handle_tab();

        assert_eq!(
            app.lines[3], "RENÉ",
            "Should NOT prepend '@' for regular character names"
        );
        assert_eq!(app.cursor_x, 4);
    }

    #[test]
    fn test_app_tab_autocomplete_location_normal() {
        let mut app = create_empty_app();
        app.lines = vec![
            "INT. STADTWERKE BITTERFELD-WOLFEN - DAY".to_string(),
            "Action".to_string(),
            "".to_string(),
            ".STADT".to_string(),
        ];
        app.parse_document();
        app.cursor_y = 3;
        app.cursor_x = 6;

        app.update_autocomplete();
        assert_eq!(
            app.suggestion.as_deref(),
            Some("WERKE BITTERFELD-WOLFEN - DAY")
        );

        app.handle_tab();

        assert_eq!(
            app.lines[3], ".STADTWERKE BITTERFELD-WOLFEN - DAY",
            "Location entered with a dot prefix should not duplicate the dot"
        );
        assert_eq!(
            app.cursor_x, 35,
            "Cursor should account for the leading dot"
        );
    }

    #[test]
    fn test_ux_smart_pairing_basic_triggers() {
        let mut app = create_empty_app();

        app.insert_char('(');
        assert_eq!(app.lines[0], "()", "Failed to auto-pair parentheses");
        assert_eq!(
            app.cursor_x, 1,
            "Cursor should be placed inside the parentheses"
        );
        assert!(app.dirty, "Document should be marked dirty after insertion");

        app.lines = vec!["".to_string()];
        app.cursor_x = 0;
        app.insert_char('"');
        assert_eq!(app.lines[0], "\"\"", "Failed to auto-pair double quotes");
        assert_eq!(
            app.cursor_x, 1,
            "Cursor should be placed inside the double quotes"
        );

        app.lines = vec!["".to_string()];
        app.cursor_x = 0;
        app.insert_char('\'');
        assert_eq!(app.lines[0], "''", "Failed to auto-pair single quotes");
        assert_eq!(
            app.cursor_x, 1,
            "Cursor should be placed inside the single quotes"
        );
    }

    #[test]
    fn test_ux_smart_pairing_step_over_existing_closing_chars() {
        let mut app = create_empty_app();

        app.lines = vec!["()".to_string()];
        app.cursor_x = 1;
        app.insert_char(')');
        assert_eq!(
            app.lines[0], "()",
            "Should step over existing closing parenthesis"
        );
        assert_eq!(
            app.cursor_x, 2,
            "Cursor should advance past the closing parenthesis"
        );

        app.lines = vec!["\"\"".to_string()];
        app.cursor_x = 1;
        app.insert_char('"');
        assert_eq!(
            app.lines[0], "\"\"",
            "Should step over existing closing double quote"
        );
        assert_eq!(
            app.cursor_x, 2,
            "Cursor should advance past the closing double quote"
        );

        app.lines = vec!["''".to_string()];
        app.cursor_x = 1;
        app.insert_char('\'');
        assert_eq!(
            app.lines[0], "''",
            "Should step over existing closing single quote"
        );
        assert_eq!(
            app.cursor_x, 2,
            "Cursor should advance past the closing single quote"
        );

        app.lines = vec!["[[]]".to_string()];
        app.cursor_x = 2;
        app.insert_char(']');
        assert_eq!(
            app.lines[0], "[[]]",
            "Should step over existing closing bracket in Fountain notes"
        );
        assert_eq!(
            app.cursor_x, 3,
            "Cursor should advance past the first closing bracket"
        );
    }

    #[test]
    fn test_ux_smart_pairing_alphanumeric_boundary_rules() {
        let mut app = create_empty_app();

        app.lines = vec!["word".to_string()];
        app.cursor_x = 0;
        app.insert_char('(');
        assert_eq!(
            app.lines[0], "(word",
            "Should not auto-pair when directly preceding an alphanumeric character"
        );
        assert_eq!(app.cursor_x, 1, "Cursor should advance normally");

        app.lines = vec!["word".to_string()];
        app.cursor_x = 0;
        app.insert_char('"');
        assert_eq!(
            app.lines[0], "\"word",
            "Should not auto-pair double quotes when directly preceding a word"
        );

        app.lines = vec![" word".to_string()];
        app.cursor_x = 0;
        app.insert_char('(');
        assert_eq!(
            app.lines[0], "() word",
            "Should auto-pair when preceding whitespace"
        );

        app.lines = vec!["don".to_string()];
        app.cursor_x = 3;
        app.insert_char('\'');
        assert_eq!(
            app.lines[0], "don'",
            "Single quote immediately following an alphanumeric character must be treated as an apostrophe, not a pair"
        );

        app.lines = vec!["don ".to_string()];
        app.cursor_x = 4;
        app.insert_char('\'');
        assert_eq!(
            app.lines[0], "don ''",
            "Single quote following a space must be treated as a pairable quote"
        );
    }

    #[test]
    fn test_ux_smart_pairing_quote_parity_and_apostrophe_logic() {
        let mut app = create_empty_app();

        app.lines = vec!["\"hello\"".to_string()];
        app.cursor_x = 6;
        app.insert_char('"');
        assert_eq!(
            app.lines[0], "\"hello\"",
            "Should recognize odd parity inside a string and step over the closing quote"
        );
        assert_eq!(app.cursor_x, 7, "Cursor should advance past the quote");

        app.lines = vec!["\"hello\" ".to_string()];
        app.cursor_x = 8;
        app.insert_char('"');
        assert_eq!(
            app.lines[0], "\"hello\" \"\"",
            "Should recognize even parity outside strings and create a new pair"
        );
        assert_eq!(
            app.cursor_x, 9,
            "Cursor should be placed inside the new pair"
        );

        app.lines = vec!["don't say ".to_string()];
        app.cursor_x = 10;
        app.insert_char('"');
        assert_eq!(
            app.lines[0], "don't say \"\"",
            "Apostrophes must be strictly excluded from string literal parity counts"
        );
        assert_eq!(
            app.cursor_x, 11,
            "Cursor should be placed inside the double quotes"
        );

        app.lines = vec!["don't say ".to_string()];
        app.cursor_x = 10;
        app.insert_char('\'');
        assert_eq!(
            app.lines[0], "don't say ''",
            "Apostrophes must not prevent single quotes from pairing properly in valid contexts"
        );
    }

    #[test]
    fn test_ux_smart_pairing_fountain_multichar_elements() {
        let mut app = create_empty_app();

        app.lines = vec!["[".to_string()];
        app.cursor_x = 1;
        app.insert_char('[');
        assert_eq!(
            app.lines[0], "[[]]",
            "Consecutive open brackets must trigger Fountain note auto-completion"
        );
        assert_eq!(
            app.cursor_x, 2,
            "Cursor must be placed inside the Fountain note"
        );

        app.lines = vec!["/".to_string()];
        app.cursor_x = 1;
        app.insert_char('*');
        assert_eq!(
            app.lines[0], "/**/",
            "Slash followed by asterisk must trigger boneyard auto-completion"
        );
        assert_eq!(
            app.cursor_x, 2,
            "Cursor must be placed inside the boneyard markers"
        );

        app.lines = vec!["*".to_string()];
        app.cursor_x = 1;
        app.insert_char('*');
        assert_eq!(
            app.lines[0], "****",
            "Consecutive asterisks must trigger bold markdown auto-completion"
        );
        assert_eq!(
            app.cursor_x, 2,
            "Cursor must be placed inside the bold markers"
        );
    }

    #[test]
    fn test_ux_smart_pairing_backspace_removes_both_pairs_safely() {
        let mut app = create_empty_app();

        let pairs_to_test = vec![
            ("()", 1),
            ("\"\"", 1),
            ("''", 1),
            ("[[]]", 2),
            ("/**/", 2),
            ("****", 2),
        ];

        for (text, cursor_pos) in pairs_to_test {
            app.lines = vec![text.to_string()];
            app.cursor_x = cursor_pos;
            app.backspace();
            assert_eq!(
                app.lines[0], "",
                "Backspace failed to cleanly remove the empty pair sequence: {}",
                text
            );
            assert_eq!(
                app.cursor_x, 0,
                "Cursor should return to position 0 after pair deletion"
            );
        }
    }

    #[test]
    fn test_ux_smart_pairing_delete_forward_removes_both_pairs_safely() {
        let mut app = create_empty_app();

        let pairs_to_test = vec![
            ("()", 0),
            ("\"\"", 0),
            ("''", 0),
            ("[[]]", 0),
            ("/**/", 0),
            ("****", 0),
        ];

        for (text, cursor_pos) in pairs_to_test {
            app.lines = vec![text.to_string()];
            app.cursor_x = cursor_pos;
            app.delete_forward();
            assert_eq!(
                app.lines[0], "",
                "Forward delete failed to cleanly remove the empty pair sequence: {}",
                text
            );
            assert_eq!(
                app.cursor_x, 0,
                "Cursor should remain at position 0 after pair deletion"
            );
        }
    }

    #[test]
    fn test_ux_smart_pairing_unicode_and_emoji_boundaries() {
        let mut app = create_empty_app();

        app.lines = vec!["слово".to_string()];
        app.cursor_x = 0;
        app.insert_char('(');
        assert_eq!(
            app.lines[0], "(слово",
            "Cyrillic characters must be treated as alphanumeric, preventing auto-pairing"
        );

        app.lines = vec!["🦀".to_string()];
        app.cursor_x = 0;
        app.insert_char('(');
        assert_eq!(
            app.lines[0], "()🦀",
            "Emojis are not alphanumeric and must allow auto-pairing"
        );

        app.lines = vec!["Д".to_string()];
        app.cursor_x = 1;
        app.insert_char('\'');
        assert_eq!(
            app.lines[0], "Д'",
            "Apostrophe logic must correctly identify Cyrillic boundaries"
        );

        app.lines = vec!["Привет()Мир".to_string()];
        app.cursor_x = 7;
        app.backspace();
        assert_eq!(
            app.lines[0], "ПриветМир",
            "Pair deletion must respect multi-byte character boundaries during string mutation"
        );
        assert_eq!(
            app.cursor_x, 6,
            "Cursor must strictly track character indexing, not byte indexing, after pair deletion"
        );
    }

    #[test]
    fn test_app_inline_note_color_parsing_strictness() {
        let mut app = create_empty_app();

        app.lines = vec![
            "[[yellow text]]".to_string(),
            "[[this comment is yellow]]".to_string(),
            "[[marker]]".to_string(),
            "[[marker blue text]]".to_string(),
            "Action with [[green inline note]] inside.".to_string(),
            "Action with [[this is not green]] inside.".to_string(),
            "[[marker invalid color]]".to_string(),
        ];

        app.parse_document();
        app.update_layout();

        let note_yellow = &app.layout[0];
        assert_eq!(
            note_yellow.override_color,
            Some(ratatui::style::Color::Yellow),
            "Note starting with yellow must be yellow"
        );

        let note_none = &app.layout[1];
        assert_eq!(
            note_none.override_color, None,
            "Color word inside the text must be ignored"
        );

        let note_marker = &app.layout[2];
        assert_eq!(
            note_marker.override_color,
            Some(ratatui::style::Color::Rgb(255, 165, 0)),
            "Marker prefix without valid color must be orange"
        );

        let note_marker_blue = &app.layout[3];
        assert_eq!(
            note_marker_blue.override_color,
            Some(ratatui::style::Color::Blue),
            "Marker prefix with blue must be blue"
        );

        let action_green = &app.layout[4];
        let color_green = action_green.fmt.note_color.values().next().copied();
        assert_eq!(
            color_green,
            Some(ratatui::style::Color::Green),
            "Inline note starting with green must be green"
        );

        let action_none = &app.layout[5];
        assert!(
            action_none.fmt.note_color.is_empty(),
            "Inline note with color word inside text must not have a color override"
        );

        let note_marker_invalid = &app.layout[6];
        assert_eq!(
            note_marker_invalid.override_color,
            Some(ratatui::style::Color::Rgb(255, 165, 0)),
            "Marker prefix with invalid color must fallback to orange"
        );
    }

    #[test]
    fn test_app_tab_no_infinite_dots_after_non_empty_line() {
        let mut app = create_empty_app();

        app.lines = vec!["Шарлотта".to_string(), "Яблоко.".to_string()];
        app.parse_document();
        app.cursor_y = 1;
        app.cursor_x = 0;

        app.handle_tab();
        assert_eq!(app.lines[1], "@Яблоко.");
        app.parse_document();
        assert_eq!(app.lines[1], "@ЯБЛОКО.");
        assert_eq!(app.types[1], LineType::Character);

        app.handle_tab();
        assert_eq!(app.lines[1], ".ЯБЛОКО.");
        app.parse_document();
        assert_eq!(app.types[1], LineType::Action);

        app.handle_tab();
        assert_eq!(
            app.lines[1], ">ЯБЛОКО.",
            "Must turn into Transition (>), preventing infinite '@.' prepends"
        );
        app.parse_document();
        assert_eq!(app.types[1], LineType::Transition);

        app.handle_tab();
        assert_eq!(app.lines[1], "ЯБЛОКО.");
        app.parse_document();
        assert_eq!(app.types[1], LineType::Action);
    }

    #[test]
    fn test_integration() {
        let tutorial_text = r#"Title: Fount Tutorial
Credit: Written by
Author: René Coignard
Draft date: Version 0.2.17
Contact:
contact@renecoignard.com

INT. FLAT IN WOLFEN-NORD - DAY

RENÉ sits at his desk, typing.

RENÉ
(turning round)
Oh, hello there. It seems you've found my terminal Rust port of Beat. Sit back and I'll show you how everything works.

I sometimes write screenplays on my Gentoo laptop, and doing it in plain nano isn't terribly comfortable (I work entirely in the terminal there). So I decided to put this port of Beat together. I used Beat's source code as a reference when writing Fount, so things work more or less the same way.

As you may have already noticed, the navigation is rather reminiscent of nano, because I did look at its source code and took inspiration, for the sake of authenticity. I'm rather fond of it, and I hope you will be too. Not quite as nerdy as vim, but honestly, I'm an average nano enjoyer and I'm not ashamed of it.

Anyway, let's get into it.

EXT. NORDPARK - DAY

As I mentioned, things work much the same as in Beat. If you start a line with **int.** or **ext.**, Fount will automatically turn it into a scene heading. You can also use tab: on an empty line, it will first turn it into a character cue, then a scene heading, and then a transition. If you simply start typing IN CAPS ON AN EMPTY LINE, LIKE SO, the text will automatically become a character cue.

You can also use notes...

/* Two sailors are walking along the deck, when one turns to the other and says: */

SAILOR
I'm not a sailor, actually.

Fount automatically inserts two blank lines after certain elements, just as Beat does, though this can be adjusted in the configuration file. There's a sample config in the repository; do make use of it. Bonus: try enabling typewriter mode and see what happens.

To create a transition, simply write in capitals and end with a colon, like so...

CUT TO:

That alone is quite enough to write a proper screenplay. But there's more! For instance, we also have these...

/*

A multi-line comment.

For very, very, very long notes.

*/

[[Comments can look like this as well. They don't differ much from other comment types, but for compatibility with Beat, all the same comment types are supported.]]

# This is a new section

= And this is a synopsis.

INT. EDEKA - ABEND

Unlike Beat, there's no full render or PDF export here, but you can always save your screenplay and open it in Beat to do that. In Beat, synopses wouldn't appear in the rendered script, nor would comments. Which is why they share the same colour here, incidentally.

As you may have noticed, there's support for **bold text**, *italics*, and even _underlined text_. When your cursor isn't on a line containing these markers, they'll be hidden from view. Move onto the line, and you'll see all the asterisks and underscores that produce the formatting.

Centred text is supported as well, and works like this...

>Centred text<

You can also force transitions...

>AN ABRUPT TRANSITION TO THE NEXT SCENE:

EXT. WOLFEN(BITTERFELD) RAILWAY STATION - MORNING

Lyrics are supported too, using a tilde at the start of the line...

~Meine Damen, meine Herrn, danke
~Dass Sie mit uns reisen
~Zu abgefahrenen Preisen
~Auf abgefahrenen Gleisen
~Für Ihre Leidensfähigkeit, danken wir spontan
~Sänk ju for träweling wis Deutsche Bahn

That's Wise Guys. Onwards.

EXT. LEIPZIG HBF - MORNING

Well, do have a go on it, write something from scratch, or edit this screenplay. You might even turn up a bug or two; if so, please do let me know :-) Everything seemed to behave itself while I was putting this tutorial together, and I hope it all runs just as smoothly for you. I hope you enjoy working in Fount.

[[marker Speaking of which, I named the application after a certain Charlotte I once knew, who wrote quite wonderful screenplays.]]
[[marker blue The colour of these comment markers can be changed, as you can see.]]

You can find more information about the Fountain markup language at https://www.fountain.io/

And Beat itself, of course: https://www.beat-app.fi/

> FADE OUT"#;

        let mut app = App::new(crate::config::Cli::default());
        app.config.mirror_scene_numbers = crate::config::MirrorOption::Off;
        app.lines = tutorial_text.lines().map(|s| s.to_string()).collect();
        app.cursor_y = 0;
        app.cursor_x = 0;

        app.parse_document();
        app.update_layout();

        let get_exact_idx =
            |search_str: &str| -> usize { app.lines.iter().position(|l| l == search_str).unwrap() };
        let get_idx = |search_str: &str| -> usize {
            app.lines
                .iter()
                .position(|l| l.starts_with(search_str))
                .unwrap()
        };

        let meta_title_idx = get_idx("Title:");
        let meta_val_idx = get_idx("contact@renecoignard");
        let scene1_idx = get_idx("INT. FLAT");

        let char1_idx = get_exact_idx("RENÉ");

        let paren_idx = get_idx("(turning round)");
        let dial_idx = get_idx("Oh, hello there");
        let boneyard1_idx = get_idx("/* Two sailors");
        let trans1_idx = get_exact_idx("CUT TO:");
        let boneyard_multiline_idx = get_exact_idx("/*");
        let section_idx = get_idx("# This is");
        let syn_idx = get_idx("= And this");
        let inline_note_idx = get_idx("[[Comments");
        let markup_idx = get_idx("As you may have noticed, there's support for");
        let center_idx = get_exact_idx(">Centred text<");
        let force_trans_idx = get_idx(">AN ABRUPT");
        let lyric1_idx = get_idx("~Meine Damen");
        let lyric6_idx = get_idx("~Sänk ju");
        let note_marker_idx = get_idx("[[marker blue");
        let fade_out_idx = get_exact_idx("> FADE OUT");

        assert_eq!(app.types[meta_title_idx], LineType::MetadataTitle);
        assert_eq!(app.types[meta_val_idx], LineType::MetadataValue);
        assert_eq!(app.types[scene1_idx], LineType::SceneHeading);
        assert_eq!(app.types[char1_idx], LineType::Character);
        assert_eq!(app.types[paren_idx], LineType::Parenthetical);
        assert_eq!(app.types[dial_idx], LineType::Dialogue);
        assert_eq!(app.types[boneyard1_idx], LineType::Boneyard);
        assert_eq!(app.types[trans1_idx], LineType::Transition);
        assert_eq!(app.types[boneyard_multiline_idx], LineType::Boneyard);
        assert_eq!(app.types[section_idx], LineType::Section);
        assert_eq!(app.types[syn_idx], LineType::Synopsis);
        assert_eq!(app.types[inline_note_idx], LineType::Note);
        assert_eq!(app.types[center_idx], LineType::Centered);
        assert_eq!(app.types[force_trans_idx], LineType::Transition);
        assert_eq!(app.types[lyric1_idx], LineType::Lyrics);
        assert_eq!(app.types[lyric6_idx], LineType::Lyrics);
        assert_eq!(app.types[note_marker_idx], LineType::Note);
        assert_eq!(app.types[fade_out_idx], LineType::Transition);

        let layout_markup = app
            .layout
            .iter()
            .find(|r| r.line_idx == markup_idx)
            .unwrap();
        assert!(layout_markup.fmt.bold.len() > 0);
        assert!(layout_markup.fmt.italic.len() > 0);
        assert!(layout_markup.fmt.underlined.len() > 0);

        let layout_note = app
            .layout
            .iter()
            .find(|r| r.line_idx == note_marker_idx)
            .unwrap();
        assert!(layout_note.override_color.is_some());
        assert_eq!(
            layout_note.override_color.unwrap(),
            ratatui::style::Color::Blue
        );

        let layout_scene = app
            .layout
            .iter()
            .find(|r| r.line_idx == scene1_idx)
            .unwrap();
        assert_eq!(layout_scene.scene_num.as_deref(), Some("1"));

        let layout_trans = app
            .layout
            .iter()
            .find(|r| r.line_idx == trans1_idx)
            .unwrap();
        let expected_indent = crate::types::PAGE_WIDTH.saturating_sub(7);
        assert_eq!(layout_trans.indent, expected_indent);
        assert_eq!(layout_trans.raw_text, "CUT TO:");

        assert!(app.characters.contains("RENÉ"));
        assert!(app.characters.contains("SAILOR"));
        assert!(app.locations.contains("FLAT IN WOLFEN-NORD - DAY"));

        let total_vis_lines = app.layout.len();
        assert!(total_vis_lines > 0, "Layout must not be empty");

        let test_coordinates: Vec<(usize, usize, String, usize)> = app
            .layout
            .iter()
            .filter_map(|r| {
                if r.is_phantom {
                    None
                } else {
                    Some((r.line_idx, r.char_start, r.raw_text.clone(), r.char_end))
                }
            })
            .collect();

        for (line_idx, char_start, raw_text, char_end) in test_coordinates {
            app.cursor_y = line_idx;
            app.cursor_x = char_start;
            app.report_cursor_position();

            let status = app
                .status_msg
                .as_ref()
                .expect("Status message should be set");

            let line_part = status.split(',').next().unwrap();
            let fraction_part = line_part.split(' ').nth(1).unwrap();

            let cur_line_str = fraction_part.split('/').next().unwrap();
            let reported_line: usize = cur_line_str.parse().unwrap();

            let total_lines_str = fraction_part.split('/').nth(1).unwrap();
            let reported_total: usize = total_lines_str.parse().unwrap();

            assert_eq!(
                reported_line,
                line_idx + 1,
                "Mismatch at logical line {} (text: '{}'). Expected logical line {}, but got {}",
                line_idx,
                raw_text,
                line_idx + 1,
                reported_line
            );

            assert_eq!(
                reported_total,
                app.lines.len(),
                "Total logical lines mismatch at logical line {}",
                line_idx
            );

            app.cursor_x = char_end;
            app.report_cursor_position();
            assert!(
                app.status_msg.is_some(),
                "report_cursor_position panicked or failed at the end of logical line {}",
                line_idx
            );
        }

        let coords: Vec<(usize, usize, usize)> = app
            .layout
            .iter()
            .filter(|r| !r.is_phantom)
            .flat_map(|row| {
                (row.char_start..=row.char_end).map(move |cx| (row.line_idx, cx, row.char_start))
            })
            .collect();

        let mut prev_char = 0usize;
        let mut prev_line = 0usize;

        for (line_idx, cx, _) in coords {
            app.cursor_y = line_idx;
            app.cursor_x = cx;
            app.report_cursor_position();

            let status = app.status_msg.as_ref().unwrap();
            let parts: Vec<&str> = status.split(", ").collect();

            let cur_line: usize = parts[0]
                .split('/')
                .next()
                .unwrap()
                .split_whitespace()
                .nth(1)
                .unwrap()
                .parse()
                .unwrap();
            let cur_char: usize = parts[2]
                .split('/')
                .next()
                .unwrap()
                .split_whitespace()
                .nth(1)
                .unwrap()
                .parse()
                .unwrap();

            assert!(
                cur_line >= prev_line,
                "line went backwards at y={} x={}: {} -> {}",
                line_idx,
                cx,
                prev_line,
                cur_line
            );
            assert!(
                cur_char >= prev_char,
                "char went backwards at y={} x={}: {} -> {}",
                line_idx,
                cx,
                prev_char,
                cur_char
            );

            prev_char = cur_char;
            prev_line = cur_line;
        }

        app.cursor_y = app
            .lines
            .iter()
            .position(|l| l.starts_with("INT. FLAT"))
            .unwrap();
        app.cursor_x = 0;
        app.update_layout();
        app.report_cursor_position();
        assert_eq!(
            app.status_msg.as_deref(),
            Some("line 8/93 (8%), col 1/31 (3%), char 126/4082 (3%)")
        );

        app.cursor_y = app
            .lines
            .iter()
            .position(|l| l.starts_with(">AN ABRUPT"))
            .unwrap();
        app.cursor_x = 0;
        app.update_layout();
        app.report_cursor_position();
        assert_eq!(
            app.status_msg.as_deref(),
            Some("line 67/93 (72%), col 1/41 (2%), char 2976/4082 (72%)")
        );

        app.cursor_y = app.lines.iter().position(|l| l == "> FADE OUT").unwrap();
        app.cursor_x = app.lines[app.cursor_y].chars().count();
        app.update_layout();
        app.report_cursor_position();
        assert_eq!(
            app.status_msg.as_deref(),
            Some("line 93/93 (100%), col 11/11 (100%), char 4082/4082 (100%)")
        );

        app.cursor_y = usize::MAX;
        app.update_layout();

        let render = crate::export::export_document(&app.layout, &app.lines, &app.config, false);

        let reference_render = r#"                      Fount Tutorial
                      Credit: Written by
                      Author: René Coignard
                      Draft date: Version 0.2.17
                      Contact:
                        contact@renecoignard.com

     1      INT. FLAT IN WOLFEN-NORD - DAY                                    1.

            RENÉ sits at his desk, typing.

                                RENÉ
                            (turning round)
                       Oh, hello there. It seems you've
                       found my terminal Rust port of
                       Beat. Sit back and I'll show you
                       how everything works.

            I sometimes write screenplays on my Gentoo laptop, and doing
            it in plain nano isn't terribly comfortable (I work entirely
            in the terminal there). So I decided to put this port of
            Beat together. I used Beat's source code as a reference when
            writing Fount, so things work more or less the same way.

            As you may have already noticed, the navigation is rather
            reminiscent of nano, because I did look at its source code
            and took inspiration, for the sake of authenticity. I'm
            rather fond of it, and I hope you will be too. Not quite as
            nerdy as vim, but honestly, I'm an average nano enjoyer and
            I'm not ashamed of it.

            Anyway, let's get into it.

     2      EXT. NORDPARK - DAY

            As I mentioned, things work much the same as in Beat. If you
            start a line with int. or ext., Fount will automatically
            turn it into a scene heading. You can also use tab: on an
            empty line, it will first turn it into a character cue, then
            a scene heading, and then a transition. If you simply start
            typing IN CAPS ON AN EMPTY LINE, LIKE SO, the text will
            automatically become a character cue.

            You can also use notes...

                                SAILOR
                       I'm not a sailor, actually.

            Fount automatically inserts two blank lines after certain
            elements, just as Beat does, though this can be adjusted in
            the configuration file. There's a sample config in the
            repository; do make use of it. Bonus: try enabling
            typewriter mode and see what happens.

            To create a transition, simply write in capitals and end
            with a colon, like so...

                                                                 CUT TO:

            That alone is quite enough to write a proper screenplay. But
            there's more! For instance, we also have these...                 2.

     3      INT. EDEKA - ABEND

            Unlike Beat, there's no full render or PDF export here, but
            you can always save your screenplay and open it in Beat to
            do that. In Beat, synopses wouldn't appear in the rendered
            script, nor would comments. Which is why they share the same
            colour here, incidentally.

            As you may have noticed, there's support for bold text,
            italics, and even underlined text. When your cursor isn't on
            a line containing these markers, they'll be hidden from
            view. Move onto the line, and you'll see all the asterisks
            and underscores that produce the formatting.

            Centred text is supported as well, and works like this...

                                    Centred text

            You can also force transitions...

                                 AN ABRUPT TRANSITION TO THE NEXT SCENE:

     4      EXT. WOLFEN(BITTERFELD) RAILWAY STATION - MORNING

            Lyrics are supported too, using a tilde at the start of the
            line...

                          Meine Damen, meine Herrn, danke
                              Dass Sie mit uns reisen
                              Zu abgefahrenen Preisen
                              Auf abgefahrenen Gleisen
                   Für Ihre Leidensfähigkeit, danken wir spontan
                      Sänk ju for träweling wis Deutsche Bahn

            That's Wise Guys. Onwards.

     5      EXT. LEIPZIG HBF - MORNING

            Well, do have a go on it, write something from scratch, or
            edit this screenplay. You might even turn up a bug or two;
            if so, please do let me know :-) Everything seemed to behave
            itself while I was putting this tutorial together, and I
            hope it all runs just as smoothly for you. I hope you enjoy
            working in Fount.

            You can find more information about the Fountain markup
            language at https://www.fountain.io/                              3.

            And Beat itself, of course: https://www.beat-app.fi/

                                                                FADE OUT
"#;

        assert_eq!(
            render, reference_render,
            "Reference render does not match expected output."
        );
    }

    #[test]
    fn test_open_scene_navigator_colors() {
        let mut app = create_empty_app();
        app.lines = vec![
            "EXT. WOODS - DAY [[red]]".to_string(),
            "Action line.".to_string(),
            "".to_string(),
            "[[blue]]".to_string(),
            "".to_string(),
            "INT. CABIN - NIGHT".to_string(),
            "[[marker green]]".to_string(),
        ];
        app.parse_document();
        app.update_layout();
        app.open_scene_navigator();

        assert_eq!(app.scenes.len(), 2);
        assert_eq!(app.scenes[0].label, "EXT. WOODS - DAY");
        assert_eq!(app.scenes[0].color, Some(Color::Red));
        assert_eq!(app.scenes[1].label, "INT. CABIN - NIGHT");
        assert_eq!(app.scenes[1].color, Some(Color::Blue));
        
        app.lines = vec![
            "INT. CABIN - NIGHT".to_string(),
            "Action here.".to_string(),
            "[[marker magenta]]".to_string(),
        ];
        app.parse_document();
        app.update_layout();
        app.open_scene_navigator();
        assert_eq!(app.scenes[0].color, Some(Color::Magenta));
    }

    #[test]
    fn test_forced_uppercase_transformation() {
        let mut app = create_empty_app();
        app.lines = vec![
            "ext. woods - day".to_string(), // Scene Heading
            "Action line.".to_string(),
            "".to_string(),
            "@john".to_string(),            // Character
            "He waits.".to_string(),
            "".to_string(),
            "cut to:".to_string(),          // Transition
        ];
        app.parse_document();

        assert_eq!(app.lines[0], "EXT. WOODS - DAY");
        assert_eq!(app.lines[3], "@JOHN");
        assert_eq!(app.lines[6], "CUT TO:");
        assert_eq!(app.lines[1], "Action line."); // Should stay original
    }

    // ── Structural Locking (Production Mode) Tests ──────────────────────

    #[test]
    fn test_increment_suffix_basic() {
        assert_eq!(App::increment_suffix("A"), "B");
        assert_eq!(App::increment_suffix("B"), "C");
        assert_eq!(App::increment_suffix("Y"), "Z");
    }

    #[test]
    fn test_increment_suffix_wrap() {
        assert_eq!(App::increment_suffix("Z"), "AA");
        assert_eq!(App::increment_suffix("AZ"), "BA");
        assert_eq!(App::increment_suffix("ZZ"), "AAA");
    }

    #[test]
    fn test_next_suffix_label_empty() {
        let existing: Vec<String> = vec![];
        assert_eq!(App::next_suffix_label(&existing), "A");
    }

    #[test]
    fn test_next_suffix_label_after_a_b() {
        let existing = vec!["A".to_string(), "B".to_string()];
        assert_eq!(App::next_suffix_label(&existing), "C");
    }

    #[test]
    fn test_next_suffix_label_after_z() {
        let existing = vec!["Z".to_string()];
        assert_eq!(App::next_suffix_label(&existing), "AA");
    }

    #[test]
    fn test_extract_scene_tag() {
        assert_eq!(
            App::extract_scene_tag("INT. ROOM - DAY #5#"),
            Some("5".to_string())
        );
        assert_eq!(
            App::extract_scene_tag("INT. ROOM - DAY #5A#"),
            Some("5A".to_string())
        );
        assert_eq!(App::extract_scene_tag("INT. ROOM - DAY"), None);
        assert_eq!(App::extract_scene_tag(""), None);
    }

    #[test]
    fn test_production_lock_auto_suffix_single_insertion() {
        let mut app = create_empty_app();
        app.lines = vec![
            "".to_string(),
            "INT. ROOM - DAY #1#".to_string(),
            "".to_string(),
            "INT. HALLWAY - NIGHT".to_string(), // No tag — should get #1A#
            "".to_string(),
            "INT. KITCHEN - DAY #2#".to_string(),
        ];
        app.config.production_lock = true;
        app.parse_document();

        assert_eq!(
            App::extract_scene_tag(&app.lines[3]),
            Some("1A".to_string()),
            "New scene between #1# and #2# should get #1A#"
        );
        // Existing scenes should be untouched
        assert_eq!(App::extract_scene_tag(&app.lines[1]), Some("1".to_string()));
        assert_eq!(App::extract_scene_tag(&app.lines[5]), Some("2".to_string()));
    }

    #[test]
    fn test_production_lock_auto_suffix_multiple_insertions() {
        let mut app = create_empty_app();
        app.lines = vec![
            "".to_string(),
            "INT. ROOM - DAY #5#".to_string(),
            "".to_string(),
            "INT. HALLWAY #5A#".to_string(), // Already suffixed
            "".to_string(),
            "INT. CLOSET".to_string(), // Should get #5B#
            "".to_string(),
            "INT. KITCHEN - DAY #6#".to_string(),
        ];
        app.config.production_lock = true;
        app.parse_document();

        assert_eq!(
            App::extract_scene_tag(&app.lines[5]),
            Some("5B".to_string()),
            "Second insertion between #5# and #6# should get #5B#"
        );
        // Existing tags remain
        assert_eq!(App::extract_scene_tag(&app.lines[1]), Some("5".to_string()));
        assert_eq!(App::extract_scene_tag(&app.lines[3]), Some("5A".to_string()));
        assert_eq!(App::extract_scene_tag(&app.lines[7]), Some("6".to_string()));
    }

    #[test]
    fn test_production_lock_scene_before_first_numbered() {
        let mut app = create_empty_app();
        app.lines = vec![
            "".to_string(),
            "INT. PROLOGUE".to_string(), // No tag, before first numbered scene
            "".to_string(),
            "INT. ROOM - DAY #1#".to_string(),
        ];
        app.config.production_lock = true;
        app.parse_document();

        assert_eq!(
            App::extract_scene_tag(&app.lines[1]),
            Some("0A".to_string()),
            "Scene before first numbered scene should use base 0"
        );
    }

    #[test]
    fn test_production_lock_scene_after_last_numbered() {
        let mut app = create_empty_app();
        app.lines = vec![
            "".to_string(),
            "INT. ROOM - DAY #10#".to_string(),
            "".to_string(),
            "INT. EPILOGUE".to_string(), // No tag, after last numbered
        ];
        app.config.production_lock = true;
        app.parse_document();

        assert_eq!(
            App::extract_scene_tag(&app.lines[3]),
            Some("10A".to_string()),
            "Scene after last numbered should suffix from that number"
        );
    }

    #[test]
    fn test_production_lock_off_no_auto_numbering() {
        let mut app = create_empty_app();
        app.lines = vec![
            "".to_string(),
            "INT. ROOM - DAY #1#".to_string(),
            "".to_string(),
            "INT. HALLWAY - NIGHT".to_string(),
            "".to_string(),
            "INT. KITCHEN - DAY #2#".to_string(),
        ];
        app.config.production_lock = false;
        app.parse_document();

        assert_eq!(
            App::extract_scene_tag(&app.lines[3]),
            None,
            "With lock OFF, un-numbered scenes should stay un-numbered"
        );
    }

    #[test]
    fn test_locknum_does_not_renumber() {
        let mut app = create_empty_app();
        app.lines = vec![
            "".to_string(),
            "INT. ROOM - DAY #5#".to_string(), // Custom number
            "".to_string(),
            "INT. HALLWAY - NIGHT #10#".to_string(), // Custom number
        ];
        app.parse_document();
        app.update_layout();

        let mut changed = false;
        let mut moved = false;
        let mut update = false;
        app.command_input = "locknum".to_string();
        app.mode = AppMode::Command;
        let _ = app.execute_command(&mut changed, &mut moved, &mut update);

        assert!(app.config.production_lock);
        // Custom numbers should NOT be overwritten
        assert_eq!(App::extract_scene_tag(&app.lines[1]), Some("5".to_string()));
        assert_eq!(App::extract_scene_tag(&app.lines[3]), Some("10".to_string()));
    }

    #[test]
    fn test_renum_works_regardless_of_lock() {
        let mut app = create_empty_app();
        app.lines = vec![
            "".to_string(),
            "INT. ROOM - DAY #5#".to_string(),
            "".to_string(),
            "INT. HALLWAY - NIGHT #10#".to_string(),
        ];
        app.config.production_lock = true;
        app.parse_document();
        app.update_layout();

        app.renumber_all_scenes();

        // /renum should override regardless of lock
        assert_eq!(App::extract_scene_tag(&app.lines[1]), Some("1".to_string()));
        assert_eq!(App::extract_scene_tag(&app.lines[3]), Some("2".to_string()));
    }
