use ambient_api::{
    core::{
        layout::components::{
            //LEGACY_MISSING_ENUM_SUPPORT: align_horizontal_center, align_horizontal_end, align_vertical_center,
            //LEGACY_MISSING_ENUM_SUPPORT: align_vertical_end, fit_horizontal_children, fit_horizontal_none,
            //LEGACY_MISSING_ENUM_SUPPORT: fit_vertical_children, fit_vertical_none,
            height,
            space_between_items,
            width,
        },
        text::components::font_size,
    },
    prelude::*,
};

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    let background = |e| FlowRow::el([e]).with_background(vec4(1., 1., 1., 0.02));
    FlowColumn::el([
        FlowRow::el([Text::el("Basic")])
            .with_background(vec4(0.1, 0.1, 0.1, 1.))
            //LEGACY_MISSING_ENUM_SUPPORT: .with_default(fit_vertical_children())
            //LEGACY_MISSING_ENUM_SUPPORT: .with_default(fit_horizontal_children())
            .with_padding_even(10.),
        FlowRow::el([Text::el("Spacing"), Text::el("between"), Text::el("items")])
            .with_background(vec4(0.1, 0.1, 0.1, 1.))
            //LEGACY_MISSING_ENUM_SUPPORT: .with_default(fit_vertical_children())
            //LEGACY_MISSING_ENUM_SUPPORT: .with_default(fit_horizontal_children())
            .with_padding_even(10.)
            .with(space_between_items(), 50.),
        FlowRow::el([Text::el("Break"), Text::el("line")])
            .with_background(vec4(0.1, 0.1, 0.1, 1.))
            //LEGACY_MISSING_ENUM_SUPPORT: .with_default(fit_vertical_children())
            //LEGACY_MISSING_ENUM_SUPPORT: .with_default(fit_horizontal_none())
            .with(width(), 50.)
            .with_padding_even(10.),
        FlowRow::el([
            background(Text::el("Align")),
            background(Text::el("Center").with(font_size(), 30.)),
        ])
        .with_background(vec4(0.1, 0.1, 0.1, 1.))
        //LEGACY_MISSING_ENUM_SUPPORT: .with_default(fit_vertical_none())
        //LEGACY_MISSING_ENUM_SUPPORT: .with_default(fit_horizontal_none())
        //LEGACY_MISSING_ENUM_SUPPORT: .with_default(align_horizontal_center())
        //LEGACY_MISSING_ENUM_SUPPORT: .with_default(align_vertical_center())
        .with(width(), 200.)
        .with(height(), 70.)
        .with_padding_even(10.)
        .with(space_between_items(), 5.),
        FlowRow::el([
            background(Text::el("Align")),
            background(Text::el("End").with(font_size(), 30.)),
        ])
        .with_background(vec4(0.1, 0.1, 0.1, 1.))
        //LEGACY_MISSING_ENUM_SUPPORT: .with_default(fit_vertical_none())
        //LEGACY_MISSING_ENUM_SUPPORT: .with_default(fit_horizontal_none())
        //LEGACY_MISSING_ENUM_SUPPORT: .with_default(align_horizontal_end())
        //LEGACY_MISSING_ENUM_SUPPORT: .with_default(align_vertical_end())
        .with(width(), 200.)
        .with(height(), 70.)
        .with_padding_even(10.)
        .with(space_between_items(), 5.),
    ])
    .with(space_between_items(), 5.)
    .with_padding_even(STREET)
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
