use iced::{
    Alignment::Center,
    Element, Font,
    Length::*,
    Task,
    widget::{
        column, container, horizontal_rule, horizontal_space, image, row, scrollable, text,
        text_input,
    },
};

use crate::{
    Draggable, State,
    assets::{Asset, AssetHandle, AssetsData, Character, Image, image::DEFAULT_IMAGE},
    style,
    widgets::{base_button, dnd::dnd_receiver},
};

#[derive(Default)]
pub struct EditPane {
    current: Option<AssetHandle>,
    temp_asset: Option<Asset>,
}

fn capitalize(a: &str) -> String {
    let (first, rest) = a.split_at(1);

    first.to_uppercase() + rest
}

impl EditPane {
    pub fn view<'a: 'b, 'b>(&'b self, state: &'a State) -> Element<'b, EditMessage> {
        if let Some(handle) = &self.current
            && let Some(_) = state.assets.get(*handle)
        {
            let path = state.assets.path(*handle).unwrap();

            let folder = capitalize(path.kind().folder());
            let name = path.name();

            let sep = text('>').size(14).style(text::secondary);

            let crumbs = row![text(folder), sep, text(name),]
                .spacing(3)
                .padding([2, 6])
                .align_y(Center);

            let Some(temp_asset) = self.temp_asset.as_ref() else {
                return "".into();
            };

            let content = match temp_asset {
                Asset::Image(img) => Element::from(
                    container(
                        column![image(&img.handle).width(350), text(path.file_name())]
                            .align_x(Center),
                    )
                    .center(Fill),
                ),
                Asset::Character(chara) => {
                    let img = state
                        .assets
                        .get_direct::<Image>(chara.img)
                        .unwrap_or(&DEFAULT_IMAGE);

                    let name_input = text_input("Name", &chara.name)
                        .size(24)
                        .font({
                            let mut f = Font::DEFAULT;
                            f.weight = iced::font::Weight::Bold;
                            f
                        })
                        .align_x(Center)
                        .style(style::text_input_inline)
                        .on_input(|input| {
                            EditMessage::UpdateTempAsset(Edit::Character(EditCharacter::Name(
                                input,
                            )))
                        });

                    let img_element = dnd_receiver(
                        |payload: &Draggable, _point| {
                            println!("{payload:?}");
                            match payload {
                                Draggable::Asset(handle) => state
                                    .assets
                                    .is::<Image>(*handle)
                                    .then_some(EditMessage::UpdateTempAsset(Edit::Character(
                                        EditCharacter::Image(*handle),
                                    ))),
                            }
                        },
                        state.dnd_payload.as_ref(),
                        image(&img.handle).width(350),
                    );

                    let editor = container(
                        scrollable(column![name_input, img_element].align_x(Center))
                            .style(style::scrollable),
                    )
                    .center(Fill);

                    column![
                        editor,
                        horizontal_rule(2),
                        row![
                            base_button("Cancel")
                                .style(style::secondary_button)
                                .on_press(EditMessage::Cancel),
                            horizontal_space(),
                            base_button("Apply").on_press(EditMessage::Apply),
                        ]
                        .padding(4)
                    ]
                    .padding(2)
                    .into()
                }
            };

            return Element::from(column![crumbs, content]);
        }

        horizontal_space().into()
    }

    pub fn update(&mut self, assets: &mut AssetsData, message: EditMessage) -> Task<EditMessage> {
        match message {
            EditMessage::SetCurrent(curr) => {
                if let Some(asset) = assets.get(curr) {
                    self.current = Some(curr);
                    self.temp_asset = Some(asset.clone());
                }

                Task::none()
            }
            EditMessage::UpdateTempAsset(edit) => {
                if let Some(asset) = &mut self.temp_asset {
                    edit.apply(asset);
                }
                Task::none()
            }
            EditMessage::Apply => {
                if let Some(handle) = self.current
                    && let Some(asset) = assets.get_mut(handle)
                {
                    *asset = self.temp_asset.clone().unwrap();
                };

                Task::none()
            }
            EditMessage::Cancel => {
                if let Some(current) = self.current {
                    Task::done(EditMessage::SetCurrent(current))
                } else {
                    Task::none()
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum EditMessage {
    SetCurrent(AssetHandle),
    UpdateTempAsset(Edit),
    Apply,
    Cancel,
}

#[derive(Debug, Clone)]
pub enum Edit {
    Image(EditImage),
    Character(EditCharacter),
}

#[derive(Debug, Clone)]
pub enum EditImage {}

#[derive(Debug, Clone)]
pub enum EditCharacter {
    Name(String),
    FileName(String),
    Image(AssetHandle),
}

impl Edit {
    fn apply(self, asset: &mut Asset) {
        match self {
            Self::Image(_) => (),
            Self::Character(edit) => {
                let Ok(chara) = <&mut Character>::try_from(asset) else {
                    return;
                };

                match edit {
                    EditCharacter::Name(name) => chara.name = name,
                    EditCharacter::FileName(_) => todo!(),
                    EditCharacter::Image(handle) => {
                        println!("set img of {} to {handle:?}", &chara.name);
                        chara.img = handle
                    }
                };
            }
        }
    }
}
