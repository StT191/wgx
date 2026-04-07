
use std::{
    ops::Range, borrow::Cow,
    path::{Path, PathBuf}, fs:: read_to_string, sync::LazyLock,
};
use regex::Regex;
use memchr::memchr;
use naga::{FastHashMap, FastHashSet};
use naga::{front::wgsl, valid::{ValidationFlags, Validator, Capabilities, ModuleInfo}};
use hashbrown::hash_map::Entry;
use anyhow::{Result as Res, Context, anyhow, bail};

pub use naga;


#[derive(Debug)]
pub struct Include { pub path: PathBuf, pub source_range: Range<usize> }


// module
#[derive(Debug)]
pub struct Module {
    path: PathBuf,
    includes: Vec<Include>,
    dependencies: FastHashSet<PathBuf>,
    source: String,
    code: String,
}

static START_REGEX: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r#"#\s*include\s+("|')"#).unwrap() // # include "<path>"
);

impl Module {

    fn parse(path: PathBuf, source: String) -> Self {

        let mut includes = Vec::new();

        let mut from = 0;

        'search: while let Some(captures) = START_REGEX.captures_at(&source, from) {

            let matched = captures.get(0).unwrap();

            let source_start = matched.start();
            let path_start = matched.end();

            let (needle, escaped, unescaped) = match &captures[1] {
                "\"" => (b'"', "\\\"", "\""),
                "'" => (b'\'', "\\'", "'"),
                _ => unreachable!(),
            };

            let mut source_end = path_start;

            while let Some(index) = memchr(needle, &source.as_bytes()[source_end..]) {

                let path_end = source_end + index;
                source_end = path_end + 1;

                let mut t = 1; // search back for escape sequence
                while source.as_bytes()[path_end-t] == b'\\' {
                    t += 1;
                }

                if t % 2 == 0 { continue; } // if escaped

                let path_string = source[path_start..path_end]
                    .replace(escaped, unescaped)
                    .replace("\\\\", "\\")
                ;

                let path = path_string.into();

                includes.push(Include {path, source_range: source_start..source_end});

                from = source_end;
                continue 'search;
            }

            break; // end of file reached
        }

        Self {
            path, includes, dependencies: FastHashSet::default(),
            code: String::new(), source,
        }
    }

    fn load_source(path: &Path) -> Res<String> {
        read_to_string(path).with_context(||
            format!("failed loading module from path '{}'", path.display())
        )
    }
}


// helper
fn parent_path(path: &Path) -> Res<&Path> {
    path.parent().with_context(|| format!("invalid path '{}'", path.display()))
}

fn normpath(path: &Path) -> PathBuf {

    let mut normal = PathBuf::with_capacity(path.as_os_str().len());
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

    normal
}


// modules

#[derive(Default)]
pub struct ModuleCache {
    map: FastHashMap<PathBuf, Module>,
}


impl ModuleCache {

    fn insert_and_get(&mut self, key: PathBuf, module: Module) -> &Module {
        match self.map.entry(key) {
            Entry::Occupied(mut occupied) => {
                occupied.insert(module);
                occupied.into_mut()
            },
            Entry::Vacant(vacant) => vacant.insert(module),
        }
    }

    fn resolve_module(&mut self, module_trace: &mut Vec<PathBuf>, path: &Path) -> Res<&Module> {

        if module_trace.iter().any(|p| *p == path) { bail!(
            "circular dependency {} from {}",
            path.display(),
            module_trace.last().unwrap().display(),
        ) }

        if !self.map.contains_key(path) {
            let code = Module::load_source(path)?;
            let mut module = Module::parse(path.to_owned(), code);

            module_trace.push(path.to_owned());

            module.resolve_includes(self, module_trace)?;
            let path = module_trace.pop().unwrap();

            Ok(self.insert_and_get(path, module))
        }
        else {
            Ok(self.map.get_mut(path).unwrap())
        }

    }
}


impl Module {

    fn resolve_includes(&mut self, cache: &mut ModuleCache, module_trace: &mut Vec<PathBuf>) -> Res<()> {

        let dir_path = parent_path(&self.path)?;
        let mut code = self.source.clone();

        for include in self.includes.iter().rev() {

            let include_path = normpath(&dir_path.join(&include.path));
            let include_dir_path = parent_path(&include_path)?;

            let module = cache.resolve_module(module_trace, &include_path)?;

            for dependency_path in &module.dependencies {
                self.dependencies.insert(
                    normpath(&include_dir_path.join(dependency_path))
                );
            }

            self.dependencies.insert(include_path);

            code.replace_range(
                include.source_range.clone(),
                &module.code,
            );
        }

        self.code = code;

        Ok(())
    }

    // module loading

    fn load_helper(cache: Option<&mut ModuleCache>, path: &Path, source_code: Option<Cow<str>>) -> Res<Self> {

        let path = normpath(path);
        parent_path(&path)?; // test for validity

        let source_code = match source_code {
            Some(code) => code.into_owned(),
            None => Self::load_source(&path)?,
        };

        let mut module = Self::parse(path, source_code);

        let mut temp_cache = ModuleCache::new();
        let cache = cache.unwrap_or(&mut temp_cache);

        Self::resolve_includes(&mut module, cache, &mut Vec::new())?;

        Ok(module)
    }

    pub fn load<'a>(path: impl AsRef<Path>, source_code: impl Into<Cow<'a, str>>) -> Res<Self> {
        Self::load_helper(None, path.as_ref(), Some(source_code.into()))
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Res<Self> {
        Self::load_helper(None, path.as_ref(), None)
    }

    pub fn path(&self) -> &Path { &self.path }

    pub fn includes(&self) -> &[Include] { &self.includes }

    pub fn dependencies(&self) -> impl ExactSizeIterator<Item=&Path> {
        self.dependencies.iter().map(|path| path.as_ref())
    }

    pub fn source(&self) -> &str { &self.source }
    pub fn code(&self) -> &str { &self.code }

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

    pub fn modules(&self) -> impl ExactSizeIterator<Item=(&Path, &Module)> {
        self.map.iter().map(|(key, module)| (key.as_ref(), module))
    }

    fn load_helper(&mut self, path: &Path, source_code: Option<Cow<str>>) -> Res<&Module> {
        let module = Module::load_helper(Some(self), path, source_code)?;
        Ok(self.insert_and_get(path.to_owned(), module))
    }

    pub fn load<'a>(&mut self, path: impl AsRef<Path>, source_code: impl Into<Cow<'a, str>>) -> Res<&Module> {
        self.load_helper(path.as_ref(), Some(source_code.into()))
    }

    pub fn load_from_path(&mut self, path: impl AsRef<Path>) -> Res<&Module> {
        self.load_helper(path.as_ref(), None)
    }
}