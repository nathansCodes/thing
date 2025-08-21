use iced::{
    Element, Event, Length, Rectangle, Size,
    advanced::{
        Clipboard, Layout, Shell, Widget,
        graphics::core::event::Status,
        layout::{Limits, Node},
        mouse::Cursor,
        widget::{
            Tree,
            tree::{self, Tag},
        },
    },
    mouse,
};

pub(super) struct DragAndDropProvider<'a, Payload, Message, Theme, Renderer>
where
    Payload: Clone + 'a,
    Message: Clone + 'a,
{
    pub(super) set_payload: Box<dyn Fn(Option<Payload>) -> Message + 'a>,
    pub(super) content: Element<'a, Message, Theme, Renderer>,
    pub(super) payload: Payload,
}

#[derive(Default)]
struct State {
    lmb_pressed: bool,
    is_hovered: bool,
    is_dragging: bool,
}

impl<'a, Payload, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for DragAndDropProvider<'a, Payload, Message, Theme, Renderer>
where
    Payload: Clone + 'a,
    Message: Clone + 'a,
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer,
{
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
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

    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::Some(Box::new(State::default()))
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
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
        let state = tree.state.downcast_mut::<State>();

        let mut status = Status::Ignored;

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
            status = Status::Captured;
        }

        if let Event::Mouse(ev) = event {
            match ev {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    state.lmb_pressed = state.is_hovered;

                    status = Status::Captured;
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    shell.publish((self.set_payload)(None));

                    state.lmb_pressed = false;
                    state.is_dragging = false;

                    status = Status::Captured;
                }
                mouse::Event::CursorMoved { .. } => {
                    let was_hovered = state.is_hovered;
                    state.is_hovered = layout
                        .bounds()
                        .intersection(viewport)
                        .is_some_and(|bounds| cursor.position_over(bounds).is_some());

                    if was_hovered && !state.is_hovered {
                        state.is_dragging = state.lmb_pressed;
                    }

                    if state.is_dragging {
                        shell.publish((self.set_payload)(Some(self.payload.clone())));
                    }

                    status = Status::Captured;
                }
                _ => (),
            }
        }

        status
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

impl<'a, Payload, Message, Theme, Renderer>
    From<DragAndDropProvider<'a, Payload, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Payload: Clone + 'a,
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer + 'a,
{
    fn from(value: DragAndDropProvider<'a, Payload, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}
