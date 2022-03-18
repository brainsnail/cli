use std::io::Write;

use anyhow::{anyhow, Result};
use clap::Parser;
use cli_macros::crud_gen;

/// Create, list, edit, view, and delete instances.
///
/// Additionally, start, stop, and reboot instances.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstance {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[crud_gen {
    tag = "instances",
}]
#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Create(CmdInstanceCreate),
    Disks(CmdInstanceDisks),
    Edit(CmdInstanceEdit),
    Start(CmdInstanceStart),
    Stop(CmdInstanceStop),
    Reboot(CmdInstanceReboot),
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstance {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        match &self.subcmd {
            SubCommand::Create(cmd) => cmd.run(ctx).await,
            SubCommand::Delete(cmd) => cmd.run(ctx).await,
            SubCommand::Disks(cmd) => cmd.run(ctx).await,
            SubCommand::Edit(cmd) => cmd.run(ctx).await,
            SubCommand::List(cmd) => cmd.run(ctx).await,
            SubCommand::Start(cmd) => cmd.run(ctx).await,
            SubCommand::Stop(cmd) => cmd.run(ctx).await,
            SubCommand::Reboot(cmd) => cmd.run(ctx).await,
            SubCommand::View(cmd) => cmd.run(ctx).await,
        }
    }
}

/// Create a new instance.
///
/// To create an instance interactively, use `oxide instance create` with no arguments.
/// The instance name will be used as the hostname if you do not explicitly specify one.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceCreate {
    /// The name of the instance to create.
    #[clap(name = "instance", default_value = "")]
    pub instance: String,

    /// The project that will hold the instance.
    #[clap(long, short, default_value = "")]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, default_value = "", env = "OXIDE_ORG")]
    pub organization: String,

    /// The description for the instance.
    #[clap(long = "description", short = 'D', default_value = "")]
    pub description: String,

    /// The hostname for the instance.
    #[clap(long = "hostname", short = 'H', default_value = "")]
    pub hostname: String,

    // TODO: handle human-like input for sizes.
    /// The memory to allocate for the instance, in bytes.
    #[clap(long, short, default_value = "0")]
    pub memory: i64,

    /// The number of CPUs to allocate for the instance.
    #[clap(long, short, default_value = "0")]
    pub cpus: i64,
}

// TODO: in interactive create it should list the projects from the user's org as a select.
#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceCreate {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let mut instance_name = self.instance.to_string();
        let mut project_name = self.project.to_string();
        let mut description = self.description.to_string();
        let mut organization = self.organization.to_string();

        let mut ncpus = self.cpus;
        let mut memory = self.memory;
        let mut hostname = self.hostname.to_string();

        if instance_name.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("[instance_name] required in non-interactive mode"));
        }

        if organization.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("--organization,-o required in non-interactive mode"));
        }

        if project_name.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("--project,-p required in non-interactive mode"));
        }

        if description.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("--description,-D required in non-interactive mode"));
        }

        if ncpus == 0 && !ctx.io.can_prompt() {
            return Err(anyhow!("--cpus,-c required in non-interactive mode"));
        }

        if memory == 0 && !ctx.io.can_prompt() {
            return Err(anyhow!("--memory,m required in non-interactive mode"));
        }

        // If they didn't specify an organization, prompt for it.
        if organization.is_empty() {
            match dialoguer::Input::<String>::new()
                .with_prompt("Project organization:")
                .interact_text()
            {
                Ok(org) => organization = org,
                Err(err) => {
                    return Err(anyhow!("prompt failed: {}", err));
                }
            }
        }

        let client = ctx.api_client("")?;

        if project_name.is_empty() {
            let mut org_projects: Vec<String> = Vec::new();
            let projects = client
                .projects()
                .get_all(&organization, oxide_api::types::NameOrIdSortMode::NameAscending)
                .await?;
            for project in projects {
                org_projects.push(project.name.to_string());
            }

            // Select the project from the selected organization.
            match dialoguer::Select::new()
                .with_prompt("Select project:")
                .items(&org_projects)
                .interact()
            {
                Ok(index) => project_name = org_projects[index].to_string(),
                Err(err) => {
                    return Err(anyhow!("prompt failed: {}", err));
                }
            }
        }

        // Prompt for the instance name.
        if instance_name.is_empty() {
            match dialoguer::Input::<String>::new()
                .with_prompt("Instance name:")
                .interact_text()
            {
                Ok(name) => instance_name = name,
                Err(err) => {
                    return Err(anyhow!("prompt failed: {}", err));
                }
            }

            // Propmt for a description if they didn't provide one.
            if description.is_empty() {
                match dialoguer::Input::<String>::new()
                    .with_prompt("Instance description:")
                    .interact_text()
                {
                    Ok(desc) => description = desc,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }

            if hostname.is_empty() {
                match dialoguer::Input::<String>::new()
                    .with_prompt("Instance hostname:")
                    .interact_text()
                {
                    Ok(name) => hostname = name,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }

            if memory == 0 {
                // TODO: make this a select.
                match dialoguer::Input::<i64>::new()
                    .with_prompt("Instance memory:")
                    .interact_text()
                {
                    Ok(m) => memory = m,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }

            if ncpus == 0 {
                // TODO: make this a select.
                match dialoguer::Input::<i64>::new()
                    .with_prompt("Instance CPUs:")
                    .interact_text()
                {
                    Ok(m) => ncpus = m,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }
        }

        let full_name = format!("{}/{}", organization, project_name);

        if hostname.is_empty() {
            // Set it to the instance name.
            hostname = instance_name.to_string();
        }

        // Create the disk.
        client
            .instances()
            .post(
                &organization,
                &project_name,
                &oxide_api::types::InstanceCreate {
                    name: instance_name.to_string(),
                    description: description.to_string(),
                    hostname: hostname.to_string(),
                    memory,
                    ncpus,
                    network_interfaces: Default::default(),
                },
            )
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Successfully created instance {} in {}",
            cs.success_icon(),
            instance_name,
            full_name
        )?;

        Ok(())
    }
}

/// List the disks attached to an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceDisks {
    /// The instance to view the disks for.
    #[clap(name = "instance", required = true)]
    pub instance: String,

    /// The project that holds the instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization to view the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Open a project in the browser.
    #[clap(short, long)]
    pub web: bool,

    /// Output JSON.
    #[clap(long)]
    pub json: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceDisks {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let client = ctx.api_client("")?;

        let disks = client
            .instances()
            .disks_get_all(
                &self.instance,
                &self.organization,
                &self.project,
                oxide_api::types::NameSortMode::NameAscending,
            )
            .await?;

        if self.json {
            // If they specified --json, just dump the JSON.
            ctx.io.write_json(&serde_json::json!(disks))?;
            return Ok(());
        }

        let table = tabled::Table::new(disks).with(tabled::Style::psql()).to_string();
        writeln!(ctx.io.out, "{}", table)?;

        Ok(())
    }
}

