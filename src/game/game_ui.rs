use bevy::{app::{Plugin, Update}, ecs::{component::Component, entity::Entity, query::With, schedule::IntoSystemConfigs, system::{Commands, Query, Res}}, hierarchy::{BuildChildren, ChildBuild}, state::{condition::in_state, state::{OnEnter, State}}, transform::components::GlobalTransform, ui::{widget::ImageNode, JustifyContent, Node, Val}, utils::default};

use crate::{assets::GameUiAssets, GameStates};

use super::{GameScreen, LevelState, StatePlugin};

pub struct GameUiPlugin(GameStates);

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(OnEnter(self.0.clone()), setup);
        app.add_systems(Update, (update_chalkboard).run_if(in_state(self.0.clone())));
    }
}

impl StatePlugin<GameUiPlugin> for GameUiPlugin {
    fn run_on_state(state: GameStates) -> GameUiPlugin {
        GameUiPlugin(state)
    }
}


fn setup(mut commands: Commands, assets: Res<GameUiAssets>){
    commands.spawn((Node {
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        justify_content: JustifyContent::SpaceBetween,
        ..default()
    }, GameScreen))
    .with_children(|root| {
        root.spawn((ImageNode::new(assets.chalkboard.clone()),
        Node {
            position_type: bevy::ui::PositionType::Absolute,
            top: Val::Px(50.),
            right: Val::Px(50.),
            width: Val::Px(134.),
            height: Val::Px(67.),
            flex_direction: bevy::ui::FlexDirection::Row,
            justify_content: JustifyContent::SpaceEvenly,
            align_items: bevy::ui::AlignItems::Center,
            ..default()
        }
    ))
        .with_children(| chalkboard| {
            let checkbox = (ImageNode::new(assets.checkbox_empty.clone()), CheckBoxEmpty, Node {
                width: Val::Px(26.),
                height: Val::Px(26.),
                ..default()
            });
            for _ in 0..3 {
                chalkboard.spawn(checkbox.clone());
            }
        });
    });

}

#[derive(Component, Debug, Clone)]
struct CheckBoxEmpty;
#[derive(Component, Debug, Clone)]
struct CheckBoxFailed;

fn update_chalkboard(mut commands: Commands, image_nodes: Query<(Entity, &GlobalTransform), With<CheckBoxEmpty>>, failed_checkboxes: Query<&CheckBoxFailed>, level_state: Res<State<LevelState>>, assets: Res<GameUiAssets>){
    let LevelState::OrdersFailed(num_failed) = level_state.get() else { return };
    let failed_checkboxes = failed_checkboxes.iter().len();
    if failed_checkboxes >= *num_failed {
        return;
    }

    image_nodes.iter().sort_by::<&GlobalTransform>(|item_1, item_2| {
        item_1.translation().x.total_cmp(&item_2.translation().x)
    }).take(num_failed - failed_checkboxes).for_each(|(image_node, _)| {
        let mut image_node = commands.entity(image_node);
        image_node.remove::<ImageNode>();
        image_node.remove::<CheckBoxEmpty>();
        image_node.insert((CheckBoxFailed, ImageNode::new(assets.checkbox_failed.clone())));
    });
}