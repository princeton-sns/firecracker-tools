extern crate serde_yaml;

use serde::{Deserialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::fs::File;

// represents an in-memory function config store
#[derive(Clone)]
pub struct Configuration {
    pub configs: BTreeMap<String, FunctionConfig>,
    runtimefs_dir: PathBuf,
    appfs_dir: PathBuf,
}

impl Configuration {
    pub fn new<R: AsRef<Path>, A: AsRef<Path>>(runtimefs_dir: R, appfs_dir: A, config_file: File) -> Configuration {
        let mut config = Configuration {
            configs: BTreeMap::new(),
            runtimefs_dir: [runtimefs_dir].iter().collect(),
            appfs_dir: [appfs_dir].iter().collect(),
        };

        let apps: serde_yaml::Result<Vec<FunctionConfig>> = serde_yaml::from_reader(config_file);
        for app in apps.unwrap() {
            config.insert(app);
        }

        return config;
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
                users: c.users,
                vcpus: c.vcpus,
                memory: c.memory,
                concurrency_limit: c.concurrency_limit,
                load_dir: c.load_dir.clone(), 
            }
        })
    }

    pub fn num_func(&self) -> usize {
        self.configs.len()
    }

    pub fn exist(&self, name: &String) -> bool {
        self.configs.contains_key(name)
    }

    // return the resource requirement of a function specified by its name
    pub fn resource_req(&self, name: &String) -> Option<(u64, usize)> {
        self.configs.get(name).map(|c| {
            (c.vcpus, c.memory)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct FunctionConfig {
    pub name: String,
    pub runtimefs: PathBuf,
    pub appfs: PathBuf,
    pub users: u32,
    pub vcpus: u64,
    pub memory: usize,
    pub concurrency_limit: usize,
    pub load_dir: Option<PathBuf>,
}

