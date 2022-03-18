use std::io::Write;

use anyhow::Result;
use clap::Parser;
use cli_macros::crud_gen;

/// Create, list, edit, view, and delete routes.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdRoute {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[crud_gen {
    tag = "routes",
}]
#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Create(CmdRouteCreate),
    Edit(CmdRouteEdit),
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdRoute {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        match &self.subcmd {
            SubCommand::Create(cmd) => cmd.run(ctx).await,
            SubCommand::Delete(cmd) => cmd.run(ctx).await,
            SubCommand::Edit(cmd) => cmd.run(ctx).await,
            SubCommand::List(cmd) => cmd.run(ctx).await,
            SubCommand::View(cmd) => cmd.run(ctx).await,
        }
    }
}

/// Create a new route.
///
/// To create a route interactively, use `oxide route create` with no arguments.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdRouteCreate {
    /// The name of the route to create.
    #[clap(name = "route", default_value = "")]
    pub route: String,

    /// The router that will hold the route.
    #[clap(long, short, default_value = "")]
    pub router: String,

    /// The VPC that holds the router.
    #[clap(long, short, default_value = "")]
    pub vpc: String,

    /// The project that holds the VPC.
    #[clap(long, short, default_value = "")]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, env = "OXIDE_ORG", default_value = "")]
    pub organization: String,

    /// The description for the route.
    #[clap(long = "description", short = 'D', default_value = "")]
    pub description: String,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdRouteCreate {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet.");
        Ok(())
    }
}

/// Edit route settings.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdRouteEdit {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdRouteEdit {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet.");
        Ok(())
    }
}
