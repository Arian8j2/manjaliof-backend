use super::Runner;
use tokio::process::Command;
use std::ffi::OsStr;

pub struct Manjaliof { }

impl Manjaliof {
    pub fn new() -> Self {
        Manjaliof { }
    }

    async fn run_command<S, T>(&self, args: T) -> Result<String, String>
    where
        T: IntoIterator<Item = S>,
        S: AsRef<OsStr>
    {
        let output = Command::new("manjaliof").args(args).output().await.map_err(|e| e.to_string())?;
        if output.status.success() {
            let stdout_output = String::from_utf8(output.stdout).unwrap();
            Ok(stdout_output)
        } else {
            let stderr_output = String::from_utf8(output.stderr).unwrap();
            Err(stderr_output)
        }
    }
}

#[async_trait]
impl Runner for Manjaliof {
    async fn validate_clients(&self, names: &Vec<String>) -> Result<(), String> {
        let mut valid_clients = 0;

        let list = self.run_command(&["list", "--trim-whitespace"]).await?;
        for line in list.lines() {
            let chunks: Vec<&str> = line.split(" ").collect();
            let name = chunks.get(0).unwrap();
            let info = chunks.get(3).unwrap();

            if names.contains(&name.to_string()) {
                if !info.starts_with("NOTPAID") {
                    return Err(format!("client '{name}' is not notpaid, it's '{info}'"));
                }

                valid_clients += 1;
            }
        }

        if valid_clients != names.len() {
            Err("cannot find clients".to_string())
        } else {
            Ok(())
        }
    }

    async fn make_client_paid(&self, name: &str) -> Result<(), String> {
        self.run_command(&["set-info", "--name", name, "--info", "PAID"]).await?;
        Ok(())
    }
}
