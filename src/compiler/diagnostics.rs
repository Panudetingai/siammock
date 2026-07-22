use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    #[allow(dead_code)]
    Info,
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn error(
        code: impl Into<String>,
        path: impl Into<String>,
        line: usize,
        column: usize,
        message: impl Into<String>,
        hint: Option<String>,
    ) -> Self {
        Self {
            severity: Severity::Error,
            code: code.into(),
            path: path.into(),
            line,
            column,
            message: message.into(),
            hint,
        }
    }

    pub fn warning(
        code: impl Into<String>,
        path: impl Into<String>,
        line: usize,
        column: usize,
        message: impl Into<String>,
        hint: Option<String>,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            code: code.into(),
            path: path.into(),
            line,
            column,
            message: message.into(),
            hint,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CompileResult {
    pub valid: bool,
    pub diagnostics: Vec<Diagnostic>,
}

impl CompileResult {
    pub fn merge(mut self, other: CompileResult) -> Self {
        self.diagnostics.extend(other.diagnostics);
        self.valid = self.valid && other.valid;
        self
    }
}
