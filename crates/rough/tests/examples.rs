//! The examples of hivevm/rough (`RoughTest`, `HachureTest`, `BezierTest`, `ParserTest`), drawn
//! as SVG through the rough engine, and the rules of its input.

use nectar_core::{CodeEngine, EngineOutput};
use nectar_rough::Rough;
use usvg::roxmltree;

fn engine() -> Rough {
    Rough::new(nectar_pdf::FONT_FILES)
}

fn svg(body: &str) -> String {
    format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"512\" height=\"512\">{body}</svg>")
}

fn sketch(body: &str) -> String {
    engine().sketch(&svg(body)).unwrap()
}

/// The attributes of every `<path>` of `sketch`, in order.
fn paths(sketch: &str) -> Vec<Vec<(String, String)>> {
    let document = roxmltree::Document::parse(sketch).unwrap();
    assert!(
        document
            .root_element()
            .children()
            .filter(|n| n.is_element())
            .all(|n| n.tag_name().name() == "path"),
        "{sketch}"
    );
    document
        .descendants()
        .filter(|n| n.has_tag_name("path"))
        .map(|n| {
            n.attributes()
                .map(|a| (a.name().to_string(), a.value().to_string()))
                .collect()
        })
        .collect()
}

fn attribute<'a>(path: &'a [(String, String)], name: &str) -> Option<&'a str> {
    path.iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v.as_str())
}

/// The bounding box of the geometry of `svg`, strokes not included: left, top, right, bottom.
fn bounds(svg: &str) -> (f32, f32, f32, f32) {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).unwrap();
    let rect = tree.root().abs_bounding_box();
    (rect.left(), rect.top(), rect.right(), rect.bottom())
}

/// Asserts that the sketch of `body` stays within `tolerance` of the shape's own bounding box
/// and reaches into each of its edges.
fn assert_traces(body: &str, tolerance: f32) {
    let (l, t, r, b) = bounds(&svg(body));
    let (sl, st, sr, sb) = bounds(&sketch(body));
    for (edge, own, sketched) in [
        ("left", l, sl),
        ("top", t, st),
        ("right", r, sr),
        ("bottom", b, sb),
    ] {
        assert!(
            (own - sketched).abs() <= tolerance,
            "{body}: {edge} edge {sketched} is not within {tolerance} of {own}"
        );
    }
}

/// The shapes of `RoughTest`; Rough.js's circle takes a diameter, SVG's a radius.
const ROUGH_TEST: &[&str] = &[
    r##"<rect x="10" y="10" width="200" height="200" fill="none" stroke="#000"/>"##,
    r##"<circle cx="80" cy="120" r="25" fill="none" stroke="#000"/>"##,
    r##"<ellipse cx="300" cy="100" rx="75" ry="40" fill="none" stroke="#000"/>"##,
    r##"<line x1="80" y1="120" x2="300" y2="100" stroke="#000"/>"##,
    r##"<path d="M80 80 A 45 45, 0, 0, 0, 125 125 L 125 80 Z" fill="none" stroke="#000"/>"##,
    r##"<path d="M230 80 A 45 45, 0, 1, 0, 275 125 L 275 80 Z" fill="none" stroke="#000"/>"##,
    r##"<path d="M80 230 A 45 45, 0, 0, 1, 125 275 L 125 230 Z" fill="none" stroke="#000"/>"##,
    r##"<path d="M230 230 A 45 45, 0, 1, 1, 275 275 L 275 230 Z" fill="none" stroke="#000"/>"##,
];

// Verifies: G-12, ADR-0019
#[test]
fn every_shape_of_rough_test_comes_back_as_sketched_paths_in_its_own_colour() {
    for shape in ROUGH_TEST {
        assert_traces(shape, 6.0);
        let paths = paths(&sketch(shape));
        assert!(!paths.is_empty(), "{shape}");
        for path in &paths {
            assert_eq!(attribute(path, "stroke"), Some("#000000"), "{shape}");
            assert_eq!(attribute(path, "fill"), Some("none"), "{shape}");
            // Rough.js strokes every outline twice.
            let moves = attribute(path, "d").unwrap().matches('M').count();
            assert!(moves >= 2, "{shape}: {moves} strokes");
        }
    }
    let blue = paths(&sketch(
        r#"<line x1="0" y1="0" x2="50" y2="50" stroke="blue" stroke-width="3"/>"#,
    ));
    assert_eq!(attribute(&blue[0], "stroke"), Some("#0000ff"));
    assert_eq!(attribute(&blue[0], "stroke-width"), Some("3"));
}

