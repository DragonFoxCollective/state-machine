use bevy::color::palettes::css;
use bevy::log::LogPlugin;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use itertools::Itertools as _;
use rand::distr::{Distribution, StandardUniform};
use text_input::{TextInput, TextInputFocused, TextInputPlugin, TextInputUnfocused};
use uuid::Uuid;

pub mod text_input;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "info,wgpu=error,naga=warn,state_machine=debug".into(),
            ..default()
        }))
        .add_plugins(TextInputPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (update_nodes, draw_noodle))
        .add_systems(Update, quit_on_esc)
        .add_observer(add_connector_observers)
        .add_observer(add_node_observers)
        .add_observer(add_state_to_side_panel)
        .add_observer(remove_state_from_side_panel)
        .add_observer(update_side_panel_state_name)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d,));

    let root = commands
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },))
        .id();

    let side_panel = commands
        .spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Percent(100.0),
                border: UiRect::all(Val::Px(5.0)).with_left(Val::Auto),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(css::DARK_GRAY.into()),
            BorderColor(css::GRAY.into()),
            BorderRadius::right(Val::Px(10.0)),
            FocusPolicy::Block,
            ChildOf(root),
        ))
        .id();

    let _side_panel_text = commands
        .spawn((
            SidePanel,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ChildOf(side_panel),
        ))
        .id();

    let main_space = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            ChildOf(root),
        ))
        .id();

    let state_type_1 = StateTypeData::new("Move Input Held", StateType::Bool);
    let state_type_1_id = state_type_1.id.clone();
    let state_type_2 = StateTypeData::new("Jump Input Held", StateType::Bool);
    let state_type_2_id = state_type_2.id.clone();

    let mut state_types = StateTypes::default();
    state_types.insert(state_type_1);
    state_types.insert(state_type_2);
    commands.insert_resource(state_types);

    commands.trigger(StateTypeAdded {
        state_type: state_type_1_id.clone(),
    });
    commands.trigger(StateTypeAdded {
        state_type: state_type_2_id.clone(),
    });

    for (name, position, state_1, state_2) in [
        ("Idle", Vec2::new(50.0, 50.0), false, false),
        ("Hovering", Vec2::new(50.0, 250.0), true, true),
        ("Walking", Vec2::new(500.0, 100.0), true, false),
        ("Jumping", Vec2::new(500.0, 300.0), false, true),
    ] {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(position.x),
                top: Val::Px(position.y),
                border: UiRect::all(Val::Px(10.0)),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(css::MAROON.into()),
            BorderColor(css::RED.into()),
            BorderRadius::all(Val::Px(10.0)),
            State {
                name: name.to_string(),
                state: vec![
                    StateValue {
                        state: state_type_1_id.clone(),
                        value: StateTypeValue::Bool(state_1),
                    },
                    StateValue {
                        state: state_type_2_id.clone(),
                        value: StateTypeValue::Bool(state_2),
                    },
                ],
            },
            Button,
            ChildOf(main_space),
        ));
    }
}

#[derive(Resource, Debug, Default, Deref, DerefMut)]
pub struct StateTypes(HashMap<StateId, StateTypeData>);

