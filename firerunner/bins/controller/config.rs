use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub struct Configuration {
    configs: BTreeMap<String, FunctionConfig>,
    runtimefs_dir: PathBuf,
    appfs_dir: PathBuf,
}

impl Configuration {
    pub fn new<R: AsRef<Path>, A: AsRef<Path>>(runtimefs_dir: R, appfs_dir: A) -> Configuration {
        Configuration {
            configs: BTreeMap::new(),
            runtimefs_dir: [runtimefs_dir].iter().collect(),
            appfs_dir: [appfs_dir].iter().collect(),
        }
    }

    pub fn insert(&mut self, config: FunctionConfig) {
        self.configs.insert(config.name.clone(), config);
    }

    pub fn get(&self, name: &String) -> Option<FunctionConfig> {
        self.configs.get(name).map(|c| {
            FunctionConfig {
                name: c.name.clone(),
                runtimefs: [self.runtimefs_dir.clone(), c.runtimefs.clone()].iter().collect(),
                appfs: [self.appfs_dir.clone(), c.appfs.clone()].iter().collect(), 
                vcpus: c.vcpus,
                memory: c.memory,
            }
        })
    }
}

pub struct FunctionConfig {
    pub name: String,
    pub runtimefs: PathBuf,
    pub appfs: PathBuf,
    pub vcpus: u64,
    pub memory: usize,
}

pub fn lorem_js() -> FunctionConfig {
    FunctionConfig {
        name: String::from("loremjs"),
        runtimefs: PathBuf::from("nodejs.ext4"),
        appfs: PathBuf::from("loremjs.ext4"),
        vcpus: 1,
        memory: 128,
    }
}

pub fn lorem_py2() -> FunctionConfig {
    FunctionConfig {
        name: String::from("lorempy2"),
        runtimefs: PathBuf::from("python2.ext4"),
        appfs: PathBuf::from("lorempy2.ext4"),
        vcpus: 1,
        memory: 128,
    }
}

