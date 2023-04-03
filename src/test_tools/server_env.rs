pub struct ServerEnv {}

impl ServerEnv {}

impl Drop for ServerEnv {
    /// Deletes the file when the FileEnv is dropped
    fn drop(&mut self) {}
}