/// Edit instance settings.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceEdit {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceEdit {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet in omicron.");
        Ok(())
    }
}

/// Start an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceStart {
    /// The instance to start. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    instance: String,

    /// The project that holds the instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceStart {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Start the instance.
        // TODO: Do we want a progress bar here?
        client
            .instances()
            .start(&self.instance, &self.organization, &self.project)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Started instance {} in {}",
            cs.success_icon(),
            self.instance,
            full_name
        )?;

        Ok(())
    }
}

/// Stop an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceStop {
    /// The instance to stop. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    instance: String,

    /// The project that holds the instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Confirm stop without prompting.
    #[clap(long)]
    pub confirm: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceStop {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow!("--confirm required when not running interactively"));
        }

        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Confirm stop.
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm stop:", self.instance))
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.trim() == full_name {
                        Ok(())
                    } else {
                        Err("mismatched confirmation")
                    }
                })
                .interact_text()
            {
                return Err(anyhow!("prompt failed: {}", err));
            }
        }

        // Stop the instance.
        // TODO: Do we want a progress bar here?
        client
            .instances()
            .stop(&self.instance, &self.organization, &self.project)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Stopped instance {} in {}",
            cs.failure_icon_with_color(ansi_term::Color::Green),
            self.instance,
            full_name
        )?;

        Ok(())
    }
}

