#![feature(linked_list_cursors)]

use std::{
    path::{Path, PathBuf}, fs:: read_to_string,
    collections::{HashMap, HashSet, LinkedList}, ops::Range, borrow::Cow, rc::Rc,
};
use lazy_static::lazy_static;
use regex::Regex;


// error and result types
pub type Error = String;
pub type Res<T> = Result<T, Error>;


#[derive(Debug)]
pub enum Type { Function, Struct, Global, Value }

#[derive(Debug)]
pub struct Artifact { pub ty: Type, pub name: String, pub name_range: Range<usize>, pub source: String, pub source_range: Range<usize> }

impl Artifact {
    pub fn quote(&self, alias: Option<&str>) -> Cow<str> {
        if let Some(alias) = alias { Cow::from(
            self.source[0..self.name_range.start].to_string()
            + alias
            + &self.source[self.name_range.end..]
        ) }
        else { Cow::from(&self.source) }
    }
}


#[derive(Debug, Clone)]
pub struct ImportRef { pub path: PathBuf, pub name: String, pub alias: Option<String> }

impl ImportRef {
    pub fn is_wildcard(&self) -> bool {
        self.name == "*"
    }
    pub fn name_alias(&self) -> Option<Cow<str>> {
        if self.is_wildcard() { None }
        else {
            self.alias.as_deref().map(|a|
                if a.contains("*") { a.replace("*", &self.name).into() } else { a.into() }
            ).or(Some((&self.name).into()))
        }
    }
    fn not_found(&self) -> String {
        format!("couldn't find {} in {}", &self.name, self.path.display())
    }
}


#[derive(Debug, Clone)]
pub struct ImportArtifact { pub trace: Vec<ImportRef>, pub artifact: Rc<Artifact> }

