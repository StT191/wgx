
use std::{
    ops::Range, borrow::Cow,
    path::{Path, PathBuf}, fs:: read_to_string, sync::LazyLock,
};
use regex::Regex;
use naga::{FastHashMap, FastHashSet};
use naga::{front::wgsl, valid::{ValidationFlags, Validator, Capabilities, ModuleInfo}};
use hashbrown::hash_map::Entry;
use anyhow::{Result as Res, Context, anyhow, bail};

pub use naga;


#[derive(Debug)]
struct Include { path: Box<Path>, source_range: Range<usize> }


// module
#[derive(Debug)]
pub struct Module {
    path: Box<Path>,
    includes: Box<[Include]>,
    dependencies: FastHashSet<Box<Path>>,
    source: Box<str>,
    code: Box<str>,
}


static TEST_REGEXES: LazyLock<[Regex; 2]> = LazyLock::new(|| [
    Regex::new(r#"(\n|}|;|^)([\t ]*)(?://)?[\t ]*&[\t ]*include[\t ]+(?:"|')(.+?)(?:"|')[\t ]*(;|\n|$)"#).unwrap(), // \n|}|; // & include "<path>" ;|\n
    Regex::new(r#"(\n|}|;|^)(\s*)/\*\s*&\s*include\s+(?:"|')(.+?)(?:"|')\s*;?\s*\*/()"#).unwrap(), // \n|}|; /* & include "<path>" ;? */
]);

impl Module {

    fn parse(path: Box<Path>, source: Cow<str>) -> Self {

        let mut includes = Vec::new();

        for test_regex in TEST_REGEXES.iter() {

            let mut from = 0;

            while let Some(captures) = test_regex.captures_at(&source, from) {

                let path: Box<Path> = AsRef::<Path>::as_ref(&captures[3]).into();

                let matched = captures.get(0).unwrap();

                let prefix = &captures[1];

                let start =
                    matched.start() + prefix.len() +
                    if prefix == "}" || prefix == ";" { captures[2].len() } else { 0 }
                ;

                let end = matched.end() - if &captures[4] == "\n" { 1 } else { 0 };

                includes.push(Include { path, source_range: start..end });

                from = matched.end() - captures[4].len();
            }
        }

        Self {
            path, includes: includes.into(), dependencies: FastHashSet::default(),
            source: source.into(), code: "".into(),
        }
    }

    fn load_source(path: &Path) -> Res<Cow<'_, str>> {

        // fetch source
        let source = read_to_string(path).with_context(
            || format!("failed loading module from path '{}'", path.display())
        )?;

        Ok(source.into())
    }
}


// helper
fn parent_path(path: &Path) -> Res<&Path> {
    path.parent().with_context(|| format!("invalid path '{}'", path.display()))
}

fn normpath(path: &Path) -> Box<Path> {

    let mut normal = PathBuf::new();
    let mut level: usize = 0;

    for part in path.iter() {
        if part == ".." {
            if level != 0 { normal.pop(); level -= 1 }
            else { normal.push(".."); }
        }
        else if part != "." {
            normal.push(part);
            level += 1;
        }
    }

    normal.into()
}


// modules

#[derive(Default)]
pub struct ModuleCache { map: FastHashMap<Box<Path>, Module> }

impl ModuleCache {

    fn insert_and_get(&mut self, key: Box<Path>, module: Module) -> &Module {
        match self.map.entry(key) {
            Entry::Occupied(mut occupied) => {
                occupied.insert(module);
                occupied.into_mut()
            },
            Entry::Vacant(vacant) => vacant.insert(module),
        }
    }

    fn resolve_module(&mut self, module_trace: &mut Vec<Box<Path>>, path: &Path) -> Res<&Module> {

        if module_trace.iter().any(|p| p.as_ref() == path) { bail!(
            "circular dependency {} from {}",
            path.display(),
            module_trace.last().unwrap().display(),
        ) }

        if !self.map.contains_key(path) {
            let code = Module::load_source(path)?;
            let mut module = Module::parse(path.into(), code);

            let dir_path = parent_path(path)?;

            module_trace.push(path.into());
            module.resolve_includes(self, module_trace, dir_path)?;
            let path = module_trace.pop().unwrap();

            Ok(self.insert_and_get(path, module))
        }
        else {
            Ok(self.map.get_mut(path).unwrap())
        }

    }
}


impl Module {

    fn resolve_includes(&mut self, cache: &mut ModuleCache, module_trace: &mut Vec<Box<Path>>, dir_path: &Path) -> Res<()> {

        let mut code = self.source.to_string();

        for include in self.includes.iter().rev() {

            let include_path = normpath(&dir_path.join(&include.path));
            let include_dir_path = parent_path(&include_path)?;

            let module = cache.resolve_module(module_trace, &include_path)?;

            for dependency in &module.dependencies {
                let include_path = normpath(&include_dir_path.join(dependency));
                self.dependencies.insert(include_path);
            }

            self.dependencies.insert(include_path);

            code.replace_range(
                include.source_range.clone(),
                &module.code,
            );
        }

        self.code = code.into();

        Ok(())
    }

    // module loading

    fn load_helper(cache: Option<&mut ModuleCache>, path: &Path, source_code: Option<Cow<str>>) -> Res<Module> {

        let path = normpath(path.as_ref());
        let dir_path = parent_path(&path)?;

        let source_code = match source_code {
            Some(code) => code,
            None => Module::load_source(&path)?,
        };

        let mut module = Module::parse(path.clone(), source_code);

        let mut temp_cache = ModuleCache::new();
        let cache = cache.unwrap_or(&mut temp_cache);

        module.resolve_includes(cache, &mut Vec::new(), dir_path)?;

        Ok(module)
    }

    pub fn load<'a>(path: impl AsRef<Path>, source_code: impl Into<Cow<'a, str>>) -> Res<Self> {
        Self::load_helper(None, path.as_ref(), Some(source_code.into()))
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Res<Self> {
        Self::load_helper(None, path.as_ref(), None)
    }

    // accessors

    pub fn dependencies(&self) -> impl Iterator<Item=&Path> {
        self.dependencies.iter().map(|path| path.as_ref())
    }

    pub fn source(&self) -> &str { self.source.as_ref() }
    pub fn code(&self) -> &str { self.code.as_ref() }

    pub fn naga_module(&self, validate: Option<(ValidationFlags, Capabilities)>) -> Res<(naga::Module, Option<ModuleInfo>)> {
        let module = naga_module(&self.code, &self.path)?;
        let module_info = match validate {
            Some(config) => Some(naga_validate(config, &module, &self.code, &self.path)?),
            None => None,
        };
        Ok((module, module_info))
    }
}


// naga validation

pub fn naga_module(source: &str, path: impl AsRef<Path>) -> Res<naga::Module> {
    wgsl::parse_str(source)
    .map_err(|err| anyhow!(err.emit_to_string_with_path(source, path)))
}

pub fn naga_validate(
    config: (ValidationFlags, Capabilities), module: &naga::Module, source: &str, path: impl AsRef<Path>,
) -> Res<ModuleInfo> {
    Validator::new(config.0, config.1).validate(module)
    .map_err(|err| anyhow!(err.emit_to_string_with_path(source, &path.as_ref().display().to_string())))
}


impl ModuleCache {

    pub fn new() -> Self { Self::default() }

    pub fn module(&self, path: impl AsRef<Path>) -> Option<&Module> {
        self.map.get(path.as_ref())
    }

    pub fn modules(&self) -> impl Iterator<Item=(&Path, &Module)> {
        self.map.iter().map(|(key, module)| (key.as_ref(), module))
    }

    fn load_helper(&mut self, path: &Path, source_code: Option<Cow<str>>) -> Res<&Module> {
        let module = Module::load_helper(Some(self), path.as_ref(), source_code)?;
        let path = module.path.clone();
        Ok(self.insert_and_get(path, module))
    }

    pub fn load<'a>(&mut self, path: impl AsRef<Path>, source_code: impl Into<Cow<'a, str>>) -> Res<&Module> {
        self.load_helper(path.as_ref(), Some(source_code.into()))
    }

    pub fn load_from_path(&mut self, path: impl AsRef<Path>) -> Res<&Module> {
        self.load_helper(path.as_ref(), None)
    }
}