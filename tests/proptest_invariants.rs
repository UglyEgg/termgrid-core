use proptest::prelude::*;
use termgrid_core::{Frame, GlyphRegistry, RenderOp, RenderProfile, Renderer, Style};

fn profile() -> RenderProfile {
    RenderProfile::bbsstalgia_xtermjs_unicode11_example()
}

fn op_strategy(max_w: u16, max_h: u16) -> impl Strategy<Value = RenderOp> {
    let x = 0u16..max_w;
    let y = 0u16..max_h;
    let w = 0u16..=max_w;
    let h = 0u16..=max_h;

    prop_oneof![
        Just(RenderOp::Clear),
        y.clone().prop_map(|yy| RenderOp::ClearLine { y: yy }),
        (x.clone(), y.clone()).prop_map(|(xx, yy)| RenderOp::ClearEol { x: xx, y: yy }),
        (x.clone(), y.clone()).prop_map(|(xx, yy)| RenderOp::ClearBol { x: xx, y: yy }),
        (x.clone(), y.clone()).prop_map(|(xx, yy)| RenderOp::ClearEos { x: xx, y: yy }),
        (x.clone(), y.clone(), w.clone(), h.clone()).prop_map(|(xx, yy, ww, hh)| {
            RenderOp::ClearRect {
                x: xx,
                y: yy,
                w: ww,
                h: hh,
            }
        }),
        (x.clone(), y.clone()).prop_map(|(xx, yy)| RenderOp::PutGlyph {
            x: xx,
            y: yy,
            glyph: "@".to_string(),
            style: Style::plain(),
        }),
        (x.clone(), y.clone(), w.clone(), h.clone()).prop_map(|(xx, yy, ww, hh)| {
            RenderOp::FillRect {
                x: xx,
                y: yy,
                w: ww,
                h: hh,
                glyph: " ".to_string(),
                style: Style::plain(),
            }
        }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, .. ProptestConfig::default() })]

    #[test]
    fn random_ops_preserve_grid_invariants(ops in prop::collection::vec(op_strategy(8, 4), 0..200)) {
        let prof = profile();
        let reg_apply = GlyphRegistry::new(prof.clone());
        let reg_check = GlyphRegistry::new(prof);

        let mut r = Renderer::new(8, 4, reg_apply).unwrap();

        let f = Frame { ops };
        let _dmg = r.apply_with_damage(&f);

        r.grid().validate_invariants(&reg_check).unwrap();
    }
}