impl ImportArtifact {
    pub fn name_alias(&self) -> Cow<str> {
        let mut alias = Cow::from(&self.artifact.name);
        for import_ref in self.trace.iter().rev() {
            if let Some(name_alias) =
                import_ref.name_alias().map(|v| v.into()) // direct import
                .or(import_ref.alias.as_ref().map(|a| a.replace("*", &alias).into())) // aliased wildcard
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
        reimported.trace.insert(0, import_ref.clone());
        reimported
    }
}


#[derive(Debug)]
pub struct Import { pub r#ref: ImportRef, pub artifacts: Vec<Rc<ImportArtifact>> }

#[derive(Debug)]
pub struct ImportLine { pub path: PathBuf, pub imports: Vec<Import>, pub source_range: Range<usize> }


#[derive(Debug, Clone)]
pub enum Node { Artifact(Rc<Artifact>), Import(Rc<ImportArtifact>) }

impl Node {
    fn prefix_name(&self, prefix: Option<&str>) -> Cow<str> {
        let name = match &self {
            Node::Artifact(artifact) => (&artifact.name).into(),
            Node::Import(import_artifact) => import_artifact.name_alias(),
        };
        prefix.map(|pf| (pf.to_string()+&name).into()).unwrap_or(name)
    }
    pub fn artifact(&self) -> &Artifact {
        match &self {
            Node::Artifact(artifact) => &artifact,
            Node::Import(import_artifact) => &import_artifact.artifact,
        }
    }
}


// module
#[derive(Debug)]
pub struct Module {
    pub path: PathBuf,
    pub nodes: LinkedList<Node>,
    pub node_map: HashMap<String, Node>,
    pub import_lines: Vec<ImportLine>,
    pub dependent_files: HashSet<PathBuf>,
    pub source: String,
    pub code: String,
}

fn register_node(node_map: &mut HashMap<String, Node>, alias: &str, node: Node, path: &PathBuf) -> Res<()> {
    if node_map.insert(alias.to_string(), node).is_some() {
        Err(format!("duplicate identifier '{alias}' in {}", path.display()))
    } else {
        Ok(())
    }
}


// import regexes
lazy_static! {
    static ref IMPORT_LINE_REGEX: Regex = Regex::new(r#"//\s*?&import\s+([\w,\s\*]*)\s+from\s+(?:"|')(.+?)(?:"|')\s*?\n"#).unwrap();
    static ref IMPORT_SPAN_REGEX: Regex = Regex::new(r#"/\*\s*?&import\s+([\w,\s\*]*)\s+from\s+(?:"|')(.+?)(?:"|')\s*?\*/\n?"#).unwrap();
    static ref IMPORT_REGEX1: Regex = Regex::new(r"^\w+$|^\*$").unwrap();
    static ref IMPORT_REGEX3: Regex = Regex::new(r"^(\w+|\*)\s+as\s+([\w*]+)$").unwrap();
}

fn find_and_register_import_lines(
    import_lines: &mut Vec<ImportLine>,
    source_path: &Path, dir_path: &Path, source_code: &str, mut range: Range<usize>,
) -> Res<()> {

    let mut source = &source_code[range.clone()];

    while let Some(captures) = IMPORT_LINE_REGEX.captures(source).or(IMPORT_SPAN_REGEX.captures(source)) {

        let cp_range = captures.get(0).unwrap().range();
        let source_range = range.start + cp_range.start .. range.start + cp_range.end;

        let mut path = PathBuf::from(dir_path);
        path.push(&captures[2]);

        let path = path.canonicalize().map_err(|err|
            format!("{err} '{}' from '{}'", path.display(), source_path.display())
        )?;

        let mut imports = Vec::new();

        for part in captures[1].split(",") {
            let part = part.trim();
            imports.push(Import {
                r#ref: if IMPORT_REGEX1.is_match(part) {
                    Ok(ImportRef { path: path.clone(), name: part.to_string(), alias: None })
                }
                else if let Some(captures) = IMPORT_REGEX3.captures(part) {
                    if &captures[1] == "*" && !captures[2].contains("*") {
                        return Err(format!("bad import '{part}'"))
                    }
                    Ok(ImportRef { path: path.clone(), name: captures[1].to_string(), alias: Some(captures[2].to_string()) })
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
    static ref VALUE_REGEX: Regex = Regex::new(r"let\s+(\w+)").unwrap();
}

impl Module {
    fn load_from_source(path: &PathBuf) -> Res<Self> {

        // normalize path
        let dir_path = path.parent().ok_or(format!("path '{}' has no parent", path.display()))?;

        // fetch source
        let source = read_to_string(&path).map_err(|err| format!("{err} '{}'", path.display()))?;

        // parse wgsl
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_wgsl::language()).expect("Error loading wgsl grammar");

        let tree = parser.parse(&source, None).unwrap();
        let mut cursor = tree.walk();

        // module
        let mut module = Self {
            path: path.clone(), nodes: LinkedList::new(), node_map: HashMap::new(), import_lines: Vec::new(),
            dependent_files: HashSet::new(), source, code: "".to_string(),
        };

        let mut line_comment = false;
        let mut last_end = 0;

        // search through declarations in source tree
        for child in tree.root_node().children(&mut cursor) {

            // find import comments in between nodes
            let range = (last_end - if line_comment {2} else {0}) .. child.byte_range().start;

            last_end = child.byte_range().end;
            line_comment = false;

            find_and_register_import_lines(&mut module.import_lines, &path, dir_path, &module.source, range)?;

            // find nodes
            let source = &module.source[child.byte_range()];

            if let (ty, Some(captures)) = match child.kind() {
                "function_decl" => (Type::Function, FN_REGEX.captures(source)),
                "struct_decl" => (Type::Struct, STRUCT_REGEX.captures(source)),
                "global_variable_decl" => (Type::Global, GLOBAL_REGEX.captures(source)),

                "//" => { // mark line-comment start
                    line_comment = true;
                    (Type::Value, None)
                },

                _ => (Type::Value, VALUE_REGEX.captures(source)) // test arbitrary as value
            } {
                let found = captures.get(1).unwrap();
                let name = found.as_str();

                let node = Node::Artifact(Artifact {
                    ty, name: name.to_string(), name_range: found.range(), source: source.to_string(), source_range: child.byte_range(),
                }.into());

                module.nodes.push_back(node.clone());
                register_node(&mut module.node_map, &name, node, &path)?;
            }
        }

        // find import after last node
        let range = (last_end - if line_comment {2} else {0}) .. module.source.len();

        find_and_register_import_lines(&mut module.import_lines, &path, dir_path, &module.source, range)?;


        Ok(module)
    }
}


// modules
pub type ModuleCache = HashMap<PathBuf, Module>;

fn resolve_module<'a>(modules: &'a mut ModuleCache, module_trace: &mut Vec<PathBuf>, path: &PathBuf) -> Res<&'a mut Module> {

    if module_trace.contains(&path) { return Err(format!(
        "circular dependency {} from {}",
        &path.display(),
        &module_trace.last().unwrap().display(),
    )) }

    if !modules.contains_key(path) {
        let mut module = Module::load_from_source(path)?;

        module_trace.push(path.clone());
        module.resolve_imports(modules, module_trace)?;
        module_trace.pop();

        modules.insert(module.path.clone(), module);
    }

    Ok(modules.get_mut(path).unwrap())
}


impl Module {
    fn resolve_imports(&mut self, modules: &mut ModuleCache, module_trace: &mut Vec<PathBuf>) -> Res<()> {

        let mut cursor = self.nodes.cursor_front_mut();
        cursor.move_next(); // advance to first node

        for line in self.import_lines.iter_mut() {

            let module = resolve_module(modules, module_trace, &line.path)?;

            while cursor.current().and_then(|node| {
                if let Node::Artifact(artifact) = node {
                    if line.source_range.start < artifact.source_range.start { return None }
                }
                Some(())
            }).is_some() { cursor.move_next() }


            for import in line.imports.iter_mut() {
                for (name, node) in
                    if import.r#ref.is_wildcard() {
                        let prefix = import.r#ref.alias.as_deref();

                        module.nodes.iter().map(|node| {
                            (node.prefix_name(prefix), node.clone())
                        }).collect()
                    }
                    else {
                        if let Some(node) = module.node_map.get(&import.r#ref.name) {
                            Ok(vec![(import.r#ref.name_alias().unwrap().into(), node.clone())])
                        }
                        else { Err(import.r#ref.not_found()) }?
                    }
                {
                    let import_artifact: Rc<ImportArtifact> = match node {
                        Node::Import(import_artifact) => import_artifact.reimport(&import.r#ref),
                        Node::Artifact(artifact) => ImportArtifact { trace: vec![import.r#ref.clone()], artifact },
                    }.into();

                    for import_ref in &import_artifact.trace {
                        if !self.dependent_files.contains(&import_ref.path) {
                            self.dependent_files.insert(import_ref.path.clone());
                        }
                    }

                    import.artifacts.push(import_artifact.clone());
                    let node = Node::Import(import_artifact);

                    register_node(&mut self.node_map, &name, node.clone(), &self.path)?;
                    cursor.insert_before(node);
                }
            }
        }

        Ok(())
    }

    fn generate_code(&mut self) {
        self.code = self.source.clone();

        for line in self.import_lines.iter().rev() {
            self.code.replace_range(
                line.source_range.clone(),
                &line.imports.iter().flat_map(|import| {
                    import.artifacts.iter().map(|import_artifact| {
                        import_artifact.quote() + match import_artifact.artifact.ty {
                            Type::Function | Type::Struct => "\n",
                            Type::Global | Type::Value => ";\n",
                        }
                    })
                }).collect::<String>()
            );
        }
    }
}


// module loading

fn canonicalize(path: impl AsRef<Path>) -> Res<PathBuf> {
    let path = path.as_ref();
    path.canonicalize().map_err(|err| format!("{err} '{}'", path.display()))
}


pub fn load(path: impl AsRef<Path>) -> Res<Module> {

    let mut module = Module::load_from_source(&canonicalize(path)?)?;

    module.resolve_imports(&mut ModuleCache::new(), &mut Vec::new())?;
    module.generate_code();

    Ok(module)
}

pub fn load_with_cache<'a>(cache: &'a mut ModuleCache, path: impl AsRef<Path>) -> Res<&'a mut Module> {

    let module = resolve_module(cache, &mut Vec::new(), &canonicalize(path)?)?;

    module.generate_code();

    Ok(module)
}