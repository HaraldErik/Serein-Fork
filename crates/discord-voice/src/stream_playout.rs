//! Keep the independent stream/call audio clocks from accumulating stale queued PCM.
use crate::Frame;
use std::{collections::VecDeque, sync::mpsc::Receiver};

#[derive(Default)]
pub(crate) struct Playout {
	pending: VecDeque<Frame>,
}
impl Playout {
	pub fn next(
		&mut self,
		source: &Receiver<Frame>,
		volume: u16,
		enabled: bool,
		stalled: bool,
	) -> Option<Frame> {
		// ponytail: bound clock drift by trimming PCM; sender-clock A/V sync needs RTCP timing.
		// The source queue holds eight 20 ms frames. Retain at most two: absorb small
		// scheduling bursts without playing the full backlog after a delayed tick.
		for _ in 0..8 {
			let Ok(frame) = source.try_recv() else { break };
			if self.pending.len() == 2 {
				self.pending.pop_front();
			}
			self.pending.push_back(frame);
		}
		if !enabled || stalled || volume == 0 {
			self.pending.clear();
			return None;
		}
		let mut frame = self.pending.pop_front()?;
		let gain = f32::from(volume.min(MAX_PLAYBACK_PERCENT)) / 100.0;
		for sample in &mut frame {
			*sample = if sample.is_finite() {
				soft_limit(*sample * gain)
			} else {
				0.0
			};
		}
		Some(frame)
	}
}

/// Highest local playback volume for a participant or a watched stream (Volume Booster).
pub const MAX_PLAYBACK_PERCENT: u16 = 400;

/// Keeps a mixed sample inside -1..=1. Samples below the knee pass unchanged; louder ones are
/// bent smoothly towards full scale, so boosted voices distort far less than hard clipping.
#[inline]
pub fn soft_limit(sample: f32) -> f32 {
	const KNEE: f32 = 0.8;
	const ROOM: f32 = 1.0 - KNEE;
	let magnitude = sample.abs();
	if magnitude <= KNEE {
		sample
	} else {
		(KNEE + ROOM * ((magnitude - KNEE) / ROOM).tanh()).copysign(sample)
	}
}
