use std::time::Instant;

#[cfg(feature = "perf-metrics")]
use std::time::Duration;

#[cfg(feature = "perf-metrics")]
#[derive(Debug)]
struct PerfInner {
    input_event_count: u64,
    action_count: u64,
    draw_count: u64,
    event_draw_count: u64,
    timer_draw_count: u64,
    queue_depth_max: usize,
    event_to_action_total: Duration,
    event_to_draw_total: Duration,
    draw_duration_total: Duration,
    event_draw_duration_total: Duration,
    timer_draw_duration_total: Duration,
    sample_window_started_at: Instant,
}

#[cfg(feature = "perf-metrics")]
impl Default for PerfInner {
    fn default() -> Self {
        Self {
            input_event_count: 0,
            action_count: 0,
            draw_count: 0,
            event_draw_count: 0,
            timer_draw_count: 0,
            queue_depth_max: 0,
            event_to_action_total: Duration::ZERO,
            event_to_draw_total: Duration::ZERO,
            draw_duration_total: Duration::ZERO,
            event_draw_duration_total: Duration::ZERO,
            timer_draw_duration_total: Duration::ZERO,
            sample_window_started_at: Instant::now(),
        }
    }
}

#[derive(Default, Debug)]
pub struct PerfMetrics {
    #[cfg(feature = "perf-metrics")]
    inner: PerfInner,
}

impl PerfMetrics {
    pub fn on_input_event(&mut self) {
        #[cfg(feature = "perf-metrics")]
        {
            self.inner.input_event_count += 1;
        }
    }

    pub fn on_action_from_event(&mut self, _event_started_at: Instant) {
        #[cfg(feature = "perf-metrics")]
        {
            self.inner.action_count += 1;
            self.inner.event_to_action_total += _event_started_at.elapsed();
        }
    }

    pub fn on_draw_started(&self) -> Instant {
        Instant::now()
    }

    pub fn on_draw_completed(
        &mut self,
        _draw_started_at: Instant,
        _last_event_started_at: Option<Instant>,
    ) {
        #[cfg(feature = "perf-metrics")]
        {
            self.inner.draw_count += 1;
            let draw_elapsed = _draw_started_at.elapsed();
            self.inner.draw_duration_total += draw_elapsed;
            if let Some(started_at) = _last_event_started_at {
                self.inner.event_draw_count += 1;
                self.inner.event_draw_duration_total += draw_elapsed;
                self.inner.event_to_draw_total += started_at.elapsed();
            } else {
                self.inner.timer_draw_count += 1;
                self.inner.timer_draw_duration_total += draw_elapsed;
            }
        }
    }

    pub fn on_queue_depth(&mut self, _depth: usize) {
        #[cfg(feature = "perf-metrics")]
        {
            self.inner.queue_depth_max = self.inner.queue_depth_max.max(_depth);
        }
    }

    pub fn maybe_log(&mut self) {
        #[cfg(feature = "perf-metrics")]
        {
            use crate::log_info;

            let elapsed = self.inner.sample_window_started_at.elapsed();
            if elapsed < Duration::from_secs(1) {
                return;
            }

            let action_avg_us = if self.inner.action_count > 0 {
                self.inner.event_to_action_total.as_micros() / self.inner.action_count as u128
            } else {
                0
            };

            let draw_avg_us = if self.inner.draw_count > 0 {
                self.inner.draw_duration_total.as_micros() / self.inner.draw_count as u128
            } else {
                0
            };

            let event_to_draw_avg_us = if self.inner.event_draw_count > 0 {
                self.inner.event_to_draw_total.as_micros() / self.inner.event_draw_count as u128
            } else {
                0
            };

            let event_draw_avg_us = if self.inner.event_draw_count > 0 {
                self.inner.event_draw_duration_total.as_micros()
                    / self.inner.event_draw_count as u128
            } else {
                0
            };

            let timer_draw_avg_us = if self.inner.timer_draw_count > 0 {
                self.inner.timer_draw_duration_total.as_micros()
                    / self.inner.timer_draw_count as u128
            } else {
                0
            };

            log_info!(
                "perf: events={} actions={} draws={} event_draws={} timer_draws={} qmax={} event->action_avg={}us event->draw_avg={}us draw_avg={}us event_draw_avg={}us timer_draw_avg={}us",
                self.inner.input_event_count,
                self.inner.action_count,
                self.inner.draw_count,
                self.inner.event_draw_count,
                self.inner.timer_draw_count,
                self.inner.queue_depth_max,
                action_avg_us,
                event_to_draw_avg_us,
                draw_avg_us,
                event_draw_avg_us,
                timer_draw_avg_us
            );

            self.inner = PerfInner {
                sample_window_started_at: Instant::now(),
                ..PerfInner::default()
            };
        }
    }
}
