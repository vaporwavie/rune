use rune::{Cx, El, Env, Runtime};
use rune_kit::{Button, ButtonVariant, Kit};

struct S {
    label: String,
    rolling: bool,
}

fn view(s: &S, _: &mut Cx<S>) -> El {
    let k = Kit::neutral(true);
    let b = Button::new(s.label.clone());
    let b = if s.rolling { b.rolling("import-label") } else { b };
    rune::row().padding(20.0).child(b.build(&k).key("button"))
}

fn runtime(label: &str, rolling: bool) -> Runtime<S> {
    let mut rt = Runtime::new(S { label: label.into(), rolling }, view, (400.0, 100.0), Env::new(true, 2.0));
    rt.frame(0.0);
    rt
}

fn rect<T>(rt: &Runtime<T>, key: &str) -> [f32; 4] {
    rt.presented(key).unwrap_or_else(|| panic!("no {key}")).0
}

#[test]
fn a_rolling_label_sizes_the_button_as_plain_text_does() {
    for label in ["Import 3 items", "Importing…"] {
        let rolling = rect(&runtime(label, true), "button");
        let plain = rect(&runtime(label, false), "button");
        assert!((rolling[2] - plain[2]).abs() < 0.5, "{label}: {rolling:?} vs {plain:?}");
        assert_eq!(rolling[3], plain[3]);
    }
}

#[test]
fn a_custom_label_gets_the_buttons_font_and_ink() {
    fn custom(_: &(), _: &mut Cx<()>) -> El {
        let k = Kit::neutral(true);
        rune::row()
            .child(Button::new("Save").build(&k).key("plain"))
            .child(Button::new("Save").label_with(|font, ink| rune::text("Save").key("custom").font(font).color(ink)).build(&k).key("rich"))
    }
    let mut rt = Runtime::new((), custom, (400.0, 100.0), Env::new(true, 2.0));
    rt.frame(0.0);
    let els = rt.elements();
    let find = |path: &str| els.iter().find(|e| e.path.ends_with(path)).unwrap_or_else(|| panic!("no {path}"));
    assert_eq!(find("rich").label.as_deref(), Some("Save"));
    assert_eq!(find("rich/custom").text_color, find("plain/label").text_color);
    assert_eq!(find("rich").rect[2], find("plain").rect[2]);
}

#[test]
fn the_button_springs_with_its_rolling_label_and_keeps_its_accessible_label() {
    let mut rt = runtime("Import 3 items", true);
    let info = |rt: &Runtime<S>| rt.elements().into_iter().find(|e| e.key.as_deref() == Some("button")).unwrap();
    assert_eq!((info(&rt).role, info(&rt).label.as_deref()), ("button", Some("Import 3 items")));
    let from = rect(&rt, "button");
    // The label's right edge sits this far inside the button's at rest.
    let inset = from[0] + from[2] - (rect(&rt, "import-label")[0] + rect(&rt, "import-label")[2]);
    let to = rect(&runtime("Importing…", false), "button");
    assert!(from[2] - to[2] > 10.0, "the label shrinks: {from:?} → {to:?}");

    rt.state.label = "Importing…".into();
    rt.invalidate();
    rt.frame(0.0);
    assert_eq!(info(&rt).label.as_deref(), Some("Importing…"));
    let mut widths = vec![rect(&rt, "button")[2]];
    for _ in 0..120 {
        rt.frame(1.0 / 120.0);
        let (b, l) = (rect(&rt, "button"), rect(&rt, "import-label"));
        widths.push(b[2]);
        let gap = b[0] + b[2] - (l[0] + l[2]);
        assert!((gap - inset).abs() < 0.5, "the padding holds as both spring: {gap} vs {inset}");
    }
    assert!((widths[0] - from[2]).abs() < 1.0, "no snap: starts at the old width {} vs {}", widths[0], from[2]);
    assert!(widths.windows(2).all(|w| (w[1] - w[0]).abs() < 4.0), "no jumps: {widths:?}");
    assert!(widths[12] < from[2] - 1.0 && widths[12] > to[2] + 1.0, "springs between: {}", widths[12]);
    assert!((widths.last().unwrap() - to[2]).abs() < 0.5, "lands on the new width: {widths:?}");
}

