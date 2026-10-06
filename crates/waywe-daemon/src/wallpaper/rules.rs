use crate::{event_loop::WallpaperTarget, wallpaper::transition::PauseState};
use waywe_ipc::command::PauseMode;
use waywe_runtime::platform::{MonitorMap, MonitorName};

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct PauseRules {
    for_all: PauseState,
    per_monitor: MonitorMap<PauseState>,
}

impl PauseRules {
    pub fn toggle_monitor(&mut self, name: MonitorName, mode: PauseMode) {
        self.per_monitor
            .entry(name)
            .and_modify(|s| *s = s.toggled(mode))
            .or_insert(self.for_all.toggled(mode));
    }

    pub fn toggle_all(&mut self, mode: PauseMode) {
        self.for_all = self.for_all.toggled(mode);

        match mode {
            PauseMode::Toggle => {
                for state in self.per_monitor.values_mut() {
                    *state = state.toggled(mode);
                }
            }
            PauseMode::On | PauseMode::Off => {
                self.per_monitor.clear();
            }
        }
    }

    pub fn toggle(&mut self, target: WallpaperTarget, mode: PauseMode) {
        match target {
            WallpaperTarget::ForAll => self.toggle_all(mode),
            WallpaperTarget::ForMonitor(name) => self.toggle_monitor(name, mode),
        }
    }

    pub fn get(&self, name: &str) -> PauseState {
        self.per_monitor.get(name).copied().unwrap_or(self.for_all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use PauseMode::*;
    use PauseState::*;

    fn name(s: &str) -> MonitorName {
        MonitorName::from_str(s)
    }

    #[test]
    fn default_get_falls_back_to_running() {
        let rules = PauseRules::default();
        assert_eq!(rules.get("DP-1"), Running);
    }

    #[test]
    fn toggle_all_modes() {
        let mut rules = PauseRules::default();

        rules.toggle_all(On);
        assert_eq!(rules.get("DP-1"), Paused);

        rules.toggle_all(Off);
        assert_eq!(rules.get("DP-1"), Running);

        rules.toggle_all(Toggle);
        assert_eq!(rules.get("DP-1"), Paused);

        rules.toggle_all(Toggle);
        assert_eq!(rules.get("DP-1"), Running);
    }

    #[test]
    fn first_toggle_monitor_seeds_from_for_all() {
        let mut rules = PauseRules::default();
        rules.toggle_all(On);

        rules.toggle_monitor(name("DP-1"), Toggle);

        assert_eq!(rules.get("DP-1"), Running);
        assert_eq!(rules.get("HDMI-1"), Paused);
    }

    #[test]
    fn toggle_monitor_updates_existing_entry() {
        let mut rules = PauseRules::default();

        rules.toggle_monitor(name("DP-1"), On);
        assert_eq!(rules.get("DP-1"), Paused);
        assert_eq!(rules.get("HDMI-1"), Running);

        rules.toggle_monitor(name("DP-1"), Toggle);
        assert_eq!(rules.get("DP-1"), Running);
        assert_eq!(rules.get("HDMI-1"), Running);
    }

    #[test]
    fn toggle_all_updates_overrides_and_unlisted() {
        let mut rules = PauseRules::default();

        rules.toggle_monitor(name("DP-1"), On);
        assert_eq!(rules.get("DP-1"), Paused);
        assert_eq!(rules.get("HDMI-1"), Running);

        rules.toggle_all(On);
        assert_eq!(rules.get("DP-1"), Paused);
        assert_eq!(rules.get("HDMI-1"), Paused);
    }

    #[test]
    fn toggle_dispatches_by_target() {
        let mut rules = PauseRules::default();

        rules.toggle(WallpaperTarget::ForAll, On);
        assert_eq!(rules.get("DP-1"), Paused);

        rules.toggle(WallpaperTarget::ForMonitor(name("DP-1")), Toggle);
        assert_eq!(rules.get("DP-1"), Running);
        assert_eq!(rules.get("HDMI-1"), Paused);
    }
}
