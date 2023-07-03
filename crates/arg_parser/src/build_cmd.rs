use crate::{
    arg_path::ArgPath, input_args::InputArgs, lua_args::LuaArgs, output_args::OutputArgs,
    resource_limit::ResourceLimit,
};
use clap::Parser;
use emblem_core::context::Iteration;

/// Arguments to the build subcommand
#[derive(Clone, Debug, Default, Parser, PartialEq, Eq)]
#[warn(missing_docs)]
pub struct BuildCmd {
    #[command(flatten)]
    #[allow(missing_docs)]
    pub input: InputArgs,

    #[command(flatten)]
    #[allow(missing_docs)]
    pub output: OutputArgs,

    #[command(flatten)]
    #[allow(missing_docs)]
    pub lua: LuaArgs,

    /// Max iterations of the typesetting loop
    #[arg(long, value_parser = ResourceLimit::<Iteration>::parser(), default_value_t = Default::default(), value_name = "max")]
    pub max_iters: ResourceLimit<Iteration>,
}

impl BuildCmd {
    #[allow(dead_code)]
    pub fn output_stem(&self) -> ArgPath {
        self.output.stem.infer_from(&self.input.file)
    }
}

impl From<&BuildCmd> for emblem_core::Builder {
    fn from(cmd: &BuildCmd) -> Self {
        let output_stem = cmd.output_stem().into();
        emblem_core::Builder::new(
            cmd.input.file.clone().into(),
            output_stem,
            cmd.output.driver.clone(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{sandbox_level::SandboxLevel, Args};
    use emblem_core::context::{Memory, Resource, Step};

    #[test]
    fn output_driver() {
        assert_eq!(
            Args::try_parse_from(["em"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .output
                .driver,
            None
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "-T", "pies"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .output
                .driver,
            Some("pies".to_owned())
        );
    }

    #[test]
    fn input_file() {
        assert_eq!(
            Args::try_parse_from(["em"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .input
                .file,
            ArgPath::try_from("main.em").unwrap(),
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "-"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .input
                .file,
            ArgPath::Stdio
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "chickens"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .input
                .file,
            ArgPath::try_from("chickens").unwrap(),
        );
    }

    #[test]
    fn output_stem() {
        assert_eq!(
            Args::try_parse_from(["em"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .output_stem(),
            ArgPath::try_from("main.em").unwrap(),
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "-"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .output_stem(),
            ArgPath::Stdio,
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "-", "pies"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .output_stem(),
            ArgPath::try_from("pies").unwrap(),
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "_", "-"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .output_stem(),
            ArgPath::Stdio,
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "_", "pies"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .output_stem(),
            ArgPath::try_from("pies").unwrap(),
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "-", "pies"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .output_stem(),
            ArgPath::try_from("pies").unwrap(),
        );
    }

    #[test]
    fn max_mem() {
        assert_eq!(
            Args::try_parse_from(["em"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .max_mem,
            ResourceLimit::Limited(Resource::default_limit()),
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "--max-mem", "25"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .max_mem,
            ResourceLimit::Limited(Memory(25))
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "--max-mem", "25K"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .max_mem,
            ResourceLimit::Limited(Memory(25 * 1024))
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "--max-mem", "25M"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .max_mem,
            ResourceLimit::Limited(Memory(25 * 1024 * 1024))
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "--max-mem", "25G"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .max_mem,
            ResourceLimit::Limited(Memory(25 * 1024 * 1024 * 1024))
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "--max-mem", "unlimited"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .max_mem,
            ResourceLimit::Unlimited,
        );

        assert!(Args::try_parse_from(["em", "build", "--max-mem", "100n"])
            .unwrap_err()
            .to_string()
            .contains("unrecognised unit: n"));
    }

    #[test]
    fn max_steps() {
        assert_eq!(
            Args::try_parse_from(["em"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .max_steps,
            ResourceLimit::Limited(Resource::default_limit()),
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "--max-steps", "25"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .max_steps,
            ResourceLimit::Limited(Step(25))
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "--max-steps", "25K"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .max_steps,
            ResourceLimit::Limited(Step(25 * 1024))
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "--max-steps", "25M"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .max_steps,
            ResourceLimit::Limited(Step(25 * 1024 * 1024))
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "--max-steps", "unlimited"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .max_steps,
            ResourceLimit::Unlimited,
        );

        {
            let err = Args::try_parse_from([
                "em",
                "build",
                "--max-steps",
                &(u32::MAX as u64 + 1).to_string(),
            ])
            .unwrap_err()
            .to_string();
            assert!(
                err.contains("invalid value '4294967296'"),
                "unexpected error: {err:#?}"
            );
        }

        {
            let err = Args::try_parse_from(["em", "build", "--max-steps", "10000G"])
                .unwrap_err()
                .to_string();
            assert!(
                err.contains("invalid value '10000G'"),
                "unexpected error: {err:#?}",
            );
        }

        assert!(
            &Args::try_parse_from(["em", "build", "--max-steps", "100n"])
                .unwrap_err()
                .to_string()
                .contains("unrecognised unit: n")
        );
    }

    #[test]
    fn sandbox_level() {
        assert_eq!(
            Args::try_parse_from(["em"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .sandbox_level,
            SandboxLevel::Standard
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "--sandbox", "unrestricted"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .sandbox_level,
            SandboxLevel::Unrestricted
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "--sandbox", "standard"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .sandbox_level,
            SandboxLevel::Standard
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "--sandbox", "strict"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .sandbox_level,
            SandboxLevel::Strict
        );

        assert!(Args::try_parse_from(["em", "build", "--sandbox", "root"]).is_err());
    }

    #[test]
    fn module_args() {
        assert_eq!(
            Args::try_parse_from(["em"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .args,
            vec![]
        );

        {
            let valid_ext_args = Args::try_parse_from(["em", "build", "-ak=v", "-ak2=v2", "-ak3="])
                .unwrap()
                .command
                .build()
                .unwrap()
                .lua
                .args
                .clone();
            assert_eq!(valid_ext_args.len(), 3);
            assert_eq!(valid_ext_args[0].name(), "k");
            assert_eq!(valid_ext_args[0].value(), "v");
            assert_eq!(valid_ext_args[1].name(), "k2");
            assert_eq!(valid_ext_args[1].value(), "v2");
            assert_eq!(valid_ext_args[2].name(), "k3");
            assert_eq!(valid_ext_args[2].value(), "");
        }

        assert!(Args::try_parse_from(["em", "-a=v"]).is_err());
    }

    #[test]
    fn max_iters() {
        assert_eq!(
            Args::try_parse_from(["em", "build"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .max_iters,
            ResourceLimit::Limited(Resource::default_limit()),
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "--max-iters", "25"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .max_iters,
            ResourceLimit::Limited(Iteration(25)),
        );
        assert_eq!(
            Args::try_parse_from(["em", "build", "--max-iters", "unlimited"])
                .unwrap()
                .command
                .build()
                .unwrap()
                .max_iters,
            ResourceLimit::Unlimited,
        );
    }
}
