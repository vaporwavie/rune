#![cfg(feature = "markdown")]

use rune::{Cx, El, Env, Key, KeyEvent, Mods, Runtime, col};
use rune_kit::{Kit, Markdown, MarkdownStyle};

fn view(_: &(), _: &mut Cx<()>) -> El {
    let src = "Before.\n\n- First paragraph.\n\n  Second paragraph.\n\n- Next item.\n\n```text\nwide code\n```\n\nAfter.";
    col().w_full().child(
        Markdown::new(src)
            .selectable(true)
            .block_keys([(0, "before"), (src.find("- First").unwrap(), "items"), (src.find("```text").unwrap(), "code"), (src.find("After.").unwrap(), "after")])
            .style(MarkdownStyle::neutral(true).prose_width(240.0))
            .build(&Kit::neutral(true))
            .key("note"),
    )
}

#[test]
fn capped_prose_keeps_code_wide_and_copies_list_paragraphs_once() {
    let mut rt = Runtime::new((), view, (640.0, 800.0), Env::new(true, 2.0));
    let clipboard = rune::clipboard::memory();
    rt.use_clipboard(&clipboard);
    rt.frame(0.0);
    let width = |key: &str| {
        rt.elements().into_iter().find(|e| e.path.ends_with(&format!("note/{key}"))).unwrap().rect[2]
    };
    for key in ["before", "items", "after"] {
        assert!(width(key) <= 240.0, "{key} stays within the prose measure");
    }
    assert!(width("code") > 240.0, "code keeps the full column");
    rt.update(|_, tx| tx.select_all_text("note"));
    rt.frame(1.0 / 60.0);
    rt.key(KeyEvent { key: Key::Command('c'), mods: Mods { cmd: true, ..Mods::default() } });
    rt.frame(1.0 / 60.0);
    assert_eq!(
        clipboard.text().as_deref().map(String::as_str),
        Some("Before.\n\n- First paragraph.\n\nSecond paragraph.\n\n- Next item.\n\nwide code\n\nAfter."),
    );
}
