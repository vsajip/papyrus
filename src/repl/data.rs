use super::*;
use crate::pfh::linking;

impl<Data> Default for ReplData<Data> {
    fn default() -> Self {
        let lib = SourceFile::lib();
        let lib_path = lib.path.clone();
        let mut map = HashMap::new();
        map.insert(lib_path.clone(), lib);

        let mut r = ReplData {
            cmdtree: Builder::new("papyrus")
                .into_commander()
                .expect("empty should pass"),
            file_map: map,
            current_file: lib_path,
            prompt_colour: Color::Cyan,
            out_colour: Color::BrightGreen,
            compilation_dir: default_compile_dir(),
            linking: LinkingConfiguration::default(),
            redirect_on_execution: true,
        };

        r.with_cmdtree_builder(Builder::new("papyrus"))
            .expect("should build fine");

        r
    }
}

impl<Data> ReplData<Data> {
    /// Set the compilation directory. The default is set to `$HOME/.papyrus`.
    pub fn with_compilation_dir<P: AsRef<Path>>(&mut self, dir: P) -> io::Result<&mut Self> {
        let dir = dir.as_ref();
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
        assert!(dir.is_dir());
        self.compilation_dir = dir.to_path_buf();
        Ok(self)
    }

    /// Uses the given `Builder` as the root of the command tree.
    ///
    /// An error will be returned if any command already exists.
    pub fn with_cmdtree_builder(
        &mut self,
        builder: Builder<'static, CommandResult<Data>>,
    ) -> Result<&mut Self, BuildError> {
        let cmdr = builder
            .root()
            .add_action("mut", "Begin a mutable block of code", |_, _| {
                CommandResult::BeginMutBlock
            })
            .begin_class("mod", "Handle modules")
            .add_action(
                "switch",
                "Switch to a module, creating one if necessary. switch <mod_name>",
                |_, _| CommandResult::Empty,
            )
            .end_class()
            .into_commander()?;

        self.cmdtree = cmdr;
        Ok(self)
    }

    /// Link an external library.
    ///
    /// This is primarily used for linking the calling library, and there
    /// is a function on `Extern` to work this path out. It is better to
    /// use `crates.io` than linking libraries, but this method allows for
    /// linking libraries not on `crates.io`.
    ///
    /// [See _linking_ module](../pfh/linking.html)
    pub fn with_external_lib(&mut self, lib: linking::Extern) -> &mut Self {
        self.linking.external_libs.insert(lib);
        self
    }

    /// The current linking configuration.
    /// Not mutable as it could lead to undefined behaviour if changed.
    pub fn linking(&self) -> &LinkingConfiguration {
        &self.linking
    }

    /// Not meant to used by developer. Use the macros instead.
    /// [See _linking_ module](../pfh/linking.html)
    pub unsafe fn set_data_type(mut self, data_type: &str) -> Self {
        self.linking = self.linking.with_data(data_type);
        self
    }
}
