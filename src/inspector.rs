use asset_system::{Asset, AssetHandle, AssetsData, Character, Image, image::DEFAULT_IMAGE};
use iced::{
    Alignment::Center,
    Element, Font,
    Length::*,
    Padding, Task,
    widget::{
        checkbox, column, container, horizontal_rule, horizontal_space, image, row, scrollable,
        text, text_input,
    },
};

use crate::{
    Draggable, State, style,
    widgets::{dnd::dnd_receiver, icon_button, icons, labeled_icon_button},
};

#[derive(Default)]
pub struct Inspector {
    current: Option<AssetHandle>,
    temp_asset: Option<Asset>,
    is_editing: bool,
    edits: Vec<Edit>,
    current_edit: usize,
    applied_edit: usize,
    is_pinned: bool,
}

fn capitalize(a: &str) -> String {
    let (first, rest) = a.split_at(1);

    first.to_uppercase() + rest
}

impl Inspector {
    pub fn view<'a: 'b, 'b>(&'b self, state: &'a State) -> Element<'b, InspectorMessage> {
        let Some(handle) = &self.current else {
            return horizontal_space().into();
        };
        let Some(_) = state.assets.get(*handle) else {
            return horizontal_space().into();
        };

        let path = state.assets.path(*handle).unwrap();

        let folder = capitalize(path.kind().folder());
        let name = path.name();

        let sep = text('>').size(14).style(text::secondary);

        let crumbs = row![text(folder), sep, text(name),]
            .spacing(3)
            .padding([0, 4])
            .align_y(Center);

        let Some(temp_asset) = self.temp_asset.as_ref() else {
            return "".into();
        };

        let content = match temp_asset {
            Asset::Image(img) => Element::from(
                container(
                    column![image(&img.handle).width(350), text(path.file_name())].align_x(Center),
                )
                .center(Fill),
            ),
            Asset::Character(chara) => {
                let img = state
                    .assets
                    .get_direct::<Image>(chara.img)
                    .unwrap_or(&DEFAULT_IMAGE);

                let mut title_font = Font::DEFAULT;
                title_font.weight = iced::font::Weight::Bold;

                let img_element = dnd_receiver(
                    |payload: &Draggable, _point| match payload {
                        Draggable::Asset(handle) => state.assets.is::<Image>(*handle).then_some(
                            InspectorMessage::UpdateTempAsset(Edit::Character(
                                EditCharacter::Image(*handle),
                            )),
                        ),
                    },
                    state.dnd_payload.as_ref(),
                    image(&img.handle).width(350),
                );

                let content = if self.is_editing {
                    let name_input = text_input("Enter Name...", &chara.name)
                        .size(24)
                        .font(title_font)
                        .align_x(Center)
                        .style(style::text_input_inline)
                        .on_input(|input| {
                            InspectorMessage::UpdateTempAsset(Edit::Character(EditCharacter::Name(
                                input,
                            )))
                        });

                    column![name_input, img_element].align_x(Center)
                } else {
                    let name = text(&chara.name)
                        .size(24)
                        .font(title_font)
                        .align_x(Center)
                        .style(text::base);

                    column![name, img_element].align_x(Center)
                };

                container(scrollable(content).style(style::scrollable))
                    .center(Fill)
                    .into()
            }
        };

        let top_row = row![crumbs, horizontal_space()]
            .push_maybe((!self.is_editing).then_some(
                labeled_icon_button(icons::EDIT, "Edit").on_press(InspectorMessage::EnterEditMode),
            ))
            .push_maybe(
                self.is_editing.then_some(
                    icon_button(icons::UNDO)
                        .on_press_maybe(self.can_undo().then_some(InspectorMessage::Undo)),
                ),
            )
            .push_maybe(
                self.is_editing.then_some(
                    icon_button(icons::REDO)
                        .on_press_maybe(self.can_redo().then_some(InspectorMessage::Redo)),
                ),
            )
            .spacing(4)
            .padding(2);

        let btn_width = 90;

        let action_row = self.is_editing.then_some(
            row![
                row![
                    labeled_icon_button(icons::CANCEL, "Cancel")
                        .style(style::secondary_button)
                        .on_press(InspectorMessage::Cancel)
                        .width(btn_width),
                    labeled_icon_button(icons::UNDO, "Revert")
                        .style(style::secondary_button)
                        .on_press_maybe(
                            (self.applied_edit != self.current_edit)
                                .then_some(InspectorMessage::Revert)
                        )
                        .width(btn_width),
                ]
                .width(FillPortion(1))
                .spacing(4),
                row![
                    horizontal_space(),
                    labeled_icon_button(icons::CHECK, "Done")
                        .on_press(InspectorMessage::Done)
                        .width(btn_width),
                    labeled_icon_button(icons::CHECK, "Apply")
                        .on_press_maybe(
                            (self.applied_edit != self.current_edit)
                                .then_some(InspectorMessage::Apply)
                        )
                        .width(btn_width),
                ]
                .width(FillPortion(1))
                .spacing(4)
            ]
            .spacing(4)
            .padding(2)
            .width(Fill),
        );

        column![top_row, container(content)]
            .push_maybe(self.is_editing.then_some(horizontal_rule(2)))
            .push_maybe(action_row)
            .padding(2)
            .into()
    }

