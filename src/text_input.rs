use bevy::prelude::*;

pub struct TextInputPlugin;

impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (focus_text_fields_mouse, focus_text_fields_keyboard).in_set(TextInputSystemSet),
        )
        .add_observer(add_text_to_input);
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

fn add_text_to_input(
    trigger: Trigger<OnAdd, TextInput>,
    text_inputs: Query<&TextInput>,
    mut commands: Commands,
) -> Result {
    let text_input_entity = trigger.target();
    let text_input = text_inputs.get(text_input_entity)?;
    let text_input_text_entity = commands
        .spawn((Text(text_input.0.clone()), ChildOf(text_input_entity)))
        .id();
    commands
        .entity(text_input_entity)
        .insert(TextInputText(text_input_text_entity));
    Ok(())
}
