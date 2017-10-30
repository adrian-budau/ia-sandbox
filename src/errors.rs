use std::result::Result as StdResult;

impl_child_error! {
    #[derive(Serialize, Deserialize, Debug)]
    pub enum ChildError {
        ChdirError(String),
        ChrootError(String),
        CreateDirError(String),
        ExecError(String),
        MountError {
            path: String,
            error: String,
        },
        PivotRootError(String),
        WriteUidError(String),
        WriteGidError(String),
        Custom(String),
    }
}

pub type ChildResult<T> = StdResult<T, ChildError>;

error_chain! {
    foreign_links {
        ChildError(ChildError);
    }
}
