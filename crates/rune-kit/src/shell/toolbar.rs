//! A window's toolbar: leading, centre and trailing slots in a band the window drags by.

use rune::{El, SystemMaterial, el, row, text};

use crate::Kit;

/// The toolbar band's height, macOS 26's unified toolbar. The traffic lights centre in it when
/// the window places them at y = (48 - 14) / 2 = 17.
pub const TOOLBAR_HEIGHT: f32 = 48.0;
/// Padding at the band's ends.
const PAD: f32 = 12.0;

/// A toolbar: `Toolbar::new().leading(..).title("..").trailing(..).build(&kit)`. The band is a
/// window drag region (buttons in it keep their presses) and its centre slot sits in the
/// middle of the band whatever the sides hold.
pub struct Toolbar {
    leading: Vec<El>,
    center: Option<El>,
    title: Option<String>,
    trailing: Vec<El>,
    pad: Option<f32>,
    height: Option<f32>,
    divider: bool,
    material: Option<SystemMaterial>,
}

impl Toolbar {
    pub fn new() -> Toolbar {
        Toolbar { leading: Vec::new(), center: None, title: None, trailing: Vec::new(), pad: None, height: None, divider: false, material: None }
    }
    pub fn leading(mut self, e: El) -> Self {
        self.leading.push(e);
        self
    }
    pub fn trailing(mut self, e: El) -> Self {
        self.trailing.push(e);
        self
    }
    /// The centre slot. Replaces a [`Toolbar::title`].
    pub fn center(mut self, e: El) -> Self {
        self.center = Some(e);
        self.title = None;
        self
    }
    /// A centred title in the kit's medium UI type.
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self.center = None;
        self
    }
    /// The band's height (default [`TOOLBAR_HEIGHT`]). A `SplitView` gives its sidebar's title
    /// bar row the same height and centres the sidebar toggle in it. The traffic lights are
    /// the window's: an app that changes the height moves them in its `Chrome` to match.
    pub fn height(mut self, points: f32) -> Self {
        self.height = Some(points);
        self
    }
    /// A hairline along the band's bottom edge, for a band over content that scrolls under it.
    /// It is drawn inside the band and takes no room.
    pub fn divider(mut self) -> Self {
        self.divider = true;
        self
    }
    /// Backs the band with a system material (macOS blurs what's behind a transparent
    /// window). Elsewhere, and headless, the kit's base background draws instead.
    pub fn material(mut self, m: SystemMaterial) -> Self {
        self.material = Some(m);
        self
    }

    /// The leading padding a `SplitView` works out (traffic lights and its sidebar toggle).
    pub(super) fn padding(mut self, points: f32) -> Self {
        self.pad = Some(points);
        self
    }

    pub(super) fn band_height(&self) -> f32 {
        self.height.unwrap_or(TOOLBAR_HEIGHT)
    }

    pub fn build(self, k: &Kit) -> El {
        let c = &k.c;
        let pl = self.pad.unwrap_or(PAD);
        let height = self.band_height();
        let mut band =
            row().key("toolbar").role("toolbar").w_full().h(height).pl(pl).pr(PAD).gap(8.0).center_y().window_drag();
        if let Some(t) = &self.title {
            band = band.label(t.clone());
        }
        let center = match (self.center, self.title) {
            (Some(e), _) => Some(e),
            (None, Some(t)) => Some(text(t).key("title").font(k.ty.ui_medium).color(c.text_base).truncate()),
            _ => None,
        };
        if let Some(center) = center {
            // Across the whole band and centred in it, so the sides can differ in width. First,
            // so the side slots draw and take presses above it.
            band = band.child(
                row().key("center").pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).center().px(96.0).child(center),
            );
        }
        band = band
            // The leading slot gives way first, so a long title truncates instead of pushing the
            // trailing controls off the band.
            .child(row().key("leading").gap(4.0).center_y().shrink().min_w(0.0).children(self.leading))
            .child(row().key("spacer").grow(1.0).a11y_hidden())
            .child(row().key("trailing").gap(4.0).center_y().children(self.trailing));
        if self.divider {
            band = band
                .child(el().key("toolbar-divider").pin(Some(0.0), None, Some(0.0), Some(0.0)).h(1.0).fill(c.border_muted).a11y_hidden());
        }
        if let Some(m) = self.material {
            band = band.system_material(m).fill(c.bg_base);
        }
        band
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}
