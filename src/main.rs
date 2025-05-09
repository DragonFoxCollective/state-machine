use bevy::color::palettes::css;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use itertools::Itertools as _;
use rand::distr::{Distribution, StandardUniform};
use uuid::Uuid;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_side_panel.run_if(resource_changed::<StateTypes>),
                update_nodes,
                draw_noodle,
            ),
        )
        .add_systems(Update, quit_on_esc)
        .add_observer(add_connector_observers)
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
            SidePanelText,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
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

    let state_type_1 = StateTypeData::new("Move Input Held");
    let state_type_1_id = state_type_1.id.clone();
    let state_type_2 = StateTypeData::new("Jump Input Held");
    let state_type_2_id = state_type_2.id.clone();

    let mut state_types = StateTypes::default();
    state_types.insert(state_type_1);
    state_types.insert(state_type_2);
    commands.insert_resource(state_types);

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
                        value: state_1,
                    },
                    StateValue {
                        state: state_type_2_id.clone(),
                        value: state_2,
                    },
                ],
            },
            Button,
            ChildOf(main_space),
        ));
    }
}

#[derive(Resource, Debug, Default)]
pub struct StateTypes(HashMap<StateType, StateTypeData>);

impl StateTypes {
    pub fn insert(&mut self, state_type: StateTypeData) {
        self.0.insert(state_type.id.clone(), state_type);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StateType(Uuid);

impl Distribution<StateType> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> StateType {
        StateType(Uuid::from_u128(rng.random()))
    }
}

#[derive(Debug)]
pub struct StateTypeData {
    pub id: StateType,
    pub name: String,
}

impl StateTypeData {
    pub fn new(name: &str) -> Self {
        Self {
            id: rand::random(),
            name: name.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StateValue {
    pub state: StateType,
    pub value: bool,
}

#[derive(Component, Clone, Debug)]
pub struct State {
    pub name: String,
    pub state: Vec<StateValue>,
}

#[derive(Component)]
#[require(Text)]
pub struct SidePanelText;

fn update_side_panel(
    mut side_panel: Query<&mut Text, With<SidePanelText>>,
    state_types: Res<StateTypes>,
) {
    for mut text in side_panel.iter_mut() {
        text.0 = state_types
            .0
            .iter()
            .map(|state_type| state_type.1.name.clone())
            .sorted()
            .join("\n");
    }
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
                    TextColor(if state_value.value {
                        css::GREEN.into()
                    } else {
                        css::RED.into()
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
                    BackgroundColor(if state_value.value {
                        css::RED.into()
                    } else {
                        css::GREEN.into()
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
        println!("Noodle connected to wrong side, removing");
        return Ok(());
    }

    if noodles.iter().any(|noodle| {
        matches!(&noodle, Noodle::Connected { start_connector, end_connector }
			if (*start_connector == new_connector && *end_connector == connector)
			|| (*start_connector == connector && *end_connector == new_connector))
    }) {
        println!("Noodle already exists, removing");
        return Ok(());
    }

    println!("Connecting noodle");
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
    println!("Dropping noodle");
    commands.entity(noodle).despawn();
    commands.entity(connector).remove::<DraggedConnector>();

    Ok(())
}

fn quit_on_esc(mut exit: EventWriter<AppExit>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
