//! Mouse button, location and delta support for EZInput.

use std::hash::Hash;

use crate::prelude::*;
use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    math::Vec2,
    prelude::{Component, EventReader, MouseButton, Query, SystemLabel},
    window::CursorMoved,
};
use serde::{Deserialize, Serialize};

#[derive(SystemLabel, Clone, Hash, Debug, PartialEq, Eq)]
pub struct MouseInputHandlingSystem;

/// All types of axis that can be moved in a mouse.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Deserialize, Serialize)]
pub enum MouseAxisType {
    X,
    Y,
    Wheel,
}

/// Mouse button, location and delta support for EZInput.
#[derive(PartialEq, Debug, Component, Clone, Default)]
pub struct MouseMarker {
    pub mouse_position: Option<Vec2>,
    pub mouse_delta: Option<Vec2>,
    pub does_mouse_location_changed_this_tick: bool,
    pub does_mouse_wheel_changed_this_tick: bool,
}

impl MouseMarker {
    /// Change the current mouse location and delta and set the last input source to Mouse.
    pub fn set_mouse_location<Keys>(
        &mut self,
        view: &mut InputView<Keys>,
        position: Vec2,
        delta: Vec2,
    ) where
        Keys: BindingTypeView,
    {
        let state = PressState::Pressed {
            started_pressing_instant: None,
        };

        view.set_axis_value(
            InputReceiver::MouseAxis(MouseAxisType::X),
            position.x,
            state,
        );
        view.set_axis_value(
            InputReceiver::MouseAxis(MouseAxisType::Y),
            position.y,
            state,
        );
        view.set_axis_value(
            InputReceiver::MouseAxisDelta(MouseAxisType::X),
            delta.x,
            state,
        );
        view.set_axis_value(
            InputReceiver::MouseAxisDelta(MouseAxisType::Y),
            delta.y,
            state,
        );

        self.mouse_delta = Some(delta);
        self.mouse_position = Some(position);
        self.does_mouse_location_changed_this_tick = true;
        view.last_input_source = Some(InputSource::Mouse);
    }

    /// Tick the mouse by stop moving the axis when released.
    pub fn tick_mouse<Keys>(&mut self, view: &mut InputView<Keys>)
    where
        Keys: BindingTypeView,
    {
        view.descriptor_or_insert(InputReceiver::MouseAxis(MouseAxisType::X))
            .axis
            .press = PressState::Released;
        view.descriptor_or_insert(InputReceiver::MouseAxis(MouseAxisType::Y))
            .axis
            .press = PressState::Released;
        view.set_axis_value(
            InputReceiver::MouseAxis(MouseAxisType::Y),
            0.,
            PressState::Released,
        );
        view.set_axis_value(
            InputReceiver::MouseAxis(MouseAxisType::Wheel),
            0.,
            PressState::Released,
        );
        view.set_axis_value(
            InputReceiver::MouseAxisDelta(MouseAxisType::X),
            0.,
            PressState::Released,
        );
        view.set_axis_value(
            InputReceiver::MouseAxisDelta(MouseAxisType::Y),
            0.,
            PressState::Released,
        );
        view.set_axis_value(
            InputReceiver::MouseAxisDelta(MouseAxisType::Wheel),
            0.,
            PressState::Released,
        );
        self.does_mouse_location_changed_this_tick = false;
        self.does_mouse_wheel_changed_this_tick = false;
        self.mouse_delta = None;
    }

    /// Set the mouse button state for the given button and set the last input source to Mouse.
    pub fn set_mouse_button_state<Keys>(
        &mut self,
        view: &mut InputView<Keys>,
        button: MouseButton,
        state: PressState,
    ) where
        Keys: BindingTypeView,
    {
        view.last_input_source = Some(InputSource::Mouse);
        view.set_key_receiver_state(InputReceiver::MouseButton(button), state);
    }

    /// Set the mouse wheel state and set the last input source to Mouse.
    pub fn set_mouse_wheel_state<Keys>(
        &mut self,
        view: &mut InputView<Keys>,
        y: f32,
        state: PressState,
    ) where
        Keys: BindingTypeView,
    {
        view.last_input_source = Some(InputSource::Mouse);
        view.set_axis_value(InputReceiver::MouseAxis(MouseAxisType::Wheel), y, state);
        self.does_mouse_wheel_changed_this_tick = true;
    }
}

/// Input system responsible for handling mouse input and setting the button state for each updated button and axis.
pub(crate) fn mouse_input_system<Keys>(
    mut query: Query<(&mut InputView<Keys>, &mut MouseMarker)>,
    mut cursor_rd: EventReader<CursorMoved>,
    mut btn_rd: EventReader<MouseButtonInput>,
    mut mtn_rd: EventReader<MouseMotion>,
    mut wheel_rd: EventReader<MouseWheel>,
) where
    Keys: BindingTypeView,
{
    for (mut view, mut mouse_svc) in query.iter_mut() {
        let view = view.as_mut();
        let mouse_svc = mouse_svc.as_mut();
        mouse_svc.tick_mouse(view);

        for (abs_position, delta) in cursor_rd.iter().zip(mtn_rd.iter()) {
            mouse_svc.set_mouse_location(view, abs_position.position, delta.delta);
        }
        for ev in btn_rd.iter() {
            mouse_svc.set_mouse_button_state(view, ev.button, ev.state.into());
        }
        for ev in wheel_rd.iter() {
            let state = if ev.y > 0. {
                PressState::Pressed {
                    started_pressing_instant: None,
                }
            } else {
                PressState::Released    
            };
            mouse_svc.set_mouse_wheel_state(view, ev.y, state);
        }
    }
}
