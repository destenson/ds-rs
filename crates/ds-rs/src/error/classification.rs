use crate::error::DeepStreamError;
use std::collections::HashMap;

/// Severity level of an error
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational, can be ignored
    Info = 0,
    /// Warning, should be monitored
    Warning = 1,
    /// Error that can be recovered from
    Recoverable = 2,
    /// Critical error, may require intervention
    Critical = 3,
    /// Fatal error, cannot continue
    Fatal = 4,
}

/// Classification of error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Network-related errors (connection, timeout)
    Network,
    /// Media decoding/encoding errors
    Codec,
    /// Pipeline state or configuration errors
    Pipeline,
    /// Resource errors (memory, file access)
    Resource,
    /// Hardware or driver errors
    Hardware,
    /// Unknown or unclassified errors
    Unknown,
}

/// Determines if an error is transient or permanent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorPersistence {
    /// Error is temporary and may resolve on retry
    Transient,
    /// Error is permanent and won't resolve on retry
    Permanent,
}

/// Recommended action for error recovery
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryAction {
    /// Retry immediately
    RetryNow,
    /// Retry with backoff
    RetryWithBackoff { initial_delay_ms: u64 },
    /// Reconnect the source
    Reconnect,
    /// Reset the pipeline element
    ResetElement { element_name: String },
    /// Restart the entire pipeline
    RestartPipeline,
    /// Mark source as failed, continue with others
    FailSource,
    /// No recovery possible
    NoRecovery,
}

/// Complete error classification
#[derive(Debug, Clone)]
pub struct ErrorClassification {
    pub severity: ErrorSeverity,
    pub category: ErrorCategory,
    pub persistence: ErrorPersistence,
    pub action: RecoveryAction,
    pub description: String,
}

/// Error classifier that maps errors to classifications
pub struct ErrorClassifier {
    patterns: HashMap<String, ErrorClassification>,
}

impl ErrorClassifier {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Network errors
        patterns.insert(
            "connection refused".to_string(),
            ErrorClassification {
                severity: ErrorSeverity::Recoverable,
                category: ErrorCategory::Network,
                persistence: ErrorPersistence::Transient,
                action: RecoveryAction::RetryWithBackoff {
                    initial_delay_ms: 1000,
                },
                description: "Network connection refused".to_string(),
            },
        );

        patterns.insert(
            "timeout".to_string(),
            ErrorClassification {
                severity: ErrorSeverity::Recoverable,
                category: ErrorCategory::Network,
                persistence: ErrorPersistence::Transient,
                action: RecoveryAction::RetryWithBackoff {
                    initial_delay_ms: 500,
                },
                description: "Network timeout".to_string(),
            },
        );

        patterns.insert(
            "host not found".to_string(),
            ErrorClassification {
                severity: ErrorSeverity::Critical,
                category: ErrorCategory::Network,
                persistence: ErrorPersistence::Permanent,
                action: RecoveryAction::NoRecovery,
                description: "Host not found".to_string(),
            },
        );

        // RTSP specific
        patterns.insert(
            "rtsp".to_string(),
            ErrorClassification {
                severity: ErrorSeverity::Recoverable,
                category: ErrorCategory::Network,
                persistence: ErrorPersistence::Transient,
                action: RecoveryAction::Reconnect,
                description: "RTSP stream error".to_string(),
            },
        );

        // Codec errors
        patterns.insert(
            "decoder".to_string(),
            ErrorClassification {
                severity: ErrorSeverity::Recoverable,
                category: ErrorCategory::Codec,
                persistence: ErrorPersistence::Transient,
                action: RecoveryAction::ResetElement {
                    element_name: "decoder".to_string(),
                },
                description: "Decoder error".to_string(),
            },
        );

        patterns.insert(
            "not-negotiated".to_string(),
            ErrorClassification {
                severity: ErrorSeverity::Critical,
                category: ErrorCategory::Codec,
                persistence: ErrorPersistence::Permanent,
                action: RecoveryAction::RestartPipeline,
                description: "Caps negotiation failed".to_string(),
            },
        );

