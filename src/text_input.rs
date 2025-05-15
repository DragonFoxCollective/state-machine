use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

pub struct TextInputPlugin;

impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                focus_text_fields_mouse,
                focus_text_fields_keyboard,
                keyboard_input,
                update_text.after(keyboard_input),
            )
                .in_set(TextInputSystemSet),
        )
        .add_observer(create_rest_of_input);
    }
}

#[derive(Component, Debug, Default)]
#[require(Button, TextInputActive)]
pub struct TextInput(pub String);

#[derive(Event, Debug, Default)]
pub struct TextInputFocused;

#[derive(Event, Debug, Default)]
pub struct TextInputUnfocused;

#[derive(Event, Debug, Default)]
pub struct TextInputSubmitted;

#[derive(Component, Debug)]
struct TextInputText(Entity);

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextInputActive {
    Active,
    #[default]
    Inactive,
}

#[derive(SystemSet, Hash, Debug, Clone, Eq, PartialEq)]
pub struct TextInputSystemSet;

fn focus_text_fields_mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    mut text_inputs: Query<(Entity, &mut TextInputActive, &Interaction)>,
    mut commands: Commands,
) {
    if buttons.just_pressed(MouseButton::Left) {
        for (entity, mut active, interaction) in text_inputs.iter_mut() {
            match (
                *active == TextInputActive::Active,
                *interaction == Interaction::Pressed,
            ) {
                (false, true) => {
                    *active = TextInputActive::Active;
                    commands.entity(entity).trigger(TextInputFocused);
                }
                (true, false) => {
                    *active = TextInputActive::Inactive;
                    commands.entity(entity).trigger(TextInputUnfocused);
                }
                _ => {}
            }
        }
    }
}

fn focus_text_fields_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut text_inputs: Query<(Entity, &mut TextInputActive)>,
    mut commands: Commands,
) {
    if keys.any_just_pressed([KeyCode::Escape, KeyCode::Enter]) {
        for (entity, mut active) in text_inputs.iter_mut() {
            if *active == TextInputActive::Active {
                *active = TextInputActive::Inactive;
                let mut entity = commands.entity(entity);
                entity.trigger(TextInputUnfocused);
                if keys.just_pressed(KeyCode::Enter) {
                    entity.trigger(TextInputSubmitted);
                }
            }
        }
    }
}

fn create_rest_of_input(
    trigger: Trigger<OnAdd, TextInput>,
    text_inputs: Query<&TextInput>,
    mut commands: Commands,
) -> Result {
    let text_input_entity = trigger.target();
    let text_input = text_inputs.get(text_input_entity)?;
    let text_input_text_entity = commands
        .spawn((
            Text(text_input.0.clone()),
            TextLayout {
                linebreak: LineBreak::WordOrCharacter,
                ..default()
            },
            ChildOf(text_input_entity),
        ))
        .id();
    commands
        .entity(text_input_entity)
        .insert(TextInputText(text_input_text_entity));
    Ok(())
}

fn keyboard_input(
    mut keys: EventReader<KeyboardInput>,
    mut text_inputs: Query<(&mut TextInput, &TextInputActive)>,
) {
    for key in keys.read() {
        for (mut text_input, _) in text_inputs
            .iter_mut()
            .filter(|(_, active)| **active == TextInputActive::Active)
        {
            if !key.state.is_pressed() {
                continue;
            };

            match key.logical_key {
                Key::Character(ref c) => {
                    text_input.0.push_str(c.as_str());
                }

                Key::Backspace => {
                    text_input.0.pop();
                }

                Key::Space => {
                    text_input.0.push(' ');
                }

                Key::Escape | Key::Enter => {}

                _ => {
                    warn!("Unhandled key input: {:?}", key.logical_key)
                }
            }
        }
    }
}

fn update_text(
    text_inputs: Query<(&TextInput, &TextInputText), Changed<TextInput>>,
    mut text_input_texts: Query<&mut Text>,
) {
    for (text_input, text_input_text) in text_inputs.iter() {
        if let Ok(mut text) = text_input_texts.get_mut(text_input_text.0) {
            text.0 = text_input.0.clone();
        }
    }
}
