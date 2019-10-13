/// Strongly type the version. `WorkflowVersion::Latest` is translated to `-1`.
pub enum WorkflowVersion {
    Latest,
    Version(i32),
}

impl Into<i32> for WorkflowVersion {
    fn into(self) -> i32 {
        match self {
            WorkflowVersion::Latest => -1,
            WorkflowVersion::Version(v) => v,
        }
    }
}
