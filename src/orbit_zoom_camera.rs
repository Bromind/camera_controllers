#![allow(dead_code)]

//!
//! A 3dsMax / Blender style camera that orbits about a target position
//!

use event::GenericEvent;

use std::num::{Float, FromPrimitive};
use vecmath::{
    Vector3,
    vec3_add,
    vec3_scale
};

use quaternion;
use quaternion::{ Quaternion };

use { input, Camera };

use input::Button::{Keyboard, Mouse};
use input::keyboard::Key;
use input::mouse::MouseButton;

bitflags!(
    flags Keys: u8 {
        const ZOOM  = 0b00000001,
        const PAN   = 0b00000010,
        const ORBIT = 0b00000100,
    }
);

///
/// Specifies key bindings and speed modifiers for OrbitZoomCamera
///
pub struct OrbitZoomCameraSettings<T=f32> {

    /// Which button to press to orbit with mouse
    pub orbit_button: input::Button,

    /// Which button to press to zoom with mouse
    pub zoom_button: input::Button,

    /// Which button to press to pan with mouse
    pub pan_button: input::Button,

    /// Modifier for orbiting speed (arbitrary unit)
    pub orbit_speed: T,

    /// Modifier for panning speed (arbitrary unit)
    pub pan_speed: T,

    /// Modifier for zoom speed (arbitrary unit)
    pub zoom_speed: T,
}

impl<T: Float + FromPrimitive> OrbitZoomCameraSettings<T> {

    ///
    /// Clicking and dragging OR two-finger scrolling will orbit camera,
    /// with LShift as pan modifer and LCtrl as zoom modifier
    ///
    pub fn default() -> OrbitZoomCameraSettings<T> {
        OrbitZoomCameraSettings {
            orbit_button : Mouse(MouseButton::Left),
            zoom_button : Keyboard(Key::LCtrl),
            pan_button : Keyboard(Key::LShift),
            orbit_speed: FromPrimitive::from_f32(0.05).unwrap(),
            pan_speed: FromPrimitive::from_f32(0.1).unwrap(),
            zoom_speed: FromPrimitive::from_f32(0.1).unwrap(),
        }
    }

}

///
/// A 3dsMax / Blender-style camera that orbits around a target point
///
pub struct OrbitZoomCamera<T=f32> {

    /// origin of camera rotation
    pub target: Vector3<T>,

    /// Rotation of camera
    pub rotation: Quaternion<T>,

    /// Pitch up/down from target
    pub pitch: T,

    /// Yaw left/right from target
    pub yaw: T,

    /// camera distance from target
    pub distance: T,

    /// Settings for the camera
    pub settings: OrbitZoomCameraSettings<T>,

    /// Current keys that are pressed
    keys: Keys,
}


impl<T: Float + FromPrimitive>
OrbitZoomCamera<T> {

    ///
    /// Create a new OrbitZoomCamera targeting the given coordinates
    ///
    pub fn new(target: [T; 3], settings: OrbitZoomCameraSettings<T>) -> OrbitZoomCamera<T> {
        OrbitZoomCamera {
            target: target,
            rotation: quaternion::id(),
            distance: FromPrimitive::from_f32(10.0).unwrap(),
            pitch: Float::zero(),
            yaw: Float::zero(),
            keys: Keys::empty(),
            settings: settings
        }
    }

    ///
    /// Return a Camera for the current OrbitZoomCamera configuration
    ///
    pub fn camera(&self, _dt: f64) -> Camera<T> {
        let target_to_camera = quaternion::rotate_vector(
            self.rotation, 
            [Float::zero(), Float::zero(), self.distance]
        );
        let mut camera = Camera::new(vec3_add(self.target, target_to_camera));
        camera.set_rotation(self.rotation);
        camera
    }

    ///
    /// Orbit the camera using the given horizontal and vertical params,
    /// or zoom or pan if the appropriate modifier keys are pressed
    ///
    fn control_camera(&mut self, dx: T, dy: T) {

        let _1 = Float::one();
        let _0 = Float::zero();

        if self.keys.contains(PAN) {

            // Pan target position along plane normal to camera direction
            let dx = dx * self.settings.pan_speed;
            let dy = dy * self.settings.pan_speed;

            let right = quaternion::rotate_vector(self.rotation, [_1, _0, _0]);
            let up = quaternion::rotate_vector(self.rotation, [_0, _1, _0]);
            self.target = vec3_add(
                vec3_add(self.target, vec3_scale(up, dy)),
                vec3_scale(right,dx)
            );

        } else if self.keys.contains(ZOOM) {

            // Zoom to / from target
            self.distance = self.distance + dy * self.settings.zoom_speed;

        } else {

            // Orbit around target
            let dx = dx * self.settings.orbit_speed;
            let dy = dy * self.settings.orbit_speed;

            self.yaw = self.yaw + dx;
            self.pitch = self.pitch + dy;
            self.rotation = quaternion::mul(
                quaternion::axis_angle([_0, _1, _0], self.yaw),
                quaternion::axis_angle([_1, _0, _0], self.pitch)
            );

        }
    }

    ///
    /// Respond to scroll and key press/release events
    ///
    pub fn event<E: GenericEvent>(&mut self, e: &E) {

        use event::{ MouseRelativeEvent, MouseScrollEvent, PressEvent, ReleaseEvent };

        e.mouse_scroll(|dx, dy| {
            let dx: T = FromPrimitive::from_f64(dx).unwrap();
            let dy: T = FromPrimitive::from_f64(dy).unwrap();
            self.control_camera(dx, dy);
        });

        e.mouse_relative(|dx, dy| {
            let dx: T = FromPrimitive::from_f64(dx).unwrap();
            let dy: T = FromPrimitive::from_f64(dy).unwrap();
            if self.keys.contains(ORBIT){
                self.control_camera(-dx, dy);
            }
        });

        e.press(|button| {
            match button {
                x if x == self.settings.orbit_button => self.keys.insert(ORBIT),
                x if x == self.settings.pan_button => self.keys.insert(PAN),
                x if x == self.settings.zoom_button => self.keys.insert(ZOOM),
                _ => {}
            }
        });

        e.release(|button| {
            match button {
                x if x == self.settings.orbit_button => self.keys.remove(ORBIT),
                x if x == self.settings.pan_button => self.keys.remove(PAN),
                x if x == self.settings.zoom_button => self.keys.remove(ZOOM),
                _ => {}
            }
        });
    }
}
