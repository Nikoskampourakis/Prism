use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessibilityPreferences {
    pub text_scale_percent: u8,
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub announce_vm_status: bool,
}

impl Default for AccessibilityPreferences {
    fn default() -> Self {
        Self {
            text_scale_percent: 100,
            high_contrast: false,
            reduced_motion: false,
            announce_vm_status: true,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AccessibilityError {
    #[error("text scale must be between 80 and 200 percent")]
    InvalidTextScale,
}

impl AccessibilityPreferences {
    pub fn set_text_scale(&mut self, percent: u8) -> Result<(), AccessibilityError> {
        if !(80..=200).contains(&percent) {
            return Err(AccessibilityError::InvalidTextScale);
        }
        self.text_scale_percent = percent;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unusable_text_scale_without_changing_preference() {
        let mut preferences = AccessibilityPreferences::default();
        assert_eq!(
            preferences.set_text_scale(250),
            Err(AccessibilityError::InvalidTextScale)
        );
        assert_eq!(preferences.text_scale_percent, 100);
        preferences.set_text_scale(150).unwrap();
        assert_eq!(preferences.text_scale_percent, 150);
    }
}
