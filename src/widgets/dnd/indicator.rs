use iced::{
    Element, Event, Length, Point, Rectangle, Size,
    advanced::{
        Clipboard, Layout, Shell, Widget,
        graphics::core::event::Status,
        layout::{Limits, Node},
        mouse::{Cursor, Interaction},
        renderer::Style,
        widget::{
            Tree,
            tree::{State, Tag},
        },
    },
    mouse,
};

pub(super) struct DragAndDropIndicator<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer,
{
    pub(super) payload_element: Option<Element<'a, Message, Theme, Renderer>>,
    pub(super) content: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for DragAndDropIndicator<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer,
{
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let mut children =
            vec![
                self.content
                    .as_widget()
                    .layout(&mut tree.children[0], renderer, limits),
            ];

        if let Some(payload_element) = &self.payload_element {
            children.push(
                payload_element
                    .as_widget()
                    .layout(&mut tree.children[0], renderer, limits)
                    .move_to(*tree.state.downcast_ref::<Point>()),
            );
        }

        Node::with_children(limits.max(), children)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            tree.children.first().unwrap(),
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );

        if let Some(payload_element) = &self.payload_element {
            renderer.with_layer(*viewport, |renderer| {
                payload_element.as_widget().draw(
                    &tree.children[1],
                    renderer,
                    theme,
                    style,
                    layout.children().nth(1).unwrap(),
                    cursor,
                    viewport,
                );
            });
        }
    }

    fn tag(&self) -> Tag {
        Tag::of::<Point>()
    }

    fn state(&self) -> State {
        State::Some(Box::new(Point::ORIGIN))
    }

    fn children(&self) -> Vec<Tree> {
        let mut children = vec![Tree::new(&self.content)];

        if let Some(payload_element) = &self.payload_element {
            children.push(Tree::new(payload_element));
        }

        children
    }

    fn diff(&self, tree: &mut Tree) {
        let mut children = vec![self.content.as_widget()];

        if let Some(payload_element) = &self.payload_element {
            children.push(payload_element.as_widget());
        }

        tree.diff_children(&children);
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

        let state = tree.state.downcast_mut::<Point>();

        if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
            *state = position;
            status = Status::Captured;
        }

        status
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> Interaction {
        if self.payload_element.is_some() {
            Interaction::Grabbing
        } else {
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout.children().next().unwrap(),
                cursor,
                viewport,
                renderer,
            )
        }
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

impl<'a, Message, Theme, Renderer> From<DragAndDropIndicator<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer + 'a,
{
    fn from(value: DragAndDropIndicator<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}
