use iced::{
    Element, Event, Length, Rectangle, Size,
    advanced::{
        Clipboard, Layout, Shell, Widget,
        graphics::core::event::Status,
        layout::{Limits, Node},
        mouse::Cursor,
        widget::{
            Tree,
            tree::{State, Tag},
        },
    },
    mouse,
};

pub(super) struct DragAndDropProvider<'a, Message: Clone, Theme, Renderer> {
    pub(super) start_dragging: Message,
    pub(super) content: Element<'a, Message, Theme, Renderer>,
}

#[derive(Default)]
struct DragAndDropProviderState {
    lmb_pressed: bool,
    is_hovered: bool,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for DragAndDropProvider<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer,
{
    fn state(&self) -> State {
        State::Some(Box::new(DragAndDropProviderState::default()))
    }

    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        Tag::of::<DragAndDropProviderState>()
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let content_layout =
            self.content
                .as_widget()
                .layout(&mut tree.children[0], renderer, limits);

        let size = limits.resolve(
            content_layout.size().width,
            content_layout.size().height,
            content_layout.size(),
        );

        Node::with_children(size, vec![content_layout])
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            &layout.bounds(),
        );
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> Status {
        if let Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) {
            return Status::Captured;
        }

        let state = tree.state.downcast_mut::<DragAndDropProviderState>();

        if let Event::Mouse(ev) = event {
            match ev {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    state.lmb_pressed = true;
                    Status::Captured
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    state.lmb_pressed = false;
                    Status::Captured
                }
                mouse::Event::CursorMoved { position } => {
                    let was_hovered = state.is_hovered;
                    state.is_hovered = layout
                        .children()
                        .next()
                        .unwrap()
                        .bounds()
                        .contains(position);

                    if was_hovered && !state.is_hovered && state.lmb_pressed {
                        shell.publish(self.start_dragging.clone());
                    }

                    Status::Captured
                }
                _ => Status::Ignored,
            }
        } else {
            Status::Ignored
        }
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.content]);
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content.as_widget().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<DragAndDropProvider<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer + 'a,
{
    fn from(value: DragAndDropProvider<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}
