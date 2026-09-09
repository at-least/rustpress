//! Spike: hypertext rsx! syntax for the shapes the renderer needs
//! (Alpine attributes, raw SVG, conditional attrs, layout fns taking
//! bodies). Delete once the renderer is real.

use hypertext::prelude::*;
use hypertext::Raw;

#[test]
fn alpine_attributes_compile_and_render() {
    let html = rsx! {
        <div x-data="{ open: false }" x-on:click="open = !open" x-show="open" x-cloak>
            "hi"
        </div>
    }
    .render().into_inner().to_string();
    assert!(html.contains("x-data"), "{html}");
    assert!(html.contains("x-on:click"), "{html}");
    assert!(html.contains("x-show"), "{html}");
    assert!(html.contains("x-cloak"), "{html}");
}

#[test]
fn alpine_shorthand_symbols() {
    let html = rsx! {
        <button @click="toggle()" :class="active ? 'on' : 'off'">
            "b"
        </button>
    }
    .render().into_inner().to_string();
    println!("{html}");
}

#[test]
fn conditional_and_dynamic_attrs() {
    let active = true;
    let inactive = !active;
    let id = "menu";
    let html = rsx! {
        <div id={ "btn-"(id) } aria-expanded="true"[active] hidden[inactive] class="VPFlyout">
            "x"
        </div>
    }
    .render().into_inner().to_string();
    assert!(html.contains("id=\"btn-menu\""), "{html}");
    assert!(html.contains("aria-expanded"), "{html}");
    assert!(!html.contains("hidden"), "{html}");
}

#[test]
fn raw_svg_splices() {
    let icon = Raw::dangerously_create(r#"<svg class="icon"><path d="M1"></path></svg>"#);
    let html = rsx! {
        <span class="icon-wrap">(icon)</span>
    }
    .render().into_inner().to_string();
    assert!(html.contains("<svg"), "{html}");
}

#[test]
fn layout_fn_takes_renderable_body() {
    fn layout<'a>(title: &'a str, body: impl Renderable + 'a) -> impl Renderable + 'a {
        rsx! {
            <html lang="en">
                <head>(title)</head>
                <body>(body)</body>
            </html>
        }
    }
    let inner = rsx! { <p>"content"</p> };
    let html = layout("T", inner).render().into_inner().to_string();
    assert!(html.contains("<html"), "{html}");
    assert!(html.contains("content"), "{html}");
}

#[test]
fn void_elements_and_class_literals() {
    let html = rsx! {
        <div class="vp-doc container">
            <br>
            <img src="x.png" alt="">
        </div>
    }
    .render().into_inner().to_string();
    assert!(html.contains("class=\"vp-doc container\""), "{html}");
}

#[test]
fn attribute_paren_expressions() {
    let cls = format!("a-{}", 42);
    let html = rsx! {
        <div class=(cls) id=(format!("id-{}", 7))>( "text" )</div>
    }
    .render().into_inner().to_string();
    assert!(html.contains("a-42"), "{html}");
    assert!(html.contains("id-7"), "{html}");
    assert!(html.contains("text"), "{html}");
}
