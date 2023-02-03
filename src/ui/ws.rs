use std::fs;
use std::error::Error;
use std::fmt::Display;
use std::path::{PathBuf, Path};

#[derive(Debug, Clone)]
pub enum WorkspaceError {
    ParentError,
    SubdirectoryError,
    RawFileError(i32)
}

impl Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceError::ParentError => write!(f, "Parent directory given to workspace does not exist and could not be created!"),
            WorkspaceError::SubdirectoryError => write!(f, "A required subdirectory in workspace does not exist and could not be created!"),
            WorkspaceError::RawFileError(run_number) => write!(f, "Run file number {} could not be found in raw binary directory!", run_number)
        }
    }
}

impl Error for WorkspaceError {

}

#[derive(Debug, Clone)]
pub struct Workspace {
    parent_dir: PathBuf
}

impl Workspace {

    pub fn new(parent: &Path) -> Result<Self, WorkspaceError> {
        if parent.exists() && parent.is_dir() {
            let ws = Workspace{ parent_dir: parent.to_path_buf() };
            ws.init_workspace()?;
            return Ok(Workspace { parent_dir: parent.to_path_buf() });
        }
        else if !parent.exists() {
            match fs::create_dir_all(&parent) {
                Ok(_) => return {
                    let ws = Workspace{ parent_dir: parent.to_path_buf() };
                    ws.init_workspace()?;
                    Ok(Workspace { parent_dir: parent.to_path_buf() })
                },
                Err(_) => return Err(WorkspaceError::ParentError)
            };
        }
        Err(WorkspaceError::ParentError)
    }

    pub fn get_parent_str(&self) -> &str {
        match self.parent_dir.as_path().to_str() {
            Some(path) => path,
            None => "InvalidParent"
        }
    }

    pub fn get_raw_binary_file(&self, run_number: &i32) -> Result<PathBuf, WorkspaceError> {
        let run_file = self.parent_dir.join("raw_binary").join(format!("run_{}.tar.gz", run_number));
        if run_file.exists() {
            Ok(run_file)
        } else {
            Err(WorkspaceError::RawFileError(*run_number))
        }
    }

    pub fn get_temp_binary_dir(&self) -> Result<PathBuf, WorkspaceError> {
        let temp_binary = self.parent_dir.join("temp_binary");
        if temp_binary.exists() {
            Ok(temp_binary)
        } else {
            Err(WorkspaceError::SubdirectoryError)
        }
    }

    pub fn get_built_file(&self, run_number: &i32) -> Result<PathBuf, WorkspaceError> {
        let mut built_file = self.parent_dir.join("built");
        if built_file.exists() {
            built_file.push(format!("run_{}.parquet", run_number));
            Ok(built_file)
        } else {
            Err(WorkspaceError::SubdirectoryError)
        }
    }

    fn init_workspace(&self) -> Result<(), WorkspaceError> {
        let raw_binary = self.parent_dir.join("raw_binary");
        let temp_binary = self.parent_dir.join("temp_binary");
        let built = self.parent_dir.join("built");

        if !raw_binary.exists() {
            match fs::create_dir(&raw_binary) {
                Ok(_) => (),
                Err(_) => return Err(WorkspaceError::SubdirectoryError)
            };
        }

        if !temp_binary.exists() {
            match fs::create_dir(&temp_binary) {
                Ok(_) => (),
                Err(_) => return Err(WorkspaceError::SubdirectoryError)
            };
        }

        if !built.exists() {
            match fs::create_dir(&built) {
                Ok(_) => (),
                Err(_) => return Err(WorkspaceError::SubdirectoryError)
            };
        }

        return Ok(());
    }
}