impl StateTypes {
    pub fn insert(&mut self, state_type: StateTypeData) {
        self.0.insert(state_type.id.clone(), state_type);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StateId(Uuid);

#[derive(Debug)]
pub enum StateType {
    Bool,
}

#[derive(Debug, Clone)]
pub enum StateTypeValue {
    Bool(bool),
}

impl Distribution<StateId> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> StateId {
        StateId(Uuid::from_u128(rng.random()))
    }
}

#[derive(Debug)]
pub struct StateTypeData {
    pub id: StateId,
    pub name: String,
    pub state_type: StateType,
}

impl StateTypeData {
    pub fn new(name: impl ToString, state_type: StateType) -> Self {
        Self {
            id: rand::random(),
            name: name.to_string(),
            state_type,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StateValue {
    pub state: StateId,
    pub value: StateTypeValue,
}

#[derive(Component, Clone, Debug)]
pub struct State {
    pub name: String,
    pub state: Vec<StateValue>,
}

#[derive(Component)]
pub struct SidePanel;

#[derive(Component)]
pub struct StateNameTextInput(pub StateId);

#[derive(Event)]
pub struct StateTypeAdded {
    pub state_type: StateId,
}

#[derive(Event)]
pub struct StateTypeNameChanged {
    pub state_type: StateId,
    pub name: String,
}

#[derive(Event)]
pub struct StateTypeRemoved {
    pub state_type: StateId,
}

fn add_state_to_side_panel(
    trigger: Trigger<StateTypeAdded>,
    mut side_panel: Query<Entity, With<SidePanel>>,
    state_types: Res<StateTypes>,
    mut commands: Commands,
) -> Result {
    let state_type = state_types
        .0
        .get(&trigger.state_type)
        .ok_or("StateType not found")?;
    debug!("Adding state type to side panel: {:?}", state_type);
    for panel in side_panel.iter_mut() {
        commands
            .spawn((
                Node {
                    border: UiRect::all(Val::Px(5.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                TextInput(state_type.name.clone()),
                BackgroundColor(css::GRAY.into()),
                BorderColor(css::BLACK.into()),
                StateNameTextInput(state_type.id.clone()),
                ChildOf(panel),
            ))
            .observe(update_state_names)
            .observe(text_field_focused_colors)
            .observe(text_field_unfocused_colors);
    }
    Ok(())
}

fn remove_state_from_side_panel(
    trigger: Trigger<StateTypeRemoved>,
    mut commands: Commands,
    state_name_text_inputs: Query<(Entity, &StateNameTextInput)>,
) {
    for (entity, _) in state_name_text_inputs
        .iter()
        .filter(|(_, state_name_text_input)| state_name_text_input.0 == trigger.state_type)
    {
        commands.entity(entity).despawn();
    }
}

fn update_side_panel_state_name(
    trigger: Trigger<StateTypeNameChanged>,
    mut state_name_text_inputs: Query<(&StateNameTextInput, &mut TextInput)>,
) {
    for (_, mut text_input) in state_name_text_inputs
        .iter_mut()
        .filter(|(state_name_text_input, _)| state_name_text_input.0 == trigger.state_type)
    {
        text_input.0 = trigger.name.clone();
    }
}

fn update_state_names(
    trigger: Trigger<TextInputUnfocused>,
    mut text_inputs: Query<(&StateNameTextInput, &mut TextInput)>,
    mut state_types: ResMut<StateTypes>,
    mut commands: Commands,
) -> Result {
    let (state_name, mut text_input) = text_inputs.get_mut(trigger.target())?;
    let state_type = state_types
        .get_mut(&state_name.0)
        .ok_or("StateType not found")?;
    if text_input.0.is_empty() {
        text_input.0 = state_type.name.clone();
    } else if text_input.0 != state_type.name {
        state_type.name = text_input.0.clone();
        commands.trigger(StateTypeNameChanged {
            state_type: state_name.0.clone(),
            name: text_input.0.clone(),
        });
    }
    Ok(())
}

fn text_field_focused_colors(
    trigger: Trigger<TextInputFocused>,
    mut text_inputs: Query<&mut BorderColor>,
) -> Result {
    let mut border_color = text_inputs.get_mut(trigger.target())?;
    border_color.0 = css::WHITE.into();
    Ok(())
}

fn text_field_unfocused_colors(
    trigger: Trigger<TextInputUnfocused>,
    mut text_inputs: Query<&mut BorderColor>,
) -> Result {
    let mut border_color = text_inputs.get_mut(trigger.target())?;
    border_color.0 = css::BLACK.into();
    Ok(())
}

fn update_nodes(
    nodes: Query<(Entity, &State), Changed<State>>,
    mut commands: Commands,
    state_types: Res<StateTypes>,
) {
    for (node, state) in nodes.iter() {
        commands.entity(node).despawn_related::<Children>();

        commands.spawn((Text(state.name.clone()), ChildOf(node)));

        let _enter_connector = commands
            .spawn((
                Node {
                    width: Val::Px(15.0),
                    height: Val::Px(15.0),
                    border: UiRect::all(Val::Px(3.0)),
                    position_type: PositionType::Absolute,
                    left: Val::Px(-20.0),
                    ..default()
                },
                BackgroundColor(css::WHITE.into()),
                BorderRadius::all(Val::Percent(100.0)),
                BorderColor(css::BLACK.into()),
                Connector::Enter,
                Button,
                ChildOf(node),
            ))
            .id();

        for (state_name, state_value) in state
            .state
            .iter()
            .map(|value| (state_types.0.get(&value.state).unwrap().name.clone(), value))
            .sorted_by_key(|(name, _)| name.clone())
        {
            let text = commands
                .spawn((
                    Text(state_name),
                    Node::default(),
                    TextColor(match state_value.value {
                        StateTypeValue::Bool(value) => {
                            if value {
                                css::GREEN.into()
                            } else {
                                css::RED.into()
                            }
                        }
                    }),
                ))
                .id();

            let connector_anchor = commands
                .spawn(Node {
                    height: Val::Percent(50.0),
                    ..default()
                })
                .id();

            let _connector = commands
                .spawn((
                    Node {
                        width: Val::Px(15.0),
                        height: Val::Px(15.0),
                        border: UiRect::all(Val::Px(3.0)),
                        position_type: PositionType::Absolute,
                        left: Val::Px(15.0),
                        ..default()
                    },
                    BackgroundColor(match state_value.value {
                        StateTypeValue::Bool(value) => {
                            if value {
                                css::GREEN.into()
                            } else {
                                css::RED.into()
                            }
                        }
                    }),
                    BorderRadius::all(Val::Percent(100.0)),
                    BorderColor(css::BLACK.into()),
                    Connector::Exit,
                    Button,
                    ChildOf(connector_anchor),
                ))
                .id();

            commands
                .spawn((
                    Node {
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ChildOf(node),
                ))
                .add_child(text)
                .add_child(connector_anchor);
        }
    }
}

#[derive(Component)]
enum Noodle {
    Connected {
        start_connector: Entity,
        end_connector: Entity,
    },
    HangingStart {
        start_position: Vec2,
        end_connector: Entity,
    },
    HangingEnd {
        start_connector: Entity,
        end_position: Vec2,
    },
}

#[derive(Component)]
enum Connector {
    Enter,
    Exit,
}

fn draw_noodle(
    noodles: Query<&Noodle>,
    connectors: Query<&GlobalTransform, With<Connector>>,
    window: Query<&Window>,
    mut gizmos: Gizmos,
) -> Result {
    let window = window.single()?;
    for noodle in noodles.iter() {
        let start = (match noodle {
            Noodle::Connected {
                start_connector, ..
            } => connectors.get(*start_connector)?.translation().xy(),
            Noodle::HangingStart { start_position, .. } => *start_position,
            Noodle::HangingEnd {
                start_connector, ..
            } => connectors.get(*start_connector)?.translation().xy(),
        } - window.size() / 2.0)
            * Vec2::new(1.0, -1.0);

        let end = (match noodle {
            Noodle::Connected { end_connector, .. } => {
                connectors.get(*end_connector)?.translation().xy()
            }
            Noodle::HangingStart { end_connector, .. } => {
                connectors.get(*end_connector)?.translation().xy()
            }
            Noodle::HangingEnd { end_position, .. } => *end_position,
        } - window.size() / 2.0)
            * Vec2::new(1.0, -1.0);

        let bezier = CubicBezier::new([[
            start,
            start + Vec2::new(100.0, 0.0),
            end - Vec2::new(100.0, 0.0),
            end,
        ]]);
        let curve = bezier.to_curve().unwrap();
        let resolution = 100 * curve.segments().len();
        gizmos.linestrip(
            curve.iter_positions(resolution).map(|pt| pt.extend(0.0)),
            Color::srgb(1.0, 1.0, 1.0),
        );
    }
    Ok(())
}

#[derive(Component)]
struct DraggedConnector {
    noodle: Entity,
}

fn add_node_observers(trigger: Trigger<OnAdd, State>, mut commands: Commands) {
    commands.entity(trigger.target()).observe(be_dragging_node);
}

fn add_connector_observers(trigger: Trigger<OnAdd, Connector>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .observe(start_dragging_connector)
        .observe(be_dragging_connector)
        .observe(drag_and_drop_connector)
        .observe(stop_dragging_connector);
}

fn start_dragging_connector(
    trigger: Trigger<Pointer<DragStart>>,
    connectors: Query<&Connector>,
    mut commands: Commands,
    window: Query<&Window>,
) -> Result {
    let window = window.single()?;
    let connector = trigger.target();
    let noodle = match connectors.get(connector)? {
        Connector::Enter => commands
            .spawn((Noodle::HangingStart {
                start_position: window.cursor_position().unwrap_or_default(),
                end_connector: connector,
            },))
            .id(),
        Connector::Exit => commands
            .spawn((Noodle::HangingEnd {
                start_connector: connector,
                end_position: window.cursor_position().unwrap_or_default(),
            },))
            .id(),
    };
    commands
        .entity(connector)
        .insert(DraggedConnector { noodle });
    Ok(())
}

fn be_dragging_connector(
    trigger: Trigger<Pointer<Drag>>,
    dragged_connectors: Query<&DraggedConnector>,
    mut noodles: Query<&mut Noodle>,
    window: Query<&Window>,
) -> Result {
    let connector = trigger.target();
    let noodle = dragged_connectors.get(connector)?.noodle;
    let mut noodle = noodles.get_mut(noodle)?;
    let window = window.single()?;
    *noodle = match *noodle {
        Noodle::HangingStart { start_position, .. } => Noodle::HangingStart {
            start_position: window.cursor_position().unwrap_or(start_position),
            end_connector: connector,
        },
        Noodle::HangingEnd { end_position, .. } => Noodle::HangingEnd {
            start_connector: connector,
            end_position: window.cursor_position().unwrap_or(end_position),
        },
        _ => unreachable!(),
    };
    Ok(())
}

fn drag_and_drop_connector(
    trigger: Trigger<Pointer<DragDrop>>,
    mut commands: Commands,
    connectors: Query<&Connector>,
    dragged_connectors: Query<&DraggedConnector>,
    mut noodles: Query<&mut Noodle>,
) -> Result {
    let connector = trigger.dropped;
    let new_connector = trigger.target();
    let new_connector_type = connectors.get(new_connector)?;
    let noodle = dragged_connectors.get(connector)?.noodle;
    let noodle_type = noodles.get(noodle)?;

    if matches!(
        (noodle_type, new_connector_type),
        (Noodle::HangingEnd { .. }, Connector::Exit)
            | (Noodle::HangingStart { .. }, Connector::Enter)
    ) {
        debug!("Noodle connected to wrong side, removing");
        return Ok(());
    }

    if noodles.iter().any(|noodle| {
        matches!(&noodle, Noodle::Connected { start_connector, end_connector }
			if (*start_connector == new_connector && *end_connector == connector)
			|| (*start_connector == connector && *end_connector == new_connector))
    }) {
        debug!("Noodle already exists, removing");
        return Ok(());
    }

    debug!("Connecting noodle");
    let mut noodle = noodles.get_mut(noodle)?;
    *noodle = match *noodle {
        Noodle::HangingStart { .. } => Noodle::Connected {
            start_connector: new_connector,
            end_connector: connector,
        },
        Noodle::HangingEnd { .. } => Noodle::Connected {
            start_connector: connector,
            end_connector: new_connector,
        },
        _ => unreachable!(),
    };
    commands.entity(connector).remove::<DraggedConnector>();

    Ok(())
}

fn stop_dragging_connector(
    trigger: Trigger<Pointer<DragEnd>>,
    mut commands: Commands,
    dragged_connectors: Query<&DraggedConnector>,
) -> Result {
    let connector = trigger.target();
    let noodle = match dragged_connectors.get(connector) {
        Ok(dragged) => dragged.noodle,
        Err(_) => return Ok(()), // Could've been handled by [`drag_and_drop_connector`]
    };
    debug!("Dropping noodle");
    commands.entity(noodle).despawn();
    commands.entity(connector).remove::<DraggedConnector>();

    Ok(())
}

fn be_dragging_node(
    trigger: Trigger<Pointer<Drag>>,
    mut nodes: Query<&mut Node>,
    children: Query<&Children>,
    interactions: Query<&Interaction>,
) -> Result {
    let node = trigger.target();
    if children.iter_descendants(node).any(|child| {
        interactions
            .get(child)
            .is_ok_and(|i| !matches!(i, Interaction::None))
    }) {
        return Ok(());
    }

    let mut node = nodes.get_mut(node)?;
    node.left = Val::Px(
        match node.left {
            Val::Px(x) => x,
            _ => unreachable!(),
        } + trigger.delta.x,
    );
    node.top = Val::Px(
        match node.top {
            Val::Px(y) => y,
            _ => unreachable!(),
        } + trigger.delta.y,
    );
    Ok(())
}

fn quit_on_esc(mut exit: EventWriter<AppExit>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