// Verifies: G-12, ADR-0019
#[test]
fn the_whole_rough_test_drawing_is_one_sketch() {
    let mut body = ROUGH_TEST.concat();
    body.push_str(
        r##"<rect x="120" y="105" width="80" height="80" fill="rgba(255,0,200,0.2)" stroke="#000"
             data-rough-fill-style="zigzag" data-rough-hachure-angle="60" data-rough-hachure-gap="8"/>"##,
    );
    let sketch = sketch(&body);
    let paths = paths(&sketch);
    let fills: Vec<_> = paths
        .iter()
        .filter(|p| attribute(p, "stroke") == Some("#ff00c8"))
        .collect();
    assert_eq!(fills.len(), 1, "{sketch}");
    assert_eq!(attribute(fills[0], "stroke-opacity"), Some("0.2"));
    assert_eq!(paths.len(), ROUGH_TEST.len() + 2, "{sketch}");
}

/// The fills `RoughTest` keeps commented out, each with what its sketch must show.
// Verifies: G-12, ADR-0019
#[test]
fn the_fills_of_rough_test_are_drawn_in_their_colours_and_weights() {
    // rc.circle(50, 50, 80, {fill: 'red'}): a hachure of red lines.
    let circle = paths(&sketch(
        r#"<circle cx="50" cy="50" r="40" fill="red" stroke="black"/>"#,
    ));
    assert_eq!(attribute(&circle[0], "stroke"), Some("#ff0000"));
    assert_eq!(attribute(&circle[0], "fill"), Some("none"));
    assert_eq!(attribute(&circle[1], "stroke"), Some("#000000"));

    // rc.rectangle(120, 15, 80, 80, {fill: 'red'}).
    let rectangle = paths(&sketch(
        r#"<rect x="120" y="15" width="80" height="80" fill="red" stroke="black"/>"#,
    ));
    assert_eq!(attribute(&rectangle[0], "stroke"), Some("#ff0000"));
    assert_eq!(attribute(&rectangle[1], "stroke"), Some("#000000"));

    // rc.circle(50, 150, 80, {fill: "rgb(10,150,10)", fillWeight: 3}).
    let weighted = paths(&sketch(
        r#"<circle cx="50" cy="150" r="40" fill="rgb(10,150,10)" stroke="black" data-rough-fill-weight="3"/>"#,
    ));
    assert_eq!(attribute(&weighted[0], "stroke"), Some("#0a960a"));
    assert_eq!(attribute(&weighted[0], "stroke-width"), Some("3"));

    // rc.rectangle(220, 15, 80, 80, {fill: 'red', hachureAngle: 60, hachureGap: 8}).
    let plain = r#"<rect x="220" y="15" width="80" height="80" fill="red" stroke="black"/>"#;
    let angled = r#"<rect x="220" y="15" width="80" height="80" fill="red" stroke="black"
        data-rough-hachure-angle="60" data-rough-hachure-gap="8"/>"#;
    let hachure = |body| {
        attribute(&paths(&sketch(body))[0], "d")
            .unwrap()
            .to_string()
    };
    assert_ne!(hachure(plain), hachure(angled));
    assert_traces(angled, 6.0);

    // rc.rectangle(120, 105, 80, 80, {fill: 'rgba(255,0,200,0.2)', fillStyle: 'solid'}).
    let solid = paths(&sketch(
        r#"<rect x="120" y="105" width="80" height="80" fill="rgba(255,0,200,0.2)" stroke="black"
            data-rough-fill-style="solid"/>"#,
    ));
    assert_eq!(attribute(&solid[0], "fill"), Some("#ff00c8"));
    assert_eq!(attribute(&solid[0], "fill-opacity"), Some("0.2"));
    assert_eq!(attribute(&solid[0], "stroke"), Some("none"));
}