/// Reboot an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceReboot {
    /// The instance to reboot. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    instance: String,

    /// The project that holds the instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Confirm reboot without prompting.
    #[clap(long)]
    pub confirm: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceReboot {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow!("--confirm required when not running interactively"));
        }

        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Confirm reboot.
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm reboot:", self.instance))
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.trim() == full_name {
                        Ok(())
                    } else {
                        Err("mismatched confirmation")
                    }
                })
                .interact_text()
            {
                return Err(anyhow!("prompt failed: {}", err));
            }
        }

        // Reboot the instance.
        // TODO: Do we want a progress bar here?
        client
            .instances()
            .reboot(&self.instance, &self.organization, &self.project)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Rebooted instance {} in {}",
            cs.success_icon(),
            self.instance,
            full_name
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::cmd::Command;

    pub struct TestItem {
        name: String,
        cmd: crate::cmd_instance::SubCommand,
        stdin: String,
        want_out: String,
        want_err: String,
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_cmd_instance() {
        let tests: Vec<TestItem> = vec![
            TestItem {
                name: "create no name".to_string(),
                cmd: crate::cmd_instance::SubCommand::Create(crate::cmd_instance::CmdInstanceCreate {
                    instance: "".to_string(),
                    organization: "".to_string(),
                    project: "".to_string(),
                    description: "".to_string(),
                    memory: 0,
                    cpus: 0,
                    hostname: "".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "[instance_name] required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no organization".to_string(),
                cmd: crate::cmd_instance::SubCommand::Create(crate::cmd_instance::CmdInstanceCreate {
                    instance: "things".to_string(),
                    organization: "".to_string(),
                    project: "".to_string(),
                    description: "".to_string(),
                    memory: 0,
                    cpus: 0,
                    hostname: "".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--organization,-o required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no project".to_string(),
                cmd: crate::cmd_instance::SubCommand::Create(crate::cmd_instance::CmdInstanceCreate {
                    instance: "things".to_string(),
                    organization: "foo".to_string(),
                    project: "".to_string(),
                    description: "".to_string(),
                    memory: 0,
                    cpus: 0,
                    hostname: "".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--project,-p required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no description".to_string(),
                cmd: crate::cmd_instance::SubCommand::Create(crate::cmd_instance::CmdInstanceCreate {
                    instance: "things".to_string(),
                    organization: "foo".to_string(),
                    project: "bar".to_string(),
                    description: "".to_string(),
                    memory: 0,
                    cpus: 0,
                    hostname: "".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--description,-D required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no cpus".to_string(),
                cmd: crate::cmd_instance::SubCommand::Create(crate::cmd_instance::CmdInstanceCreate {
                    instance: "things".to_string(),
                    organization: "foo".to_string(),
                    project: "bar".to_string(),
                    description: "blah blah".to_string(),
                    memory: 0,
                    cpus: 0,
                    hostname: "".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--cpus,-c required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no memory".to_string(),
                cmd: crate::cmd_instance::SubCommand::Create(crate::cmd_instance::CmdInstanceCreate {
                    instance: "things".to_string(),
                    organization: "foo".to_string(),
                    project: "bar".to_string(),
                    description: "blah blah".to_string(),
                    memory: 0,
                    cpus: 2,
                    hostname: "".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--memory,m required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "delete no --confirm non-interactive".to_string(),
                cmd: crate::cmd_instance::SubCommand::Delete(crate::cmd_instance::CmdInstanceDelete {
                    instance: "things".to_string(),
                    organization: "".to_string(),
                    project: "".to_string(),
                    confirm: false,
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--confirm required when not running interactively".to_string(),
            },
            TestItem {
                name: "list zero limit".to_string(),
                cmd: crate::cmd_instance::SubCommand::List(crate::cmd_instance::CmdInstanceList {
                    limit: 0,
                    organization: "".to_string(),
                    project: "".to_string(),
                    paginate: false,
                    json: false,
                    sort_by: Default::default(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--limit must be greater than 0".to_string(),
            },
        ];

        let mut config = crate::config::new_blank_config().unwrap();
        let mut c = crate::config_from_env::EnvConfig::inherit_env(&mut config);

        for t in tests {
            let (mut io, stdout_path, stderr_path) = crate::iostreams::IoStreams::test();
            if !t.stdin.is_empty() {
                io.stdin = Box::new(std::io::Cursor::new(t.stdin));
            }
            // We need to also turn off the fancy terminal colors.
            // This ensures it also works in GitHub actions/any CI.
            io.set_color_enabled(false);
            io.set_never_prompt(true);
            let mut ctx = crate::context::Context {
                config: &mut c,
                io,
                debug: false,
            };

            let cmd_instance = crate::cmd_instance::CmdInstance { subcmd: t.cmd };
            match cmd_instance.run(&mut ctx).await {
                Ok(()) => {
                    let stdout = std::fs::read_to_string(stdout_path).unwrap();
                    let stderr = std::fs::read_to_string(stderr_path).unwrap();
                    assert!(stderr.is_empty(), "test {}: {}", t.name, stderr);
                    if !stdout.contains(&t.want_out) {
                        assert_eq!(stdout, t.want_out, "test {}: stdout mismatch", t.name);
                    }
                }
                Err(err) => {
                    let stdout = std::fs::read_to_string(stdout_path).unwrap();
                    let stderr = std::fs::read_to_string(stderr_path).unwrap();
                    assert_eq!(stdout, t.want_out, "test {}", t.name);
                    if !err.to_string().contains(&t.want_err) {
                        assert_eq!(err.to_string(), t.want_err, "test {}: err mismatch", t.name);
                    }
                    assert!(stderr.is_empty(), "test {}: {}", t.name, stderr);
                }
            }
        }
    }
}
