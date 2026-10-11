use dioxus::prelude::*;

/// Scrollable list that only renders the rows that are currently visible.
/// All rows must have the same height (set via CSS on `.vrow`).
///
/// `render_row` keeps its identity across renders, so the visible rows are only re-rendered when
/// `count` or `deps` changes. `deps` must hold every value that `render_row` reads from its
/// surroundings (e.g. the selected row).
#[component]
pub fn VirtualList<D: Clone + PartialEq + 'static>(
    count: usize,
    deps: D,
    render_row: Callback<usize, Element>,
    #[props(into, default)] class: String,
) -> Element {
    let _ = deps;
    const OVERSCAN: usize = 8;
    const INITIAL_VISIBLE_ROWS: usize = 16;

    // (scroll_top, scroll_height, client_height) of the scroll container.
    // Rows are located proportionally, so this is unaffected by CSS zoom.
    let mut scroll = use_signal(|| (0.0f64, 0.0f64, 0.0f64));
    let (scroll_top, scroll_height, client_height) = scroll();

    let (first, visible) = if scroll_height > 0.0 && count > 0 {
        let row_height = scroll_height / count as f64;
        let first = (scroll_top / row_height).floor() as usize;
        let visible = (client_height / row_height).ceil() as usize;
        (first, visible)
    } else {
        (0, INITIAL_VISIBLE_ROWS)
    };
    let start = first.saturating_sub(OVERSCAN).min(count);
    let end = (first + visible + OVERSCAN).min(count);

    rsx! {
        div {
            class: "vlist {class}",
            onscroll: move |event| {
                let data = event.data();
                scroll
                    .set((
                        data.scroll_top(),
                        f64::from(data.scroll_height()),
                        f64::from(data.client_height()),
                    ));
            },
            div {
                class: "vlist-inner",
                style: "--rows: {count}; --start: {start};",
                div { class: "vlist-window",
                    for index in start..end {
                        {render_row.call(index)}
                    }
                }
            }
        }
    }
}