    pub fn view_controls(&self) -> Element<'_, InspectorMessage> {
        container(
            checkbox("Pin", self.is_pinned)
                .on_toggle(InspectorMessage::SetPinned)
                .style(style::checkbox),
        )
        .padding(Padding::default().right(6))
        .center_y(Fill)
        .into()
    }

    pub fn update(
        &mut self,
        assets: &mut AssetsData,
        message: InspectorMessage,
    ) -> Task<InspectorMessage> {
        match message {
            InspectorMessage::SetCurrent(new, override_pin) => {
                if self.is_pinned && !override_pin && self.current.is_some() {
                    return Task::none();
                }
                if let Some(asset) = assets.get(new) {
                    // only clear edits if its a different asset
                    if self.current.is_some_and(|curr| curr != new) {
                        self.edits.clear();
                    }
                    // but if it is the same asset, make sure edits get un-applied or re-applied
                    // properly in the correct order, so undo/redo work properly without skipping
                    // any steps
                    else if let Some(temp_asset) = &mut self.temp_asset {
                        // edit = '∙', current_edit = 'C', applied_edit = 'A'
                        // [∙∙∙∙∙∙∙∙A∙∙∙∙∙∙C∙]
                        //          <-------
                        if self.current_edit > self.applied_edit {
                            self.edits[self.applied_edit..self.current_edit]
                                .iter_mut()
                                .rev()
                                .for_each(|edit| {
                                    edit.apply_and_invert(temp_asset);
                                });
                        }
                        // [∙∙C∙∙∙∙∙A∙∙∙∙∙∙∙∙]
                        //    ------>
                        else if self.current_edit < self.applied_edit {
                            self.edits[self.current_edit..self.applied_edit]
                                .iter_mut()
                                .for_each(|edit| {
                                    edit.apply_and_invert(temp_asset);
                                });
                        }
                    }
                    self.current = Some(new);
                    self.temp_asset = Some(asset.clone());
                    self.current_edit = self.applied_edit;
                }

                Task::none()
            }
            InspectorMessage::UpdateTempAsset(mut edit) => {
                if let Some(asset) = &mut self.temp_asset {
                    println!("applying edit to temp asset: {edit:#?}");

                    if !self.edits.is_empty() && self.current_edit != self.edits.len() {
                        println!("truncating {} edits", self.edits.len() - self.current_edit);
                        self.edits.truncate(self.current_edit);
                        println!("new length: {}", self.edits.len());
                    }
                    println!("applying edit: {edit:#?}");
                    edit.apply_and_invert(asset);
                    println!("inverted edit: {edit:#?}");
                    self.edits.push(edit);
                    self.current_edit = self.edits.len();
                    println!("pushed edit into edits; new length: {}", self.edits.len());
                    println!("current edit index: {}", self.current_edit);
                }
                Task::none()
            }
            InspectorMessage::EnterEditMode => {
                self.is_editing = true;
                Task::none()
            }
            InspectorMessage::ExitEditMode => {
                self.is_editing = false;
                Task::none()
            }
            InspectorMessage::Undo => {
                if self.current_edit != 0
                    && !self.edits.is_empty()
                    && let Some(asset) = self.temp_asset.as_mut()
                    && let Some(edit) = self.edits.get_mut(self.current_edit - 1)
                {
                    // TODO: apply stored edit (index self.current_edit - 1) to temp asset,
                    // replace stored edit with inverted one, self.current_edit -= 1
                    println!("Undoing edit {}", self.current_edit);
                    println!("applying edit: {edit:#?}");

                    edit.apply_and_invert(asset);
                    self.current_edit -= 1;

                    println!("inverted edit: {edit:#?}");
                    println!("current edit index: {}", self.current_edit);
                }

                Task::none()
            }
            InspectorMessage::Redo => {
                if self.current_edit != self.edits.len()
                    && !self.edits.is_empty()
                    && let Some(asset) = self.temp_asset.as_mut()
                    && let Some(edit) = self.edits.get_mut(self.current_edit)
                {
                    // TODO: apply stored edit (index self.current_edit) to temp asset,
                    // replace stored edit with inverted one, self.current_edit += 1
                    println!("Redoing edit {}", self.current_edit);
                    println!("applying edit: {edit:#?}");

                    edit.apply_and_invert(asset);
                    self.current_edit += 1;

                    println!("(un)inverted edit: {edit:#?}");
                    println!("current edit index: {}", self.current_edit);
                }

                Task::none()
            }
            InspectorMessage::Revert => {
                if let Some(current) = self.current {
                    Task::done(InspectorMessage::SetCurrent(current, true))
                } else {
                    Task::none()
                }
            }
            InspectorMessage::Cancel => Task::batch([
                Task::done(InspectorMessage::Revert),
                Task::done(InspectorMessage::ExitEditMode),
            ]),
            InspectorMessage::Apply => {
                self.apply(assets);
                self.applied_edit = self.current_edit;

                Task::none()
            }
            InspectorMessage::Done => {
                self.apply(assets);

                Task::done(InspectorMessage::ExitEditMode)
            }
            InspectorMessage::SetPinned(pinned) => {
                self.is_pinned = pinned;
                Task::none()
            }
        }
    }

    fn apply(&mut self, assets: &mut AssetsData) {
        if self.applied_edit == self.current_edit {
            return;
        }

        if let Some(handle) = self.current
            && let Some(asset) = assets.get_mut(handle)
        {
            *asset = self.temp_asset.clone().unwrap();
        };
    }

    fn can_undo(&self) -> bool {
        self.current_edit != 0
    }

    fn can_redo(&self) -> bool {
        !self.edits.is_empty() && self.current_edit < self.edits.len()
    }
}

#[derive(Debug, Clone)]
pub enum InspectorMessage {
    SetCurrent(AssetHandle, bool),
    UpdateTempAsset(Edit),
    EnterEditMode,
    ExitEditMode,
    Undo,
    Redo,
    Revert,
    Cancel,
    Apply,
    Done,
    SetPinned(bool),
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
    Image(AssetHandle),
}

impl Edit {
    /// Applies the edit to the asset and inverts itself (you can then apply it again to undo the
    /// edit)
    fn apply_and_invert(&mut self, asset: &mut Asset) {
        match self {
            Self::Image(_) => (),
            Self::Character(edit) => {
                let Ok(chara) = <&mut Character>::try_from(asset) else {
                    return;
                };

                let inverse = match edit {
                    EditCharacter::Name(name) => {
                        EditCharacter::Name(std::mem::replace(&mut chara.name, name.to_owned()))
                    }
                    EditCharacter::Image(handle) => {
                        EditCharacter::Image(std::mem::replace(&mut chara.img, handle.to_owned()))
                    }
                };

                *self = Self::Character(inverse);
            }
        }
    }
}
