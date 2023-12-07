#![feature(linked_list_cursors)]

use std::{
    path::Path, fs:: read_to_string,
    collections::{HashMap, HashSet, LinkedList}, ops::Range, borrow::Cow, rc::Rc,
};
use lazy_static::lazy_static;
use regex::Regex;
use normalize_path::NormalizePath;


// error and result types
pub type Error = String;
pub type Res<T> = Result<T, Error>;


#[derive(Debug)]
pub enum Type { Function, Struct, Global, Const }

#[derive(Debug)]
pub struct Artifact { pub ty: Type, pub name: Rc<str>, pub name_range: Range<usize>, pub source: Box<str>, pub source_range: Range<usize> }

impl Artifact {
    pub fn quote(&self, alias: Option<&str>) -> Cow<str> {
        if let Some(alias) = alias { Cow::from(
            self.source[0..self.name_range.start].to_string()
            + alias
            + &self.source[self.name_range.end..]
        ) }
        else { Cow::from(&*self.source) }
    }
}


#[derive(Debug, Clone)]
pub struct ImportRef { pub path: Rc<Path>, pub name: Rc<str>, pub alias: Option<Rc<str>> }

impl ImportRef {
    pub fn is_wildcard(&self) -> bool {
        self.name.as_ref() == "*"
    }
    pub fn name_alias(&self) -> Option<Cow<str>> {
        if self.is_wildcard() { None }
        else {
            self.alias.as_deref().map(|a|
                if a.contains('*') { a.replace('*', &self.name).into() } else { a.into() }
            ).or_else(|| Some((&*self.name).into()))
        }
    }
    fn not_found(&self) -> String {
        format!("couldn't find {} in {}", &self.name, self.path.display())
    }
}


#[derive(Debug, Clone)]
pub struct ImportArtifact { pub trace: Vec<Rc<ImportRef>>, pub artifact: Rc<Artifact> }

impl ImportArtifact {
    pub fn name_alias(&self) -> Cow<str> {
        let mut alias = Cow::from(&*self.artifact.name);
        for import_ref in self.trace.iter().rev() {
            if let Some(name_alias) =
                import_ref.name_alias() // direct import
                .or_else(|| import_ref.alias.as_ref().map(|a| a.replace('*', &alias).into())) // aliased wildcard
            {
                alias = name_alias;
            }
        }
        alias
    }
    pub fn quote(&self) -> Cow<str> {
        self.artifact.quote(Some(&self.name_alias()))
    }
    fn reimport(&self, import_ref: &ImportRef) -> Self {
        let mut reimported = self.clone();
        reimported.trace.insert(0, import_ref.clone().into());
        reimported
    }
}


