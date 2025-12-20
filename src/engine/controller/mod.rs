use std::collections::VecDeque;

use winit::event::KeyEvent;


#[derive(Debug)]
pub enum Key {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
    TurnLeft,
    TurnRight,
}

#[derive(Debug)]
pub enum KeyState {
    Press,
    Release,
}

#[derive(Debug, Default)]
pub struct ControllerKey {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub turnleft: bool,
    pub turnright: bool,
}

// type KeyCmd = (Key, KeyState);

#[derive(Debug, Default)]
pub struct Controller {
    // commands: VecDeque<KeyCmd>,
    keys: ControllerKey,
}

impl Controller {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_state(&self) -> &ControllerKey {
        &self.keys
    }

    pub fn set_state(&mut self, key: Key, state: bool) -> anyhow::Result<()> {
        // let state = match state {
        //     KeyState::Press => true,
        //     KeyState::Release => false,
        // };
        match key {
            Key::Forward => {self.keys.forward = state;},
            Key::Backward => {self.keys.backward = state;},
            Key::Left => {self.keys.left = state;},
            Key::Right => {self.keys.right = state;},
            Key::Up => {self.keys.up = state;},
            Key::Down => {self.keys.down = state;},
            Key::TurnLeft => {self.keys.turnleft = state;},
            Key::TurnRight => {self.keys.turnright = state;},
        }
        Ok(())
    }
}

impl Controller {
    pub fn parse_key_event(&mut self, event: &KeyEvent) -> bool {
        match event {
            KeyEvent {
                state,
                physical_key,
                logical_key,
                ..
            } => match (logical_key, physical_key) {
                (winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space), _) => {
                    let _ = self.set_state(Key::Up, state.is_pressed());
                    true
                }
                (_, winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Space)) => {
                    let _ = self.set_state(Key::Up, state.is_pressed());
                    true
                }
                (_, winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ShiftLeft)) => {
                    let _ = self.set_state(Key::Down, state.is_pressed());
                    true
                }
                (_, winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyW)) => {
                    let _ = self.set_state(Key::Forward, state.is_pressed());
                    true
                }
                (_, winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyS)) => {
                    let _ = self.set_state(Key::Backward, state.is_pressed());
                    true
                }
                (_, winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyA)) => {
                    let _ = self.set_state(Key::Left, state.is_pressed());
                    true
                }
                (_, winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyD)) => {
                    let _ = self.set_state(Key::Right, state.is_pressed());
                    true
                }
                (_, winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyQ)) => {
                    let _ = self.set_state(Key::TurnLeft, state.is_pressed());
                    true
                }
                (_, winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyE)) => {
                    let _ = self.set_state(Key::TurnRight, state.is_pressed());
                    true
                }
                _ => false,
            },
        }
    }
}