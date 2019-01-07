use crate::loader::Loader;
use crate::module::Module;
use crate::servo_arc::Arc;
use hashbrown::HashMap;
use parking_lot::Mutex;
use std::fs::File;
use std::io::Read;

pub type RcModuleRegistry = Arc<Mutex<ModuleRegistry>>;

pub struct ModuleRegistry {
    modules: HashMap<usize, Box<Module>>,
}

impl ModuleRegistry {
    pub fn with_rc() -> RcModuleRegistry {
        Arc::new(Mutex::new(ModuleRegistry {
            modules: HashMap::new(),
        }))
    }

    /// Parses a full file path pointing to a module.
    pub fn parse_module(&mut self, path: &str) -> Result<&Module, std::io::Error> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let loader = Loader::new();
        let module = loader.load_file(&bytes[..]).unwrap();

        let name = module.name;
        self.add_module(name, module);
        Ok(&self.modules[&name])
    }

    pub fn add_module(&mut self, atom: usize, module: Module) {
        self.modules.insert(atom, Box::new(module));
    }

    pub fn lookup(&self, atom: usize) -> Option<&Module> {
        self.modules.get(&atom).map(|module| &(**module))
    }
}