#[derive(Debug)]
pub struct Import { pub r#ref: ImportRef, pub artifacts: Vec<Rc<ImportArtifact>> }

#[derive(Debug)]
pub struct ImportLine { pub path: Rc<Path>, pub imports: Vec<Import>, pub source_range: Range<usize> }


#[derive(Debug, Clone)]
pub enum Node { Artifact(Rc<Artifact>), Import(Rc<ImportArtifact>) }

impl Node {
    fn prefix_name(&self, prefix: Option<&str>) -> Cow<str> {
        let name = match &self {
            Node::Artifact(artifact) => Cow::from(&*artifact.name),
            Node::Import(import_artifact) => import_artifact.name_alias(),
        };
        prefix.map(|pf| (pf.to_string()+&name).into()).unwrap_or(name)
    }
    pub fn artifact(&self) -> &Rc<Artifact> {
        match &self {
            Node::Artifact(artifact) => artifact,
            Node::Import(import_artifact) => &import_artifact.artifact,
        }
    }
}


// module
#[derive(Debug)]
pub struct Module {
    pub nodes: LinkedList<Node>,
    pub node_map: HashMap<Rc<str>, Node>,
    pub import_lines: Vec<ImportLine>,
    pub dependencies: HashSet<Rc<Path>>,
    pub source: Box<str>,
    pub code: Box<str>,
}

fn register_node(node_map: &mut HashMap<Rc<str>, Node>, alias: Rc<str>, node: Node) -> Res<()> {
    if node_map.insert(alias.clone(), node).is_some() {
        Err(format!("duplicate identifier '{alias}'"))
    } else {
        Ok(())
    }
}


// import regexes
lazy_static! {
    static ref IMPORT_LINE_REGEX: Regex = Regex::new(r#"//\s*?&\s*?import\s+([\w,\s\*]*)\s+from\s+(?:"|')(.+?)(?:"|')\s*?\n"#).unwrap();
    static ref IMPORT_SPAN_REGEX: Regex = Regex::new(r#"/\*\s*?&\s*?import\s+([\w,\s\*]*)\s+from\s+(?:"|')(.+?)(?:"|')\s*?\*/\n?"#).unwrap();
    static ref IMPORT_ERROR_REGEX: Regex = Regex::new(r#"\s*?&\s*?import\s+([\w,\s\*]*)\s+from\s+(?:"|')(.+?)(?:"|')\s*?"#).unwrap();
    static ref IMPORT_REGEX1: Regex = Regex::new(r"^\w+$|^\*$").unwrap();
    static ref IMPORT_REGEX3: Regex = Regex::new(r"^(\w+|\*)\s+as\s+([\w*]+)$").unwrap();
}

fn find_and_register_import_lines(
    import_lines: &mut Vec<ImportLine>, source_code: &str, mut range: Range<usize>, error_line: bool
) -> Res<()> {

    let mut source = &source_code[range.clone()];

    while let Some(captures) = {
        if error_line { IMPORT_ERROR_REGEX.captures(source) }
        else {
            IMPORT_LINE_REGEX.captures(source)
            .or_else(|| IMPORT_SPAN_REGEX.captures(source))
        }
    } {

        let cp_range = captures.get(0).unwrap().range();
        let source_range = range.start + cp_range.start .. range.start + cp_range.end;

        let path: Rc<Path> = AsRef::<Path>::as_ref(&captures[2]).into();

        let mut imports = Vec::new();

        for part in captures[1].split(',') {
            let part = part.trim();
            imports.push(Import {
                r#ref: if IMPORT_REGEX1.is_match(part) {
                    Ok(ImportRef { path: path.clone(), name: part.into(), alias: None })
                }
                else if let Some(captures) = IMPORT_REGEX3.captures(part) {
                    if &captures[1] == "*" && !captures[1].contains('*') {
                        return Err(format!("bad import '{part}'"))
                    }
                    Ok(ImportRef { path: path.clone(), name: captures[1].into(), alias: Some(captures[2].into()) })
                }
                else { Err(format!("bad import '{part}'")) }
                ?,
                artifacts: Vec::new(),
            });
        }

        import_lines.push(ImportLine { path, imports, source_range: source_range.clone() });

        range = source_range.end..range.end;
        source = &source_code[range.clone()];
    }

    Ok(())
}


// module parsing regexes
lazy_static! {
    static ref FN_REGEX: Regex = Regex::new(r"fn\s+(\w+)").unwrap();
    static ref STRUCT_REGEX: Regex = Regex::new(r"struct\s+(\w+)").unwrap();
    static ref GLOBAL_REGEX: Regex = Regex::new(r"(\w+)\s*:").unwrap();
    static ref CONST_REGEX: Regex = Regex::new(r"const\s+(\w+)").unwrap();
}

impl Module {

    fn load_source_from_path(path: impl AsRef<Path>) -> Res<Self> {

        let path = path.as_ref();

        // fetch source
        let source = read_to_string(path).map_err(|err| format!("{err} '{}'", path.display()))?;

        Self::load_source(source.into())
    }


    fn load_source(source: Cow<str>) -> Res<Self> {

        // parse wgsl
        let language = tree_sitter_wgsl::language();

        // SAFETY: we rely on the type equivalency between tree-sitter and tree-sitter-c2rust
        let language = unsafe { std::mem::transmute(language) };

        let mut parser = tree_sitter_c2rust::Parser::new();
        parser.set_language(language).expect("Error loading wgsl grammar");

        let tree = parser.parse(source.as_ref(), None).unwrap();
        let mut cursor = tree.walk();

        // module
        let mut module = Self {
            nodes: LinkedList::new(), node_map: HashMap::new(), import_lines: Vec::new(),
            dependencies: HashSet::new(), source: source.into(), code: "".into(),
        };

        let mut line_comment = false;
        let mut last_end = 0;

        // search through declarations in source tree
        for child in tree.root_node().children(&mut cursor) {

            // find import comments in between nodes
            let range = (last_end - if line_comment {2} else {0}) .. child.byte_range().start;

            last_end = child.byte_range().end;
            line_comment = false;

            find_and_register_import_lines(&mut module.import_lines, &module.source, range, false)?;

            // find nodes
            let source = &module.source[child.byte_range()];

            if let (ty, Some(captures)) = match child.kind() {
                "function_decl" => (Type::Function, FN_REGEX.captures(source)),
                "struct_decl" => (Type::Struct, STRUCT_REGEX.captures(source)),
                "global_variable_decl" => (Type::Global, GLOBAL_REGEX.captures(source)),

                "//" => { // mark line-comment start
                    line_comment = true;
                    (Type::Const, None)
                },

                "ERROR" => { // mark line-comment start
                    find_and_register_import_lines(&mut module.import_lines, &module.source, child.byte_range(), true)?;
                    (Type::Const, None)
                },

                _ => (Type::Const, CONST_REGEX.captures(source)), // test arbitrary as const

            } {
                let found = captures.get(1).unwrap();
                let name: Rc<str> = found.as_str().into();

                let node = Node::Artifact(Artifact {
                    ty, name: name.clone(), name_range: found.range(), source: source.into(), source_range: child.byte_range(),
                }.into());

                module.nodes.push_back(node.clone());
                register_node(&mut module.node_map, name, node)?;
            }
        }

        // find import after last node
        let range = (last_end - if line_comment {2} else {0}) .. module.source.len();

        find_and_register_import_lines(&mut module.import_lines, &module.source, range, false)?;

        Ok(module)
    }
}


// helper
fn parent_path(path: &Path) -> Res<&Path> {
    path.parent().ok_or_else(|| format!("invalid path '{}'", path.display()))
}

fn normpath(path: &Path) -> Cow<Path> {
    if path.is_normalized() {
        path.into()
    }
    else {
        path.try_normalize().map(|p| p.into())
        .unwrap_or(path.into())
    }
}


// modules

pub struct ModuleCache(HashMap<Rc<Path>, Module>);

impl std::ops::Deref for ModuleCache {
    type Target = HashMap<Rc<Path>, Module>;
    fn deref(&self) -> &HashMap<Rc<Path>, Module> { &self.0 }
}

impl std::ops::DerefMut for ModuleCache {
    fn deref_mut(&mut self) -> &mut HashMap<Rc<Path>, Module> { &mut self.0 }
}


impl ModuleCache {

    fn resolve_module(&mut self, module_trace: &mut Vec<Rc<Path>>, path: &Path) -> Res<&mut Module> {

        let path = normpath(path).as_ref().into();

        if module_trace.contains(&path) { return Err(format!(
            "circular dependency {} from {}",
            path.display(),
            module_trace.last().unwrap().display(),
        )) }

        if !self.contains_key(&path) {
            let mut module = Module::load_source_from_path(&path)?;

            let dir_path = parent_path(&path)?;

            module_trace.push(path.clone());
            module.resolve_imports(self, module_trace, &Rc::from(dir_path))?;
            module_trace.pop();

            self.insert(path.clone(), module);
        }

        Ok(self.get_mut(&path).unwrap())
    }
}


impl Module {

    fn resolve_imports(&mut self, cache: &mut ModuleCache, module_trace: &mut Vec<Rc<Path>>, dir_path: &Path) -> Res<()> {

        let mut cursor = self.nodes.cursor_front_mut();
        cursor.move_next(); // advance to first node

        for line in self.import_lines.iter_mut() {

            let import_path = dir_path.join(&line.path);
            let import_path = normpath(&import_path);

            let import_dir_path = parent_path(&import_path)?;

            let module = cache.resolve_module(module_trace, &import_path)?;

            // sync node curser to line
            let is_before_line = |node: &mut Node| {
                if let Node::Artifact(artifact) = node {
                    if artifact.source_range.start > line.source_range.start { return None }
                }
                Some(())
            };

            while cursor.current().and_then(is_before_line).is_some() {
                cursor.move_next();
            }

            // ... before inserting import nodes

            for import in line.imports.iter_mut() {
                for (name, node) in
                    if import.r#ref.is_wildcard() {
                        let prefix = import.r#ref.alias.as_deref();

                        module.nodes.iter().map(|node| {
                            (node.prefix_name(prefix), node.clone())
                        }).collect()
                    }
                    else {
                        if let Some(node) = module.node_map.get(&*import.r#ref.name) {
                            Ok(vec![(import.r#ref.name_alias().unwrap(), node.clone())])
                        }
                        else { Err(import.r#ref.not_found()) }?
                    }
                {
                    let import_artifact: Rc<ImportArtifact> = match node {
                        Node::Import(import_artifact) => import_artifact.reimport(&import.r#ref),
                        Node::Artifact(artifact) => ImportArtifact { trace: vec![import.r#ref.clone().into()], artifact },
                    }.into();

                    for import_ref in &import_artifact.trace {

                        let import_ref_path = normpath(&import_dir_path.join(&import_ref.path)).into();

                        // even if already in there
                        self.dependencies.insert(import_ref_path);
                    }

                    import.artifacts.push(import_artifact.clone());
                    let node = Node::Import(import_artifact);

                    register_node(&mut self.node_map, name.into(), node.clone())?;
                    cursor.insert_before(node);
                }
            }
        }

        Ok(())
    }

    fn generate_code(&mut self) {
        let mut code = self.source.to_string();

        for line in self.import_lines.iter().rev() {
            code.replace_range(
                line.source_range.clone(),
                &line.imports.iter().flat_map(|import| {
                    import.artifacts.iter().map(|import_artifact| {
                        import_artifact.quote() + match import_artifact.artifact.ty {
                            Type::Function | Type::Struct => "\n",
                            Type::Global | Type::Const => ";\n",
                        }
                    })
                }).collect::<String>()
            );
        }

        self.code = code.into();
    }


    // module loading

    pub fn load<'a>(source_code: impl Into<Cow<'a ,str>>) -> Res<Self> {
        let mut module = Module::load_source(source_code.into())?;
        module.resolve_imports(&mut ModuleCache::new(), &mut Vec::new(), "".as_ref())?;
        module.generate_code();
        Ok(module)
    }


    pub fn load_from_path(path: impl AsRef<Path>) -> Res<Self> {

        let path = path.as_ref();
        let dir_path = parent_path(path)?;

        let mut module = Module::load_source_from_path(path)?;

        module.resolve_imports(&mut ModuleCache::new(), &mut Vec::new(), dir_path)?;
        module.generate_code();

        Ok(module)
    }
}



impl ModuleCache {

    pub fn new() -> Self { Self(HashMap::new()) }

    pub fn module(&mut self, path: impl AsRef<Path>) -> Option<&mut Module> {
        self.get_mut(path.as_ref().into())
    }

    pub fn load<'a>(&mut self, path: impl AsRef<Path>, source_code: impl Into<Cow<'a ,str>>) -> Res<&Module> {
        let path: Rc<Path> = normpath(path.as_ref()).as_ref().into();
        let dir_path = parent_path(&path)?;

        let mut module = Module::load_source(source_code.into())?;

        module.resolve_imports(self, &mut Vec::new(), &dir_path)?;

        module.generate_code();

        self.insert(path.clone(), module);

        Ok(self.get_mut(&path).unwrap())
    }

    pub fn load_from_path(&mut self, path: impl AsRef<Path>) -> Res<&mut Module> {

        let module = self.resolve_module(&mut Vec::new(), path.as_ref())?;

        module.generate_code();

        Ok(module)
    }

}