        // Resource errors
        patterns.insert(
            "file not found".to_string(),
            ErrorClassification {
                severity: ErrorSeverity::Critical,
                category: ErrorCategory::Resource,
                persistence: ErrorPersistence::Permanent,
                action: RecoveryAction::FailSource,
                description: "File not found".to_string(),
            },
        );

        patterns.insert(
            "out of memory".to_string(),
            ErrorClassification {
                severity: ErrorSeverity::Fatal,
                category: ErrorCategory::Resource,
                persistence: ErrorPersistence::Permanent,
                action: RecoveryAction::NoRecovery,
                description: "Out of memory".to_string(),
            },
        );

        // Pipeline errors
        patterns.insert(
            "state change".to_string(),
            ErrorClassification {
                severity: ErrorSeverity::Recoverable,
                category: ErrorCategory::Pipeline,
                persistence: ErrorPersistence::Transient,
                action: RecoveryAction::RetryWithBackoff {
                    initial_delay_ms: 100,
                },
                description: "Pipeline state change error".to_string(),
            },
        );

        patterns.insert(
            "pad linking".to_string(),
            ErrorClassification {
                severity: ErrorSeverity::Critical,
                category: ErrorCategory::Pipeline,
                persistence: ErrorPersistence::Permanent,
                action: RecoveryAction::RestartPipeline,
                description: "Pad linking failed".to_string(),
            },
        );

        Self { patterns }
    }

    /// Classify an error based on its message
    pub fn classify_error(&self, error: &DeepStreamError) -> ErrorClassification {
        let error_str = error.to_string().to_lowercase();

        // Check for pattern matches
        for (pattern, classification) in &self.patterns {
            if error_str.contains(pattern) {
                return classification.clone();
            }
        }

        // Default classification based on error type
        match error {
            DeepStreamError::GStreamer(_) | DeepStreamError::GStreamerBool(_) => {
                ErrorClassification {
                    severity: ErrorSeverity::Recoverable,
                    category: ErrorCategory::Pipeline,
                    persistence: ErrorPersistence::Transient,
                    action: RecoveryAction::RetryWithBackoff {
                        initial_delay_ms: 500,
                    },
                    description: "GStreamer error".to_string(),
                }
            }
            DeepStreamError::StateChange(_) => ErrorClassification {
                severity: ErrorSeverity::Recoverable,
                category: ErrorCategory::Pipeline,
                persistence: ErrorPersistence::Transient,
                action: RecoveryAction::RetryWithBackoff {
                    initial_delay_ms: 200,
                },
                description: "State change error".to_string(),
            },
            DeepStreamError::PadLinking(_) | DeepStreamError::PadNotFound { .. } => {
                ErrorClassification {
                    severity: ErrorSeverity::Critical,
                    category: ErrorCategory::Pipeline,
                    persistence: ErrorPersistence::Permanent,
                    action: RecoveryAction::RestartPipeline,
                    description: "Pad error".to_string(),
                }
            }
            DeepStreamError::Timeout(_) => ErrorClassification {
                severity: ErrorSeverity::Recoverable,
                category: ErrorCategory::Network,
                persistence: ErrorPersistence::Transient,
                action: RecoveryAction::RetryWithBackoff {
                    initial_delay_ms: 1000,
                },
                description: "Timeout error".to_string(),
            },
            DeepStreamError::Io(_) => ErrorClassification {
                severity: ErrorSeverity::Recoverable,
                category: ErrorCategory::Resource,
                persistence: ErrorPersistence::Transient,
                action: RecoveryAction::RetryWithBackoff {
                    initial_delay_ms: 500,
                },
                description: "IO error".to_string(),
            },
            _ => ErrorClassification {
                severity: ErrorSeverity::Warning,
                category: ErrorCategory::Unknown,
                persistence: ErrorPersistence::Transient,
                action: RecoveryAction::RetryWithBackoff {
                    initial_delay_ms: 1000,
                },
                description: "Unknown error".to_string(),
            },
        }
    }

    /// Add a custom error pattern
    pub fn add_pattern(&mut self, pattern: String, classification: ErrorClassification) {
        self.patterns.insert(pattern, classification);
    }

    /// Check if an error is retryable
    pub fn is_retryable(&self, error: &DeepStreamError) -> bool {
        let classification = self.classify_error(error);
        matches!(
            classification.action,
            RecoveryAction::RetryNow
                | RecoveryAction::RetryWithBackoff { .. }
                | RecoveryAction::Reconnect
        )
    }

    /// Get recommended delay before retry
    pub fn get_retry_delay(&self, error: &DeepStreamError) -> Option<std::time::Duration> {
        let classification = self.classify_error(error);
        match classification.action {
            RecoveryAction::RetryNow => Some(std::time::Duration::from_millis(0)),
            RecoveryAction::RetryWithBackoff { initial_delay_ms } => {
                Some(std::time::Duration::from_millis(initial_delay_ms))
            }
            RecoveryAction::Reconnect => Some(std::time::Duration::from_secs(1)),
            _ => None,
        }
    }
}

