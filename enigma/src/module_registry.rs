use crate::atom::Atom;
use crate::loader::Loader;
use crate::module::Module;
use hashbrown::HashMap;
use parking_lot::Mutex;

pub type RcModuleRegistry = Mutex<ModuleRegistry>;

pub struct ModuleRegistry {
    pub modules: HashMap<Atom, Box<Module>>,
}

impl ModuleRegistry {
    pub fn with_rc() -> RcModuleRegistry {
        Mutex::new(ModuleRegistry {
            modules: HashMap::new(),
        })
    }

    /// Parses a full file path pointing to a module.
    pub fn parse_file(&mut self, path: &str) -> Result<&Module, std::io::Error> {
        let bytes = std::fs::read(path)?;

        self.parse_module(&bytes[..])
    }

    /// Parses a full file path pointing to a module.
    pub fn parse_module(&mut self, bytes: &[u8]) -> Result<&Module, std::io::Error> {
        let loader = Loader::new();
        let module = loader.load_file(bytes).unwrap();

        // TODO: handle uncompress, like the bif does

        let name = module.name;
        Ok(self.add_module(name, Box::new(module)))
    }

    pub fn add_module(&mut self, atom: Atom, module: Box<Module>) -> &Module {
        self.modules.insert(atom, module);
        &*self.modules[&atom]
    }

    pub fn lookup(&self, atom: Atom) -> Option<&Module> {
        self.modules.get(&atom).map(|module| &(**module))
    }
}