#[test]
fn a_rolling_button_with_an_icon_and_neighbour_springs_without_jumps() {
    fn row_view(label: &String, _: &mut Cx<String>) -> El {
        let k = Kit::neutral(true);
        rune::row()
            .padding(20.0)
            .gap(8.0)
            .child(Button::new(label.clone()).icon(&k.glyph.lock).rolling("import-row-label").build(&k).key("button"))
            .child(Button::new("Cancel").build(&k).key("cancel"))
            .child(Button::new(label.clone()).icon(&k.glyph.lock).rolling("fixed-row-label").build(&k).key("fixed").w(180.0))
    }
    let mut rt = Runtime::new("Import 3 items".to_owned(), row_view, (500.0, 100.0), Env::new(true, 2.0));
    rt.frame(0.0);
    let icon_gap = |rt: &Runtime<String>| {
        let els = rt.elements();
        let ic = els.iter().find(|e| e.path.ends_with("fixed/icon")).unwrap().rect;
        let lb = els.iter().find(|e| e.key.as_deref() == Some("fixed-row-label")).unwrap().rect;
        lb[0] - (ic[0] + ic[2])
    };
    let rest_icon_gap = icon_gap(&rt);
    let cancel_from = rect(&rt, "cancel")[0];

    rt.state = "Importing…".into();
    rt.invalidate();
    rt.frame(0.0);
    let mut cancel_xs = vec![rect(&rt, "cancel")[0]];
    for _ in 0..120 {
        rt.frame(1.0 / 120.0);
        cancel_xs.push(rect(&rt, "cancel")[0]);
        let g = icon_gap(&rt);
        assert!((g - rest_icon_gap).abs() < 0.6, "icon and rolling label move on the same curve: {g} vs {rest_icon_gap}");
    }
    assert!((cancel_xs[0] - cancel_from).abs() < 1.0, "neighbour starts at its old x: {} vs {cancel_from}", cancel_xs[0]);
    assert!(cancel_xs.windows(2).all(|x| (x[1] - x[0]).abs() < 4.0), "neighbour slides with no jump: {cancel_xs:?}");
}

#[test]
fn a_shown_shortcut_follows_the_label_in_faint_text_and_spells_escape() {
    fn view(_: &(), _: &mut Cx<()>) -> El {
        let k = Kit::neutral(true);
        rune::row()
            .gap(8.0)
            .child(Button::new("Cancel").variant(ButtonVariant::Ghost).shortcut("esc").show_shortcut().build(&k).key("cancel"))
            .child(Button::new("Close").shortcut("⎋").show_shortcut().build(&k).key("close"))
            .child(Button::new("Open").variant(ButtonVariant::Contrast).shortcut("⏎").show_shortcut().build(&k).key("open"))
            .child(Button::new("Save").shortcut("⌘S").build(&k).key("save"))
    }
    let k = Kit::neutral(true);
    let mut rt = Runtime::new((), view, (600.0, 100.0), Env::new(true, 2.0));
    rt.frame(0.0);
    let els = rt.elements();
    let find = |path: &str| els.iter().find(|e| e.path.ends_with(path)).unwrap_or_else(|| panic!("no {path}"));
    let esc = if cfg!(target_os = "macos") { "esc" } else { "Esc" };

    let (button, label, shortcut) = (find("cancel"), find("cancel/label"), find("cancel/shortcut"));
    assert_eq!(shortcut.text.as_deref(), Some(esc));
    assert!(shortcut.rect[0] > label.rect[0] + label.rect[2], "the shortcut follows the label");
    assert!(shortcut.rect[0] + shortcut.rect[2] <= button.rect[0] + button.rect[2], "inside the button");
    assert_eq!(shortcut.text_color, Some(k.c.text_faint), "faint, as a menu item's shortcut");
    assert_eq!(button.label.as_deref(), Some("Cancel"), "a reader hears the label, and the shortcut as a hint");
    assert_eq!(button.help.as_deref(), Some(format!("Cancel  {esc}").as_str()));

    assert_eq!(find("close/shortcut").text.as_deref(), Some("esc"), "⎋ is spelled out wherever it is drawn");
    assert_eq!(find("close").help.as_deref(), Some("Close  esc"));
    assert_eq!(find("open/shortcut").text_color, Some(k.alpha(k.c.text_contrast, 0.6)), "faint text wouldn't read on a contrast fill");
    assert!(els.iter().all(|e| !e.path.ends_with("save/shortcut")), "a shortcut shows only when asked");
}
