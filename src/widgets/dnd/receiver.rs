use iced::{
    Element, Event, Length, Point, Rectangle, Size, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget,
        graphics::core::event::Status,
        layout::{Limits, Node},
        mouse::{Cursor, Interaction},
        overlay,
        widget::Tree,
    },
    mouse,
};

pub(super) struct DragAndDropReceiver<'a, Payload, Message, Theme, Renderer>
where
    Payload: Clone + 'a,
    Message: Clone + std::fmt::Debug + 'a,
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer + 'a,
    Theme: 'a,
{
    pub(super) receive: Box<dyn Fn(Payload, Point) -> Option<Message> + 'a>,
    pub(super) payload: Option<Payload>,
    pub(super) content: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Payload, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for DragAndDropReceiver<'a, Payload, Message, Theme, Renderer>
where
    Payload: Clone + 'a,
    Message: Clone + std::fmt::Debug + 'a,
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer + 'a,
    Theme: 'a,
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
        _viewport: &Rectangle,
    ) -> Status {
        let Some(payload) = self.payload.clone() else {
            return self.content.as_widget_mut().on_event(
                &mut tree.children[0],
                event,
                layout.children().next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                &layout.children().next().unwrap().bounds(),
            );
        };

        match event {
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(relative_cursor_pos) = cursor.position_in(layout.bounds())
                    && let Some(message) = (self.receive)(payload, relative_cursor_pos)
                {
                    shell.publish(message);
                    Status::Captured
                } else {
                    Status::Ignored
                }
            }
            _ => Status::Ignored,
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        if let Some(relative_cursor_pos) = cursor.position_in(layout.bounds())
            && let Some(payload) = self.payload.clone()
            && (self.receive)(payload, relative_cursor_pos).is_none()
        {
            Interaction::NotAllowed
        } else {
            Interaction::None
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }
}

impl<'a, Payload, Message, Theme, Renderer>
    From<DragAndDropReceiver<'a, Payload, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Payload: Clone + 'a,
    Message: Clone + std::fmt::Debug + 'a,
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer + 'a,
    Theme: 'a,
{
    fn from(value: DragAndDropReceiver<'a, Payload, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}