// TODO: GET RID OF THIS GLOBAL & dependency on lazy_static
// Global error classifier instance
lazy_static::lazy_static! {
    static ref GLOBAL_CLASSIFIER: ErrorClassifier = ErrorClassifier::new();
}

/// Classify an error using the global classifier
pub fn classify(error: &DeepStreamError) -> ErrorClassification {
    GLOBAL_CLASSIFIER.classify_error(error)
}

/// Check if an error is retryable using the global classifier
pub fn is_retryable(error: &DeepStreamError) -> bool {
    GLOBAL_CLASSIFIER.is_retryable(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_error_classification() {
        let classifier = ErrorClassifier::new();

        let error = DeepStreamError::Unknown("Connection refused".to_string());
        let classification = classifier.classify_error(&error);

        assert_eq!(classification.category, ErrorCategory::Network);
        assert_eq!(classification.persistence, ErrorPersistence::Transient);
        assert!(matches!(
            classification.action,
            RecoveryAction::RetryWithBackoff { .. }
        ));
    }

    #[test]
    fn test_timeout_error_classification() {
        let classifier = ErrorClassifier::new();

        let error = DeepStreamError::Timeout("Request timeout".to_string());
        let classification = classifier.classify_error(&error);

        assert_eq!(classification.category, ErrorCategory::Network);
        assert_eq!(classification.persistence, ErrorPersistence::Transient);
        assert_eq!(classification.severity, ErrorSeverity::Recoverable);
    }

    #[test]
    fn test_permanent_error_classification() {
        let classifier = ErrorClassifier::new();

        let error = DeepStreamError::Unknown("File not found".to_string());
        let classification = classifier.classify_error(&error);

        assert_eq!(classification.category, ErrorCategory::Resource);
        assert_eq!(classification.persistence, ErrorPersistence::Permanent);
        assert_eq!(classification.action, RecoveryAction::FailSource);
    }

    #[test]
    fn test_is_retryable() {
        let classifier = ErrorClassifier::new();

        let retryable_error = DeepStreamError::Timeout("Timeout".to_string());
        assert!(classifier.is_retryable(&retryable_error));

        let permanent_error = DeepStreamError::Unknown("Out of memory".to_string());
        assert!(!classifier.is_retryable(&permanent_error));
    }

    #[test]
    fn test_retry_delay() {
        let classifier = ErrorClassifier::new();

        let error = DeepStreamError::Unknown("Connection refused".to_string());
        let delay = classifier.get_retry_delay(&error);

        assert!(delay.is_some());
        assert_eq!(delay.unwrap(), std::time::Duration::from_millis(1000));
    }

    #[test]
    fn test_custom_pattern() {
        let mut classifier = ErrorClassifier::new();

        classifier.add_pattern(
            "custom error".to_string(),
            ErrorClassification {
                severity: ErrorSeverity::Fatal,
                category: ErrorCategory::Unknown,
                persistence: ErrorPersistence::Permanent,
                action: RecoveryAction::NoRecovery,
                description: "Custom fatal error".to_string(),
            },
        );

        let error = DeepStreamError::Unknown("This is a custom error".to_string());
        let classification = classifier.classify_error(&error);

        assert_eq!(classification.severity, ErrorSeverity::Fatal);
        assert_eq!(classification.action, RecoveryAction::NoRecovery);
    }
}
