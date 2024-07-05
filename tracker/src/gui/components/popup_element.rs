use iced::{
    advanced::{widget::Tree, Widget},
    Element, Point, Rectangle,
};

pub struct PopupElement<'a, Message, Theme, Renderer> {
    base: Element<'a, Message, Theme, Renderer>,
    popup: Element<'a, Message, Theme, Renderer>,
    on_hovered: Option<Message>,
    on_idle: Option<Message>,
    ideal_cursor_margin: f32,
    min_cursor_margin: f32,
}

impl<'a, Message, Theme, Renderer> PopupElement<'a, Message, Theme, Renderer> {
    const DEFAULT_MIN_CURSOR_MARGIN: f32 = 1.0;
    const DEFAULT_IDEAL_CURSOR_MARGIN: f32 = 10.0;
    pub fn new(
        base: impl Into<Element<'a, Message, Theme, Renderer>>,
        popup: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        PopupElement {
            base: base.into(),
            popup: popup.into(),
            on_hovered: None,
            on_idle: None,
            ideal_cursor_margin: Self::DEFAULT_IDEAL_CURSOR_MARGIN,
            min_cursor_margin: Self::DEFAULT_MIN_CURSOR_MARGIN,
        }
    }

    pub fn on_hovered(self, on_hovered: Message) -> Self {
        Self {
            on_hovered: Some(on_hovered),
            ..self
        }
    }

    pub fn on_idle(self, on_idle: Message) -> Self {
        Self {
            on_idle: Some(on_idle),
            ..self
        }
    }

    pub fn ideal_cursor_margin(self, ideal_cursor_margin: f32) -> Self {
        Self {
            ideal_cursor_margin,
            ..self
        }
    }

    pub fn min_cursor_margin(self, min_cursor_margin: f32) -> Self {
        Self {
            min_cursor_margin,
            ..self
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for PopupElement<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Message: Clone,
{
    fn size(&self) -> iced::Size<iced::Length> {
        self.base.as_widget().size()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.base
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.base.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn size_hint(&self) -> iced::Size<iced::Length> {
        self.size()
    }

    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.base), Tree::new(&self.popup)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.base, &self.popup])
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation<Message>,
    ) {
        self.base
            .as_widget()
            .operate(&mut tree.children[0], layout, renderer, operation)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> iced_futures::core::event::Status {
        let state = tree.state.downcast_mut::<State>();

        let new_state = cursor
            .position_over(layout.bounds())
            .map(|cursor_position| State::Hovered { cursor_position })
            .unwrap_or_default();

        match state {
            State::Idle => {
                if new_state != State::Idle {
                    if let Some(hovered_message) = self.on_hovered.as_ref() {
                        shell.publish(hovered_message.clone());
                    }
                }
            }
            State::Hovered { cursor_position: _ } => {
                if new_state == State::Idle {
                    if let Some(idle_message) = self.on_idle.as_ref() {
                        shell.publish(idle_message.clone());
                    }
                }
            }
        }

        *state = new_state;

        self.base.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.base.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &Renderer,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_ref::<State>();

        let (content_state, popup_state) = {
            let (a, b) = tree.children.split_at_mut(1);
            (&mut a[0], &mut b[0])
        };

        let content_overlay =
            self.base
                .as_widget_mut()
                .overlay(content_state, layout, renderer, translation);

        let popup_overlay = if let State::Hovered { cursor_position } = *state {
            Some(iced::advanced::overlay::Element::new(Box::new(Overlay {
                position: layout.position() + translation,
                popup_content: &mut self.popup,
                tree: popup_state,
                base_bounds: layout.bounds(),
                cursor_position,
                min_cursor_margin: self.min_cursor_margin,
                ideal_cursor_margin: self.ideal_cursor_margin,
            })))
        } else {
            None
        };

        let children: Vec<iced::advanced::overlay::Element<'_, Message, Theme, Renderer>> =
            content_overlay.into_iter().chain(popup_overlay).collect();

        (!children.is_empty())
            .then(|| iced::advanced::overlay::Group::with_children(children).overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<PopupElement<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: 'a + iced::advanced::Renderer,
    Theme: 'a,
    Message: 'a + Clone,
{
    fn from(value: PopupElement<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

#[derive(Default, Debug, PartialEq)]
enum State {
    #[default]
    Idle,
    Hovered {
        cursor_position: Point,
    },
}

struct Overlay<'a, 'b, Message, Theme, Renderer> {
    popup_content: &'b mut Element<'a, Message, Theme, Renderer>,
    tree: &'b mut Tree,
    position: Point,
    cursor_position: Point,
    min_cursor_margin: f32,
    ideal_cursor_margin: f32,
    base_bounds: Rectangle,
}

impl<'a, 'b, Message, Theme, Renderer> iced::advanced::Overlay<Message, Theme, Renderer>
    for Overlay<'a, 'b, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: iced::Size) -> iced::advanced::layout::Node {
        let limits = iced::advanced::layout::Limits::new(iced::Size::ZERO, iced::Size::INFINITY);
        let content_layout =
            self.popup_content
                .as_widget()
                .layout(&mut self.tree, renderer, &limits);
        let content_bounds = {
            let content_bounds = content_layout.bounds();
            let translation = self.position - self.base_bounds.position();
            let real_cursor_position = Point::new(
                self.cursor_position.x + translation.x,
                self.cursor_position.y + translation.y,
            );

            let width_needed = content_bounds.width + self.min_cursor_margin;
            let height_needed = content_bounds.height + self.min_cursor_margin;

            let space_above = real_cursor_position.y;
            let space_below = bounds.height - real_cursor_position.y;
            let space_after = bounds.width - real_cursor_position.x;

            let y = if space_below < height_needed {
                // align above cursor
                let remaining_space = space_above - content_bounds.height;
                let offset = remaining_space.min(self.ideal_cursor_margin);
                real_cursor_position.y - (content_bounds.height + offset)
            } else {
                // align below cursor
                let remaining_space = space_below - content_bounds.height;
                let offset = remaining_space.min(self.ideal_cursor_margin);
                real_cursor_position.y + offset
            };

            let x = if space_after < width_needed {
                // align as right as we can
                let remaining_space = bounds.width - real_cursor_position.x;
                let offset = remaining_space - content_bounds.width;
                real_cursor_position.x + offset
            } else {
                let remaining_space = space_after - content_bounds.width;
                let offset = remaining_space.min(self.ideal_cursor_margin);
                real_cursor_position.x + offset
            };

            iced::Rectangle {
                x,
                y,
                width: content_bounds.width,
                height: content_bounds.height,
            }
        };

        iced::advanced::layout::Node::with_children(content_bounds.size(), vec![content_layout])
            .translate(iced::Vector::new(content_bounds.x, content_bounds.y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
    ) {
        self.popup_content.as_widget().draw(
            &self.tree,
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            &layout.bounds(),
        )
    }

    fn operate(
        &mut self,
        layout: iced::advanced::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation<Message>,
    ) {
        self.popup_content.as_widget_mut().operate(
            &mut self.tree,
            layout.children().next().unwrap(),
            renderer,
            operation,
        )
    }

    fn on_event(
        &mut self,
        event: iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
    ) -> iced_futures::core::event::Status {
        self.popup_content.as_widget_mut().on_event(
            &mut self.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        )
    }

    fn mouse_interaction(
        &self,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.popup_content.as_widget().mouse_interaction(
            &self.tree,
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn is_over(
        &self,
        _layout: iced::advanced::Layout<'_>,
        _renderer: &Renderer,
        _cursor_position: Point,
    ) -> bool {
        false
    }
}
