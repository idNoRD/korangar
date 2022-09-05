use crate::graphics::{InterfaceRenderer, Renderer};
use crate::interface::*;

#[derive(Default)]
pub struct CloseButton {
    state: ElementState,
}

impl Element for CloseButton {

    fn get_state(&self) -> &ElementState {
        &self.state
    }

    fn get_state_mut(&mut self) -> &mut ElementState {
        &mut self.state
    }

    fn resolve(&mut self, placement_resolver: &mut PlacementResolver, _interface_settings: &InterfaceSettings, theme: &Theme) {

        let (size, position) = placement_resolver.allocate_right(&theme.close_button.size_constraint);
        self.state.cached_size = size.finalize();
        self.state.cached_position = position;
    }

    fn hovered_element(&self, mouse_position: Position) -> HoverInformation {
        self.state.hovered_element(mouse_position)
    }

    fn left_click(&mut self, _force_update: &mut bool) -> Option<ClickAction> {
        Some(ClickAction::CloseWindow)
    }

    fn render(
        &self,
        render_target: &mut <InterfaceRenderer as Renderer>::Target,
        renderer: &InterfaceRenderer,
        _state_provider: &StateProvider,
        interface_settings: &InterfaceSettings,
        theme: &Theme,
        parent_position: Position,
        clip_size: ClipSize,
        hovered_element: Option<&dyn Element>,
        _focused_element: Option<&dyn Element>,
        _second_theme: bool,
    ) {

        let mut renderer = self
            .state
            .element_renderer(render_target, renderer, interface_settings, parent_position, clip_size);

        let background_color = match self.is_element_self(hovered_element) {
            true => *theme.close_button.hovered_background_color,
            false => *theme.close_button.background_color,
        };

        renderer.render_background(*theme.close_button.border_radius, background_color);

        renderer.render_text(
            "X",
            *theme.close_button.text_offset,
            *theme.close_button.foreground_color,
            *theme.close_button.font_size,
        );
    }
}