// Verifies: G-12, ADR-0019
#[test]
fn each_of_the_seven_fill_styles_gives_a_pattern_of_its_own() {
    let styles = [
        "hachure",
        "solid",
        "zigzag",
        "cross-hatch",
        "dots",
        "dashed",
        "zigzag-line",
    ];
    let mut fills = Vec::new();
    for style in styles {
        let body = format!(
            r#"<rect x="10" y="10" width="120" height="80" fill="red" data-rough-fill-style="{style}"/>"#
        );
        assert_traces(&body, 8.0);
        let paths = paths(&sketch(&body));
        assert_eq!(
            paths.len(),
            1,
            "{style}: a shape without stroke has its fill alone"
        );
        let colour = if style == "solid" { "fill" } else { "stroke" };
        assert_eq!(attribute(&paths[0], colour), Some("#ff0000"), "{style}");
        fills.push(attribute(&paths[0], "d").unwrap().to_string());
    }
    for (i, a) in fills.iter().enumerate() {
        for (j, b) in fills.iter().enumerate().skip(i + 1) {
            assert_ne!(a, b, "{} and {} draw the same", styles[i], styles[j]);
        }
    }
}

// Verifies: G-12, ADR-0019
#[test]
fn the_polygon_of_hachure_test_is_filled_with_lines_at_its_angle_and_gap() {
    let body = r#"<polygon points="10,10 200,10 100,100 300,100 60,200" fill="teal"
        data-rough-hachure-angle="45" data-rough-hachure-gap="10"/>"#;
    assert_traces(body, 8.0);
    let paths = paths(&sketch(body));
    let lines = attribute(&paths[0], "d").unwrap().matches('M').count();
    // About 30 lines 10 apart cross the polygon at 45°, each drawn twice.
    assert!(lines >= 40, "{lines} strokes");
}

// Verifies: G-12, ADR-0019
#[test]
fn the_curves_of_bezier_test_and_the_paths_of_parser_test_are_traced() {
    for body in [
        // BezierTest: a cubic curve, the curve through six points as one, and pointsOnPath.
        r#"<path d="M70 240 C145 60 275 90 300 230" fill="none" stroke="black"/>"#,
        r#"<polyline points="20,240 95,69 225,90 250,180 290,220 380,80" fill="none" stroke="black"/>"#,
        r#"<path d="M240,100c50,0,0,125,50,100s0,-125,50,-150s175,50,50,100s-175,50,-300,0s0,-125,50,-100s0,125,50,150s0,-100,50,-100" fill="none" stroke="black"/>"#,
        // ParserTest: relative lines, and a quadratic curve with its shorthand.
        r#"<path d="M10 10 h 80 v 80 h -80 Z" fill="none" stroke="black"/>"#,
        r#"<path d=" M 10 80 Q 52.5 10, 95 80 T 180 80" fill="none" stroke="black"/>"#,
    ] {
        assert_traces(body, 6.0);
    }
}

// Verifies: G-12, ADR-0019
#[test]
fn a_style_on_a_group_reaches_its_shapes_and_one_on_a_shape_overrides_it() {
    let body = r#"<g data-rough-fill-style="solid">
        <rect x="10" y="10" width="50" height="50" fill="red"/>
        <rect x="100" y="10" width="50" height="50" fill="red" data-rough-fill-style="hachure"/>
    </g>"#;
    let paths = paths(&sketch(body));
    assert_eq!(attribute(&paths[0], "fill"), Some("#ff0000"));
    assert_eq!(attribute(&paths[1], "fill"), Some("none"));
    assert_eq!(attribute(&paths[1], "stroke"), Some("#ff0000"));
}

