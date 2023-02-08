use crate::diff::Diff;
use delta_lib::{
    cli, config, env, git_config, subcommands,
    utils::{
        bat::{assets, output},
        process,
    },
};
use kube::api::DynamicObject;
use std::path::PathBuf;
use tui::widgets::Paragraph;

pub struct Delta {
    config: config::Config,
    output_type: output::OutputType,
}

impl Delta {
    pub fn new() -> Self {
        process::start_determining_calling_process_in_thread();

        let assets = assets::load_highlighting_assets();
        let env = env::DeltaEnv::init();
        let mut opt =
            cli::Opt::from_git_config(env.clone(), git_config::GitConfig::try_create(&env), assets);
        opt.computed.paging_mode = output::PagingMode::Never;
        opt.side_by_side = true;
        let config = config::Config::from(opt);

        let output_type =
            output::OutputType::from_mode(&env, config.paging_mode, config.pager.clone(), &config)
                .unwrap();

        Delta {
            config: config,
            output_type: output_type,
        }
    }
}

impl<'a> Diff<'a> for Delta {
    fn diff(&mut self, minus_file: PathBuf, plus_file: PathBuf) -> std::io::Result<i32> {
        let writer = self.output_type.handle().unwrap();
        let exit_code = subcommands::diff::diff(&minus_file, &plus_file, &self.config, writer);
        Ok(exit_code)
    }

    #[allow(unused_variables)]
    fn tui_diff(
        &mut self,
        pre: &DynamicObject,
        next: &DynamicObject,
    ) -> (Paragraph<'a>, Paragraph<'a>) {
        panic!("error: not implemented yet.")
    }
}
