//! a slider element that allows selecting a value in a range

use derive_setters::Setters;
use glam::{Vec2, vec2};

use crate::{
  draw::UiDrawCommand, element::{MeasureContext, ProcessContext, UiElement}, frame::{Frame, FrameRect}, layout::{compute_size, Size2d}, measure::Response, rect::FillColor, signal::{trigger::SignalTriggerArg, Signal}
};


//TODO: use state for slider?
// ^ useful if the user only hanldes the drag end event or has large step sizes with relative mode

//TODO: adopt frame api here

/// Follow mode for the slider
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum SliderFollowMode {
  /// Slider will change based on the absolute mouse position in the slider
  ///
  /// This is the default mode and is recommended for most use cases
  #[default]
  Absolute,

  /// Slider will change based on the difference between the current and starting mouse position
  ///
  /// This is an experimental option and does not currently work well for sliders with large step sizes
  Relative,
}

/// A slider element that allows selecting a value in a range
#[derive(Setters)]
#[setters(prefix = "with_")]
pub struct Slider {
  /// Value of the slider, should be in range 0..1
  ///
  /// Out of range values will be clamped
  pub value: f32,

  /// Size of the element
  #[setters(into)]
  pub size: Size2d,

  /// Track frame
  pub track: Box<dyn Frame>,

  /// Track active frame
  pub track_active: Box<dyn Frame>,

  /// Handle frame
  pub handle: Box<dyn Frame>,

  /// Track height relative to the slider height\
  ///
  /// Range: 0.0..=1.0
  pub track_height_ratio: f32,

  /// Handle width in pixels
  pub handle_width: f32,

  /// Follow mode
  pub follow_mode: SliderFollowMode,

  #[setters(skip)]
  pub on_change: Option<SignalTriggerArg<f32>>,
}

impl Default for Slider {
  fn default() -> Self {
    Self {
      value: 0.0,
      size: Size2d::default(),
      handle: Box::new(FrameRect::color((0.0, 0.0, 1.))),
      track: Box::new(FrameRect::color((0.5, 0.5, 0.5))),
      track_active: Box::new(FrameRect::color((0.0, 0.0, 0.75))),
      track_height_ratio: 0.25,
      handle_width: 15.0,
      follow_mode: SliderFollowMode::default(),
      on_change: None
    }
  }
}

impl Slider {
  pub const DEFAULT_HEIGHT: f32 = 20.0;

  pub fn new(value: f32) -> Self {
    Self {
      value,
      ..Default::default()
    }
  }

  pub fn on_change<S: Signal, T: Fn(f32) -> S + 'static>(self, f: T) -> Self {
    Self {
      on_change: Some(SignalTriggerArg::new(f)),
      ..self
    }
  }
}

impl UiElement for Slider {
  fn name(&self) -> &'static str {
    "slider"
  }

  fn measure(&self, ctx: MeasureContext) -> Response {
    Response {
      size: compute_size(ctx.layout, self.size, (ctx.layout.max_size.x, Self::DEFAULT_HEIGHT).into()),
      ..Default::default()
    }
  }

  fn process(&self, ctx: ProcessContext) {
    //XXX: some of these assumptions are wrong if the  corners are rounded

    //Compute handle size:
    // This is kinda counter-intuitive, but if the handle is transparent, we treat it as completely disabled
    // To prevent confusing offset from the edge of the slider, we set the handle size to 0
    // let handle_size = if self.handle_color.is_transparent() {
    //   Vec2::ZERO
    // } else {
    //   vec2(15., ctx.measure.size.y)
    // };
    let handle_size = vec2(self.handle_width, ctx.measure.size.y);

    //Draw the track
    //If the active part is opaque and value >= 1., we don't need to draw the background as the active part will cover it
    //However, if the handle is not opaque, we need to draw the background as the active part won't quite reach the end
    //Of corse, if it's fully transparent, we don't need to draw it either
    // if !(self.track_color.is_transparent() || (self.track_active_color.is_opaque() && self.handle_color.is_opaque() && self.value >= 1.)) {
    if !(self.track_active.covers_opaque() && self.handle.covers_opaque() && self.value >= 1.) {
      self.track.draw(
        ctx.draw,
        ctx.layout.position + ctx.measure.size * vec2(0., 0.5 - self.track_height_ratio / 2.),
        ctx.measure.size * vec2(1., self.track_height_ratio),
      );
    }

    //"Active" part of the track
    //We can skip drawing it if it's fully transparent or value <= 0.
    //But if the handle is not opaque, it should be visible even if value is zero
    // if !(self.track_active_color.is_transparent() || (self.value <= 0. && self.handle_color.is_opaque())) {
    if !(self.handle.covers_opaque() && self.value <= 0.) {
      self.track_active.draw(
        ctx.draw,
        ctx.layout.position + ctx.measure.size * vec2(0., 0.5 - self.track_height_ratio / 2.),
        (ctx.measure.size - handle_size * Vec2::X) * vec2(self.value, self.track_height_ratio) + handle_size * Vec2::X / 2.,
      );
    }

    // The handle
    // if handle_size.x != 0. && !self.handle_color.is_transparent() {
    //   let value = self.value.clamp(0., 1.);
    //   ctx.draw.add(UiDrawCommand::Rectangle {
    //     position: ctx.layout.position + ((ctx.measure.size.x - handle_size.x) * value) * Vec2::X,
    //     size: handle_size,
    //     color: self.handle_color.into(),
    //     texture: None,
    //     rounded_corners: None,
    //   });
    // }
    self.handle.draw(
      ctx.draw,
      ctx.layout.position + ((ctx.measure.size.x - handle_size.x) * self.value) * Vec2::X,
      handle_size,
    );

    //handle events
    if let Some(res) = ctx.input.check_active(ctx.measure.rect(ctx.layout.position)) {
      let new_value = match self.follow_mode {
        SliderFollowMode::Absolute => {
          ((res.position_in_rect.x - handle_size.x / 2.) / (ctx.measure.size.x - handle_size.x)).clamp(0., 1.)
        },
        SliderFollowMode::Relative => {
          let delta = res.position_in_rect.x - res.last_position_in_rect.x;
          let delta_ratio = delta / (ctx.measure.size.x - handle_size.x);
          (self.value + delta_ratio).clamp(0., 1.)
        }
      };
      if let Some(signal) = &self.on_change {
        signal.fire(ctx.signal, new_value);
      }
      //TODO call signal with new value
    }
  }
}