// Verifies: G-12, ADR-0019
#[test]
fn the_same_text_gives_the_same_sketch_and_another_seed_another() {
    let body = ROUGH_TEST.concat();
    assert_eq!(sketch(&body), sketch(&body));
    let seeded = format!(r#"<g data-rough-seed="7">{body}</g>"#);
    assert_ne!(sketch(&body), sketch(&seeded));
}

// Verifies: G-12, ADR-0019
#[test]
fn text_that_is_no_svg_and_an_unreadable_attribute_are_errors() {
    let engine = engine();
    assert!(engine.render("rough", "rule ::= 'a' ;").is_err());
    assert!(engine.render("rough", "<svg").is_err());

    let error = engine
        .render(
            "rough",
            &svg(r#"<rect width="9" height="9" data-rough-roughness="much"/>"#),
        )
        .unwrap_err()
        .to_string();
    assert!(error.contains("data-rough-roughness"), "{error}");

    let error = engine
        .render("rough", &svg(r#"<g data-rough-colour="red"/>"#))
        .unwrap_err()
        .to_string();
    assert!(error.contains("data-rough-colour"), "{error}");
}

// Verifies: G-12, ADR-0019
#[test]
fn the_engine_supports_rough_and_returns_a_graphic() {
    let engine = engine();
    assert!(engine.supports("rough"));
    assert!(!engine.supports("railroad"));
    let text = svg(ROUGH_TEST[0]);
    match engine.render("rough", &text).unwrap() {
        EngineOutput::Graphic(graphic) => {
            assert!(graphic.svg.starts_with("<svg"));
            assert_eq!(graphic.code, text);
            assert_eq!(graphic.language, "rough");
        }
        other => panic!("{other:?}"),
    }
}

// Verifies: ADR-0020
#[test]
fn text_becomes_outlines_and_an_image_is_dropped_without_reading_it() {
    let body = r#"<text x="10" y="40" font-family="NoSuchFont" font-size="24" fill="black">Nectar</text>
        <image href="/etc/hostname" x="0" y="0" width="10" height="10"/>
        <image href="data:image/png;base64,iVBORw0KGgo=" x="0" y="0" width="10" height="10"/>"#;
    let sketch = sketch(body);
    assert!(!sketch.contains("<text"), "{sketch}");
    assert!(!sketch.contains("<image"), "{sketch}");
    let paths = paths(&sketch);
    assert!(!paths.is_empty(), "the text has outlines: {sketch}");
    assert!(
        paths
            .iter()
            .all(|p| attribute(p, "fill") == Some("#000000")),
        "{sketch}"
    );
    assert_eq!(sketch, self::sketch(body));
}

// Verifies: ADR-0020
#[test]
fn text_and_shapes_are_scaled_alike_by_the_view_box() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="400" viewBox="0 0 100 100">
        <text x="10" y="50" font-size="10" fill="black">Nectar</text></svg>"#;
    let (left, top, _, bottom) = bounds(&engine().sketch(svg).unwrap());
    // The text stands at x = 10, its baseline at y = 50, in units four times as large.
    assert!((left - 40.0).abs() < 4.0, "left {left}");
    assert!(
        bottom <= 204.0 && top >= 150.0,
        "top {top}, bottom {bottom}"
    );
}

// Verifies: G-12, ADR-0019
#[test]
fn a_shape_drawn_through_use_keeps_its_style_and_takes_the_style_of_the_use() {
    // The style on the referenced shape.
    let referenced = paths(&sketch(
        r##"<defs><rect id="r" width="50" height="50" fill="red" data-rough-fill-style="solid"/></defs>
            <use href="#r" x="10" y="10"/>"##,
    ));
    assert_eq!(attribute(&referenced[0], "fill"), Some("#ff0000"));

    // The style on the `<use>`, and on a group around it.
    for body in [
        r##"<defs><rect id="r" width="50" height="50" fill="red"/></defs>
            <use href="#r" data-rough-fill-style="solid"/>"##,
        r##"<defs><rect id="r" width="50" height="50" fill="red"/></defs>
            <g data-rough-fill-style="solid"><use href="#r"/></g>"##,
    ] {
        assert_eq!(
            attribute(&paths_of(body)[0], "fill"),
            Some("#ff0000"),
            "{body}"
        );
    }

    // Each shape of a referenced group in its own style, the shape's over the `<use>`'s.
    let pair = paths(&sketch(
        r##"<defs><g id="pair">
              <rect width="40" height="40" fill="red"/>
              <rect x="50" width="40" height="40" fill="blue" data-rough-fill-style="hachure"/>
            </g></defs>
            <use href="#pair" data-rough-fill-style="solid"/>"##,
    ));
    assert_eq!(attribute(&pair[0], "fill"), Some("#ff0000"));
    assert_eq!(attribute(&pair[1], "stroke"), Some("#0000ff"));
}

fn paths_of(body: &str) -> Vec<Vec<(String, String)>> {
    paths(&sketch(body))
}

// Verifies: G-12, ADR-0019
#[test]
fn duplicate_ids_and_ids_like_the_engines_own_keep_each_shape_its_style() {
    let paths = paths_of(
        r#"<rect id="a" width="40" height="40" fill="red" data-rough-fill-style="solid"/>
           <rect id="a" x="50" width="40" height="40" fill="red" data-rough-fill-style="hachure"/>"#,
    );
    assert_eq!(attribute(&paths[0], "fill"), Some("#ff0000"));
    assert_eq!(attribute(&paths[1], "fill"), Some("none"));

    let paths = paths_of(
        r#"<rect width="40" height="40" fill="red"/>
           <rect id="nectar-rough-1" x="50" width="40" height="40" fill="red" data-rough-fill-style="solid"/>"#,
    );
    assert_eq!(attribute(&paths[0], "fill"), Some("none"));
    assert_eq!(attribute(&paths[1], "fill"), Some("#ff0000"));
}

// Verifies: G-12, ADR-0019
#[test]
fn an_svg_with_entities_from_its_dtd_is_sketched() {
    let svg = r#"<!DOCTYPE svg [<!ENTITY r "<rect width='40' height='40' fill='red'/>">]>
        <svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">&r;</svg>"#;
    let paths = paths(&engine().sketch(svg).unwrap());
    assert_eq!(attribute(&paths[0], "stroke"), Some("#ff0000"));
}

// Verifies: G-12, ADR-0019
#[test]
fn every_shape_that_encloses_an_area_is_filled_and_a_line_is_not() {
    for body in [
        // A bowtie, whose two halves wind in opposite directions.
        r#"<path d="M0 0 L100 100 L100 0 L0 100 Z" fill="red"/>"#,
        // A square of two triangles of opposite winding.
        r#"<path d="M0 0 L100 0 L0 100 Z M100 0 L100 100 L0 100 Z" fill="red"/>"#,
    ] {
        assert_eq!(paths_of(body).len(), 1, "{body}");
    }
    let small = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 1 1">
        <rect x="0.1" y="0.1" width="0.07" height="0.07" fill="red"/></svg>"#;
    assert_eq!(paths(&engine().sketch(small).unwrap()).len(), 1);
    assert!(paths_of(r#"<line x1="0" y1="0" x2="100" y2="50" fill="red"/>"#).is_empty());
}

// Verifies: G-12, ADR-0019
#[test]
fn the_opacity_of_a_group_fades_its_shapes() {
    let paths = paths_of(
        r#"<g opacity="0.5"><rect width="40" height="40" fill="red" fill-opacity="0.5" stroke="black"/></g>"#,
    );
    assert_eq!(attribute(&paths[0], "stroke-opacity"), Some("0.25"));
    assert_eq!(attribute(&paths[1], "stroke-opacity"), Some("0.5"));
}

// Verifies: G-12, ADR-0019
#[test]
fn a_shape_inside_a_marker_is_drawn_in_the_default_style() {
    let paths = paths_of(
        r##"<defs><marker id="m" markerWidth="10" markerHeight="10">
              <rect width="10" height="10" fill="red" data-rough-fill-style="solid"/></marker></defs>
            <line x1="20" y1="20" x2="200" y2="20" stroke="black" marker-end="url(#m)"/>"##,
    );
    assert!(
        paths
            .iter()
            .any(|p| attribute(p, "stroke") == Some("#ff0000")),
        "the marker's rect is sketched with a hachure, not solid"
    );
}

// Verifies: G-12, ADR-0019, ADR-0020
#[test]
fn every_shape_drawn_through_use_keeps_its_own_style_beside_markers_and_unpainted_shapes() {
    // A marker adds paths to what a `<use>` draws; the marked shape keeps its own style.
    let marked = paths_of(
        r##"<defs>
              <marker id="m" markerWidth="10" markerHeight="10"><rect width="10" height="10" fill="blue"/></marker>
              <g id="g"><path d="M10 10 L200 10" stroke="black" marker-end="url(#m)" data-rough-disable-multi-stroke="true"/></g>
            </defs>
            <use href="#g" data-rough-fill-style="solid"/>"##,
    );
    let line = marked
        .iter()
        .find(|p| attribute(p, "stroke") == Some("#000000"))
        .unwrap();
    assert_eq!(
        attribute(line, "d").unwrap().matches('M').count(),
        2,
        "one stroke, its own style: {marked:?}"
    );
    // The marker's rect takes the defaults, not the `<use>`'s solid.
    assert!(
        marked
            .iter()
            .any(|p| attribute(p, "stroke") == Some("#0000ff"))
    );

    // An unpainted hit area and a `<switch>` before a styled shape.
    for body in [
        r##"<defs><g id="g"><rect width="90" height="90" fill="none"/>
              <rect x="10" y="10" width="40" height="40" fill="red" data-rough-fill-style="solid"/></g></defs>
            <use href="#g"/>"##,
        r##"<defs><g id="g"><switch><rect width="20" height="20" fill="blue"/></switch>
              <rect x="30" y="10" width="40" height="40" fill="red" data-rough-fill-style="solid"/></g></defs>
            <use href="#g"/>"##,
    ] {
        let paths = paths_of(body);
        assert!(
            paths
                .iter()
                .any(|p| attribute(p, "fill") == Some("#ff0000")),
            "{body}: {paths:?}"
        );
    }
}

// Verifies: G-12, ADR-0020
#[test]
fn a_use_is_placed_by_its_position_and_a_symbol_by_its_view_box() {
    let placed = r##"<defs><rect id="r" width="40" height="40" fill="none" stroke="black"/></defs>
        <use href="#r" x="100" y="50" transform="translate(10 0)"/>"##;
    let (left, top, _, _) = bounds(&sketch(placed));
    assert!(
        (left - 110.0).abs() < 4.0 && (top - 50.0).abs() < 4.0,
        "{left} {top}"
    );

    let symbol = r##"<defs><symbol id="s" viewBox="0 0 10 10">
          <rect width="10" height="10" fill="red" data-rough-fill-style="solid"/></symbol></defs>
        <use href="#s" x="20" y="20" width="100" height="100"/>"##;
    let sketch = sketch(symbol);
    let (left, top, right, bottom) = bounds(&sketch);
    assert!(
        (left - 20.0).abs() < 4.0 && (right - 120.0).abs() < 4.0,
        "{left} {right}"
    );
    assert!(
        (top - 20.0).abs() < 4.0 && (bottom - 120.0).abs() < 4.0,
        "{top} {bottom}"
    );
    assert_eq!(attribute(&paths(&sketch)[0], "fill"), Some("#ff0000"));
}

// Verifies: ADR-0019
#[test]
fn a_use_that_references_itself_draws_nothing() {
    let body = r##"<g id="loop"><rect width="20" height="20" fill="none" stroke="black"/><use href="#loop" x="30"/></g>"##;
    assert_eq!(
        paths_of(body).len(),
        1,
        "the rect, and nothing for the <use>"
    );
}

// Verifies: G-12, ADR-0019
#[test]
fn a_symbol_lends_its_paint_and_style_to_its_shapes() {
    let paths = paths_of(
        r##"<defs><symbol id="s" fill="lime" data-rough-fill-style="solid"><rect width="40" height="40"/></symbol></defs>
            <use href="#s" width="40" height="40"/>"##,
    );
    assert_eq!(attribute(&paths[0], "fill"), Some("#00ff00"), "{paths:?}");
}

// Verifies: G-12, ADR-0020
#[test]
fn a_use_inside_a_clip_path_leaves_the_clipped_shape_drawn() {
    let paths = paths_of(
        r##"<defs><rect id="cr" width="50" height="50"/><clipPath id="c"><use href="#cr"/></clipPath></defs>
            <rect width="100" height="100" fill="red" clip-path="url(#c)"/>"##,
    );
    assert_eq!(paths.len(), 1, "{paths:?}");
}

// Verifies: ADR-0019
#[test]
fn a_symbol_with_a_shape_from_an_entity_is_an_svg_and_no_crash() {
    let svg = r##"<!DOCTYPE svg [<!ENTITY sh "<rect x='20' width='10' height='10' fill='red'/>">]>
        <svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <defs><symbol id="s"><circle cx="5" cy="5" r="5" fill="blue"/>&sh;</symbol></defs>
          <use href="#s"/></svg>"##;
    assert!(!paths(&engine().sketch(svg).unwrap()).is_empty());
}

// Verifies: ADR-0019
#[test]
fn uses_that_reference_each_other_or_grow_without_end_stop() {
    let cycle = paths_of(
        r##"<g id="a"><rect width="20" height="20" fill="none" stroke="black"/><use href="#b" x="30"/></g>
            <g id="b"><rect y="30" width="20" height="20" fill="none" stroke="black"/><use href="#a" y="30"/></g>"##,
    );
    assert!(
        cycle.len() == 2,
        "the two rects, and nothing for the circular uses: {} paths",
        cycle.len()
    );

    // Six levels of eight `<use>` each in undrawn definitions, and the same drawn.
    let mut defs = String::from(r#"<rect id="g0" width="1" height="1" fill="red"/>"#);
    for level in 1..=6 {
        defs.push_str(&format!("<g id=\"g{level}\">"));
        for i in 0..8 {
            defs.push_str(&format!("<use href=\"#g{}\" x=\"{i}\"/>", level - 1));
        }
        defs.push_str("</g>");
    }
    let started = std::time::Instant::now();
    assert!(paths_of(&format!("<defs>{defs}</defs>")).is_empty());
    assert!(started.elapsed().as_secs() < 10, "{:?}", started.elapsed());
    let small = format!(r##"<defs>{defs}</defs><use href="#g2"/>"##);
    assert_eq!(paths_of(&small).len(), 64);
    // Drawn, five levels would be 32 768 shapes: the engine stops within its budget and
    // usvg within its node limit, with a sketch or an error, but soon.
    let started = std::time::Instant::now();
    let _ = engine().sketch(&svg(&format!(r##"<defs>{defs}</defs><use href="#g5"/>"##)));
    assert!(started.elapsed().as_secs() < 30, "{:?}", started.elapsed());
}

// Verifies: G-12, ADR-0020
#[test]
fn a_use_in_a_prefixed_svg_namespace_is_resolved_in_it() {
    let svg = r##"<s:svg xmlns:s="http://www.w3.org/2000/svg" width="100" height="100">
        <s:defs><s:rect id="r" width="40" height="40" fill="red" data-rough-fill-style="solid"/></s:defs>
        <s:use href="#r"/></s:svg>"##;
    let paths = paths(&engine().sketch(svg).unwrap());
    assert_eq!(attribute(&paths[0], "fill"), Some("#ff0000"));
}

// Verifies: ADR-0019
#[test]
fn a_use_that_sizes_an_svg_is_left_to_usvg_which_scales_it() {
    let body = r##"<defs><svg id="v" width="10" height="10" viewBox="0 0 10 10">
          <rect width="10" height="10" fill="red" data-rough-fill-style="solid"/></svg></defs>
        <use href="#v" width="100" height="100"/>"##;
    let sketch = sketch(body);
    let (left, _, right, _) = bounds(&sketch);
    assert!((right - left - 100.0).abs() < 6.0, "{left} {right}");
    // Unresolved, its shape takes the defaults: a hachure, not solid.
    assert_eq!(attribute(&paths(&sketch)[0], "stroke"), Some("#ff0000"));
}

// Verifies: ADR-0020
#[test]
fn a_copy_that_does_not_parse_where_it_lands_spoils_no_other_use() {
    // The copy of `g` would carry `n:a` outside the element that declares `n`.
    let body = r##"<defs xmlns:n="urn:n">
          <g id="g" n:a="1"><rect width="20" height="20" fill="blue"/></g>
          <rect id="r" x="50" width="20" height="20" fill="red" data-rough-fill-style="solid"/>
        </defs>
        <use href="#g"/><use href="#r"/>"##;
    let paths = paths_of(body);
    assert!(
        paths
            .iter()
            .any(|p| attribute(p, "fill") == Some("#ff0000")),
        "{paths:?}"
    );
}

// Verifies: G-12, ADR-0019
#[test]
fn a_shape_from_an_entity_takes_no_style_from_a_shape_that_shares_its_id() {
    let svg = r##"<!DOCTYPE svg [<!ENTITY b "<rect id='a' x='50' width='40' height='40' fill='blue'/>">]>
        <svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <rect id="a" width="40" height="40" fill="red" data-rough-fill-style="solid"/>&b;</svg>"##;
    let paths = paths(&engine().sketch(svg).unwrap());
    assert!(
        paths
            .iter()
            .any(|p| attribute(p, "fill") == Some("#ff0000")),
        "{paths:?}"
    );
    assert!(
        paths
            .iter()
            .any(|p| attribute(p, "stroke") == Some("#0000ff")),
        "{paths:?}"
    );
    assert!(
        !paths
            .iter()
            .any(|p| attribute(p, "fill") == Some("#0000ff")),
        "{paths:?}"
    );
}
