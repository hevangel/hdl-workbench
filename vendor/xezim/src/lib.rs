//! xezim — SystemVerilog bytecode interpreter.
//!
//! Parsing, elaboration, and shared runtime primitives live in the
//! sibling `xezim-core` crate. This crate adds the event-driven
//! interpreter (`simulator`) and bytecode IR (`bytecode`).
//!
//! For ahead-of-time native compilation, use the `xezim-b` crate.

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use std::time;
#[cfg(target_arch = "wasm32")]
pub(crate) use web_time as time;

/// Internal engine chatter (`[PHASE]` timings, end-of-run `[PROF]`/`[FUSE]`
/// counters, compile-time optimisation notes) goes through this instead of
/// `eprintln!`: it prints only when [`verbose`] is on, so a default run shows
/// just the design's output, warnings/errors and the final result line.
macro_rules! chatter {
    ($($arg:tt)*) => {
        if $crate::verbose() {
            eprintln!($($arg)*);
        }
    };
}

static VERBOSE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Turn internal engine chatter on or off (see `chatter!`). The CLI enables
/// it for `--verbose`, `--profile`, `--sim-debug`, `XEZIM_VERBOSE=1` and the
/// profiling switches `XEZIM_PROFILE_REPORT=1` / `XEZIM_PROFILE_TIMING=1`.
pub fn set_verbose(on: bool) {
    VERBOSE.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// Whether internal engine chatter is printed.
pub fn verbose() -> bool {
    VERBOSE.load(std::sync::atomic::Ordering::Relaxed)
}

/// `XEZIM_RSS_TRACE=1`: print the process's resident memory (current and
/// high-water mark, from `/proc/self/status`) at a pipeline milestone. Peak
/// RSS is a construction-time number on large designs, so attributing it
/// needs the curve between phases, not just the final maximum.
pub fn rss_trace(label: &str) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*ON.get_or_init(|| std::env::var_os("XEZIM_RSS_TRACE").is_some()) {
        return;
    }
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let field = |key: &str| -> u64 {
        status
            .lines()
            .find(|l| l.starts_with(key))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    };
    eprintln!(
        "[RSS] {:<40} cur={:>6} MB  peak={:>6} MB",
        label,
        field("VmRSS:") / 1024,
        field("VmHWM:") / 1024
    );
}

pub mod benchw;
pub mod compiler;
pub mod env_vars;
pub mod hier_ref_lint;
pub mod intra_delay;
pub mod should_fail_lint;
pub mod type_lint;

use xezim_core::elaborate;

// Re-export xezim-core surface so existing `xezim::...` paths keep working.
pub use xezim_core::{
    LibraryCli, ModuleTimescaleCli, ParseResult, SourceDefinition, XEZIM_BYTECODE_MAGIC,
    adopted_lib_files, ast, diagnostics, lexer, log_eprintln, log_println, parse,
    parse_and_elaborate_multi, parse_str, preprocess_adopted_lib, preprocessor, progress_clear,
    progress_status, read_compiled, render_parse_diagnostics, set_compile_verbose,
    set_implicit_net_warn, set_library_cli, set_module_timescale_cli, set_strict_top, sv_parser,
    tokenize_file, write_compiled,
};

static PREPROCESSED_STASH: std::sync::Mutex<Option<xezim_core::PreprocessedSources>> =
    std::sync::Mutex::new(None);

/// Hand a finished preprocessing pass over the design to the next
/// `simulate_multi` call, which reuses it when its inputs match exactly
/// (otherwise it preprocesses afresh).
pub fn stash_preprocessed(pre: xezim_core::PreprocessedSources) {
    if let Ok(mut slot) = PREPROCESSED_STASH.lock() {
        *slot = Some(pre);
    }
}

fn take_preprocessed(
    sources: &[String],
    source_paths: &[String],
    include_dirs: &[String],
    defines: &[(String, Option<String>)],
) -> Option<xezim_core::PreprocessedSources> {
    PREPROCESSED_STASH
        .lock()
        .ok()
        .and_then(|mut slot| slot.take())
        .filter(|p| p.matches(sources, source_paths, include_dirs, defines))
}

/// Content-addressed cache for elaborated designs. The payload uses the
/// versioned `.xezbc` format, so cache hits skip parsing and elaboration while
/// runtime state is still rebuilt for each simulation.
#[derive(Clone, Debug)]
pub struct DesignCacheConfig {
    pub directory: std::path::PathBuf,
    /// CLI semantics not represented directly by the preprocessed sources.
    pub semantic_salt: String,
    /// Library files that may be adopted on demand during elaboration.
    pub dependency_files: Vec<std::path::PathBuf>,
}

static DESIGN_CACHE_CONFIG: std::sync::OnceLock<std::sync::Mutex<Option<DesignCacheConfig>>> =
    std::sync::OnceLock::new();

/// Enable or disable the elaborated-design cache for subsequent library calls.
/// The library stays cache-free unless an embedding application opts in.
pub fn set_design_cache(config: Option<DesignCacheConfig>) {
    let cell = DESIGN_CACHE_CONFIG.get_or_init(|| std::sync::Mutex::new(None));
    if let Ok(mut slot) = cell.lock() {
        *slot = config;
    }
}

fn design_cache_config() -> Option<DesignCacheConfig> {
    DESIGN_CACHE_CONFIG
        .get()
        .and_then(|cell| cell.lock().ok().and_then(|slot| slot.clone()))
}

/// Deterministic 128-bit content hash. This is a cache key, not a security
/// boundary. Length framing prevents ambiguous concatenations.
struct CacheHash {
    a: u64,
    b: u64,
}

impl CacheHash {
    fn new() -> Self {
        Self {
            a: 0xcbf29ce484222325,
            b: 0x84222325cbf29ce4,
        }
    }

    fn bytes(&mut self, bytes: &[u8]) {
        self.a ^= bytes.len() as u64;
        self.a = self.a.wrapping_mul(0x100000001b3);
        self.b ^= (bytes.len() as u64).rotate_left(31);
        self.b = self.b.wrapping_mul(0x9e3779b185ebca87);
        for &byte in bytes {
            self.a ^= byte as u64;
            self.a = self.a.wrapping_mul(0x100000001b3);
            self.b ^= (byte as u64).wrapping_add(0x9d);
            self.b = self.b.wrapping_mul(0x9e3779b185ebca87);
        }
    }

    fn text(&mut self, text: &str) {
        self.bytes(text.as_bytes());
    }

    fn finish(&self) -> String {
        format!("{:016x}{:016x}", self.a, self.b)
    }
}

fn design_cache_key(
    config: &DesignCacheConfig,
    sources: &[String],
    source_paths: &[String],
    top_module_name: Option<&str>,
    include_dirs: &[String],
    defines: &[(String, Option<String>)],
    pre: Option<&xezim_core::PreprocessedSources>,
) -> (
    String,
    Vec<String>,
    Vec<Option<sv_parser::source_map::LineMap>>,
) {
    let mut hash = CacheHash::new();
    hash.bytes(XEZIM_BYTECODE_MAGIC);
    hash.text(env!("CARGO_PKG_VERSION"));
    hash.text(&config.semantic_salt);
    hash.text(top_module_name.unwrap_or(""));
    // An elaboration with unobserved-port elision on leaves nets out, so it
    // must not serve a run that may observe them.
    hash.bytes(&[xezim_core::elaborate::port_elision_requested() as u8]);
    // A run that downgraded driver conflicts to warnings must not serve one
    // that reports them as errors.
    hash.bytes(&[xezim_core::elaborate::relax_driver_conflicts() as u8]);
    // §6.21: the accepted initializer's lifetime changes elaboration.
    hash.bytes(&[xezim_core::elaborate::relax_implicit_static() as u8]);

    // Invalidate after a local rebuild even when the package version did not
    // change, since the executable may contain elaboration fixes.
    if let Ok(exe) = std::env::current_exe() {
        if let Ok(meta) = std::fs::metadata(exe) {
            hash.bytes(&meta.len().to_le_bytes());
            if let Ok(modified) = meta.modified() {
                if let Ok(age) = modified.duration_since(std::time::UNIX_EPOCH) {
                    hash.bytes(&age.as_nanos().to_le_bytes());
                }
            }
        }
    }

    // Hash the same preprocessed text that elaboration sees. This makes
    // nested `include contents part of the key without scanning unrelated
    // files in broad include search directories.
    let mut pp = preprocessor::Preprocessor::new();
    for dir in include_dirs {
        pp.add_include_dir(std::path::PathBuf::from(dir));
        hash.text(dir);
    }
    for (name, value) in defines {
        pp.define(
            name.clone(),
            preprocessor::MacroDef {
                name: name.clone(),
                params: None,
                body: value.clone().unwrap_or_default(),
            },
        );
        hash.text(name);
        hash.text(value.as_deref().unwrap_or(""));
    }
    // Keep the per-file preprocessed text: on a cache HIT the caller feeds
    // it back into `elab.source_texts` so runtime diagnostics (e.g. the
    // zero-delay stall report) resolve spans to `file:line` exactly as a
    // fresh parse would — the artifact itself skips the (large) texts.
    // `begin_top_level_file` matches the parse-time preprocessor state.
    let mut preprocessed_texts: Vec<String> = Vec::with_capacity(sources.len());
    let mut line_maps = Vec::with_capacity(sources.len());
    if let Some(pre) = pre {
        // Same text a fresh pass would produce (`matches` checked the
        // inputs); the caller keeps ownership for elaboration.
        for (idx, text) in pre.texts.iter().enumerate() {
            hash.text(source_paths.get(idx).map_or("", String::as_str));
            hash.text(text);
        }
    } else {
        for (idx, source) in sources.iter().enumerate() {
            let source_path = source_paths.get(idx).map(std::path::PathBuf::from);
            hash.text(source_paths.get(idx).map_or("", String::as_str));
            pp.begin_top_level_file();
            let text = pp.preprocess_file(source, source_path.as_deref());
            hash.text(&text);
            preprocessed_texts.push(text);
            line_maps.push(pp.take_line_map());
        }
    }

    let mut dependencies = config.dependency_files.clone();
    dependencies.sort();
    dependencies.dedup();
    for path in dependencies {
        hash.text(&path.to_string_lossy());
        match std::fs::read(&path) {
            Ok(bytes) => {
                let source = String::from_utf8_lossy(&bytes);
                let mut dep_pp = preprocessor::Preprocessor::new();
                for dir in include_dirs {
                    dep_pp.add_include_dir(std::path::PathBuf::from(dir));
                }
                for (name, value) in defines {
                    dep_pp.define(
                        name.clone(),
                        preprocessor::MacroDef {
                            name: name.clone(),
                            params: None,
                            body: value.clone().unwrap_or_default(),
                        },
                    );
                }
                hash.text(&dep_pp.preprocess_file(&source, Some(&path)));
            }
            Err(err) => hash.text(&format!("<unreadable:{:?}>", err.kind())),
        }
    }
    (hash.finish(), preprocessed_texts, line_maps)
}

fn read_design_cache(config: &DesignCacheConfig, key: &str) -> Option<elaborate::ElaboratedModule> {
    let path = config.directory.join(format!("{}.xezbc", key));
    if !path.is_file() {
        chatter!("[CACHE] miss {}", key);
        return None;
    }
    match read_compiled(path.to_string_lossy().as_ref()) {
        Ok(Some(elab)) => {
            chatter!("[CACHE] hit {} ({})", key, path.display());
            Some(elab)
        }
        Ok(None) => {
            eprintln!("[CACHE] invalid artifact {}; rebuilding", path.display());
            let _ = std::fs::remove_file(path);
            None
        }
        Err(err) => {
            eprintln!(
                "[CACHE] cannot load {}: {}; rebuilding",
                path.display(),
                err
            );
            let _ = std::fs::remove_file(path);
            None
        }
    }
}

fn write_design_cache(config: &DesignCacheConfig, key: &str, elab: &elaborate::ElaboratedModule) {
    if let Err(err) = std::fs::create_dir_all(&config.directory) {
        eprintln!(
            "[CACHE] cannot create {}: {}; continuing without cache",
            config.directory.display(),
            err
        );
        return;
    }
    let final_path = config.directory.join(format!("{}.xezbc", key));
    let temp_path = config
        .directory
        .join(format!(".{}.{}.tmp", key, std::process::id()));
    if let Err(err) = write_compiled(elab, temp_path.to_string_lossy().as_ref()) {
        eprintln!("[CACHE] cannot write {}: {}", temp_path.display(), err);
        let _ = std::fs::remove_file(temp_path);
        return;
    }
    if let Err(err) = std::fs::rename(&temp_path, &final_path) {
        // A concurrent process may have populated the same content key first.
        if !final_path.is_file() {
            eprintln!("[CACHE] cannot publish {}: {}", final_path.display(), err);
        }
        let _ = std::fs::remove_file(temp_path);
        return;
    }
    chatter!("[CACHE] stored {} ({})", key, final_path.display());
}

#[cfg(test)]
mod design_cache_tests {
    use super::*;

    fn key(config: &DesignCacheConfig, source: &str, top: Option<&str>) -> String {
        design_cache_key(
            config,
            &[source.to_string()],
            &["design.sv".to_string()],
            top,
            &["include".to_string()],
            &[("FEATURE".to_string(), Some("1".to_string()))],
            None,
        )
        .0
    }

    #[test]
    fn design_cache_key_is_stable_and_tracks_semantics() {
        let base = DesignCacheConfig {
            directory: std::path::PathBuf::from("unused"),
            semantic_salt: "sv2023=true;strict=true".to_string(),
            dependency_files: Vec::new(),
        };
        assert_eq!(
            key(&base, "module top; endmodule", Some("top")),
            key(&base, "module top; endmodule", Some("top"))
        );
        assert_ne!(
            key(&base, "module top; endmodule", Some("top")),
            key(&base, "module top; wire x; endmodule", Some("top"))
        );
        assert_ne!(
            key(&base, "module top; endmodule", Some("top")),
            key(&base, "module top; endmodule", Some("other"))
        );

        let mut different_mode = base.clone();
        different_mode.semantic_salt = "sv2023=false;strict=true".to_string();
        assert_ne!(
            key(&base, "module top; endmodule", Some("top")),
            key(&different_mode, "module top; endmodule", Some("top"))
        );
    }

    #[test]
    fn design_cache_key_tracks_library_contents() {
        let unique = format!(
            "xezim-cache-key-{}-{:?}.sv",
            std::process::id(),
            std::thread::current().id()
        );
        let path = std::env::temp_dir().join(unique);
        std::fs::write(&path, "module cell; endmodule\n").unwrap();
        let config = DesignCacheConfig {
            directory: std::path::PathBuf::from("unused"),
            semantic_salt: String::new(),
            dependency_files: vec![path.clone()],
        };
        let before = key(&config, "module top; endmodule", Some("top"));
        std::fs::write(&path, "module cell; wire changed; endmodule\n").unwrap();
        let after = key(&config, "module top; endmodule", Some("top"));
        let _ = std::fs::remove_file(path);
        assert_ne!(before, after);
    }

    #[test]
    fn design_cache_key_tracks_included_contents() {
        let unique = format!(
            "xezim-cache-include-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).unwrap();
        let header = dir.join("defs.svh");
        std::fs::write(&header, "`define WIDTH 8\n").unwrap();
        let source = "`include \"defs.svh\"\nmodule top; logic [`WIDTH-1:0] x; endmodule\n";
        let config = DesignCacheConfig {
            directory: std::path::PathBuf::from("unused"),
            semantic_salt: String::new(),
            dependency_files: Vec::new(),
        };
        let source_path = dir.join("design.sv").to_string_lossy().into_owned();
        let include_dir = dir.to_string_lossy().into_owned();
        let before = design_cache_key(
            &config,
            &[source.to_string()],
            &[source_path.clone()],
            Some("top"),
            &[include_dir.clone()],
            &[],
            None,
        );
        std::fs::write(&header, "`define WIDTH 16\n").unwrap();
        let after = design_cache_key(
            &config,
            &[source.to_string()],
            &[source_path],
            Some("top"),
            &[include_dir],
            &[],
            None,
        );
        let _ = std::fs::remove_dir_all(dir);
        assert_ne!((before.0, before.1), (after.0, after.1));
    }
}

// ---------------------------------------------------------------------------
// Static variable initializers that call simulation-time system functions
// (issue #26).
//
// IEEE 1800-2017 §6.21 / §10.5: a static variable's initializer is evaluated
// once at simulation start, as if the assignment were made from an `initial`
// block — so it may legally call system functions such as $urandom_range,
// $sformatf("%m"), $test$plusargs or $sqrt whose results only exist at run
// time.
//
// xezim-core's elaboration classifies ANY system call with constant arguments
// as a constant expression (so the §13.4.3 elaboration constants — $clog2,
// $bits, … — still fold in generate conditions and widths), but its
// const-eval implements only that elaboration-constant subset; every other
// system function silently folds to 0/"" and the initializer expression is
// then discarded. Rather than change core's classification, re-scan the
// parsed AST here and re-issue those initializers as synthetic time-0
// assignments in `static_init_blocks`, which the simulator schedules ahead of
// every user `initial` block — giving them the runtime evaluation §6.21
// requires.

/// System functions xezim-core's elaboration const-eval genuinely implements
/// (see `eval_const_expr_val` in xezim-core/src/elaborate.rs). Initializers
/// whose only calls are these keep their elaboration-time folded value.
const ELAB_CONST_SYSFUNCS: &[&str] = &[
    "$clog2",
    "$bits",
    "$unsigned",
    "$signed",
    "$countones",
    "$onehot",
    "$onehot0",
    "$isunknown",
    "$countbits",
    "$size",
    "$left",
    "$right",
    "$high",
    "$low",
    "$dimensions",
];

/// Does the expression contain a system call that elaboration-time const-eval
/// cannot actually evaluate (i.e. one that needs simulation-time state)?
fn contains_simtime_syscall(e: &ast::expr::Expression) -> bool {
    use ast::expr::ExprKind;
    match &e.kind {
        ExprKind::SystemCall { name, args } => {
            !ELAB_CONST_SYSFUNCS.contains(&name.as_str())
                || args.iter().any(contains_simtime_syscall)
        }
        ExprKind::Unary { operand, .. } => contains_simtime_syscall(operand),
        ExprKind::Binary { left, right, .. } => {
            contains_simtime_syscall(left) || contains_simtime_syscall(right)
        }
        ExprKind::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            contains_simtime_syscall(condition)
                || contains_simtime_syscall(then_expr)
                || contains_simtime_syscall(else_expr)
        }
        ExprKind::Concatenation(parts) => parts.iter().any(contains_simtime_syscall),
        ExprKind::Paren(inner) => contains_simtime_syscall(inner),
        ExprKind::MemberAccess { expr, .. } => contains_simtime_syscall(expr),
        ExprKind::Index { expr, index } => {
            contains_simtime_syscall(expr) || contains_simtime_syscall(index)
        }
        _ => false,
    }
}

/// Mirror of xezim-core's `is_const_expr` classification (elaborate.rs,
/// read-only there): true iff elaboration treated `e` as a constant and
/// FOLDED it (discarding the expression). Initializers classified non-const
/// already get a synthetic initial-block assignment from elaboration, so
/// re-issuing those here would run their side effects twice.
fn elab_classifies_const(
    e: &ast::expr::Expression,
    elab: &elaborate::ElaboratedModule,
    scope: &str,
) -> bool {
    use ast::expr::ExprKind;
    // Child-instance parameters are merged into the top table under their
    // instance path ("u1.P"), so check both the bare and scoped names.
    let has_param = |n: &str| -> bool {
        elab.parameters.contains_key(n)
            || (!scope.is_empty() && elab.parameters.contains_key(&format!("{}.{}", scope, n)))
    };
    match &e.kind {
        ExprKind::Number(_) | ExprKind::StringLiteral(_) => true,
        ExprKind::Ident(hier) => {
            let last = hier.path.last().map(|s| s.name.name.as_str()).unwrap_or("");
            let base = hier
                .path
                .first()
                .map(|s| s.name.name.as_str())
                .unwrap_or("");
            has_param(last) || (hier.path.len() > 1 && has_param(base))
        }
        ExprKind::Unary { operand, .. } => elab_classifies_const(operand, elab, scope),
        ExprKind::Binary { left, right, .. } => {
            elab_classifies_const(left, elab, scope) && elab_classifies_const(right, elab, scope)
        }
        ExprKind::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            elab_classifies_const(condition, elab, scope)
                && elab_classifies_const(then_expr, elab, scope)
                && elab_classifies_const(else_expr, elab, scope)
        }
        ExprKind::Concatenation(parts) => {
            parts.iter().all(|p| elab_classifies_const(p, elab, scope))
        }
        ExprKind::Paren(inner) => elab_classifies_const(inner, elab, scope),
        ExprKind::MemberAccess { expr, member } => {
            elab_classifies_const(expr, elab, scope) || has_param(&member.name)
        }
        ExprKind::Index { expr, index } => {
            elab_classifies_const(expr, elab, scope) && elab_classifies_const(index, elab, scope)
        }
        ExprKind::SystemCall { args, .. } => {
            args.iter().all(|a| elab_classifies_const(a, elab, scope))
        }
        _ => false,
    }
}

fn make_bare_ident(name: &str, span: ast::Span) -> ast::expr::Expression {
    use ast::expr::{ExprKind, Expression, HierPathSegment, HierarchicalIdentifier};
    Expression::new(
        ExprKind::Ident(HierarchicalIdentifier {
            root: None,
            path: vec![HierPathSegment {
                name: ast::Identifier {
                    name: name.to_string(),
                    span,
                },
                selects: Vec::new(),
            }],
            span,
            cached_signal_id: std::cell::Cell::new(None),
            cached_resolved_name: std::cell::OnceCell::new(),
        }),
        span,
    )
}

fn walk_module_static_inits(
    items: &[ast::decl::ModuleItem],
    defs: &xezim_core::hasher::HashMap<String, SourceDefinition>,
    elab: &elaborate::ElaboratedModule,
    scope: &str,
    depth: u32,
    out: &mut Vec<elaborate::InitialBlock>,
) {
    use ast::decl::ModuleItem;
    use ast::stmt::{Statement, StatementKind};
    if depth > 64 {
        return; // defensive recursion cap
    }
    for item in items {
        match item {
            ModuleItem::DataDeclaration(dd) => {
                for d in &dd.declarators {
                    // Unpacked-array declarators take elaboration's
                    // assignment-pattern path (always procedural) — skip.
                    if !d.dimensions.is_empty() {
                        continue;
                    }
                    let Some(init) = &d.init else { continue };
                    if contains_simtime_syscall(init) && elab_classifies_const(init, elab, scope) {
                        out.push(elaborate::InitialBlock {
                            stmt: Statement::new(
                                StatementKind::BlockingAssign {
                                    lvalue: make_bare_ident(&d.name.name, d.name.span),
                                    rvalue: init.clone(),
                                },
                                d.name.span,
                            ),
                            scope: scope.to_string(),
                        });
                    }
                }
            }
            // Unconditional `generate ... endgenerate` region — same scope.
            ModuleItem::GenerateRegion(gr) => {
                walk_module_static_inits(&gr.items, defs, elab, scope, depth + 1, out);
            }
            ModuleItem::ModuleInstantiation(mi) => {
                if let Some(SourceDefinition::Module(child)) = defs.get(&mi.module_name.name) {
                    for inst in &mi.instances {
                        // Instance arrays get per-element scopes ("u[i]") —
                        // out of scope here; leave elaboration behavior.
                        if !inst.dimensions.is_empty() {
                            continue;
                        }
                        let child_scope = if scope.is_empty() {
                            inst.name.name.clone()
                        } else {
                            format!("{}.{}", scope, inst.name.name)
                        };
                        walk_module_static_inits(
                            &child.items,
                            defs,
                            elab,
                            &child_scope,
                            depth + 1,
                            out,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

/// Re-issue module/interface-scope static variable initializers that call
/// simulation-time system functions (which elaboration const-folded to
/// 0/"") as time-0 `static_init_blocks` assignments — IEEE 1800-2017 §6.21.
pub fn defer_static_syscall_inits(
    defs: &xezim_core::hasher::HashMap<String, SourceDefinition>,
    elab: &mut elaborate::ElaboratedModule,
) {
    let mut out: Vec<elaborate::InitialBlock> = Vec::new();
    if let Some(SourceDefinition::Module(top)) = defs.get(&elab.name) {
        walk_module_static_inits(&top.items, defs, elab, "", 0, &mut out);
    }
    // §6.21 / §25.3: interface variables have static lifetime too. The module
    // walk above only descends through module definitions, so it never sees
    // an interface's initializers. Use the elaborated instance paths here:
    // they include generated and arrayed interfaces, with the actual scope
    // needed by `%m`, and exclude inactive generate branches.
    if let Some(SourceDefinition::Interface(top)) = defs.get(&elab.name) {
        walk_module_static_inits(&top.items, defs, elab, "", 0, &mut out);
    }
    for inst in &elab.instances {
        if let Some(SourceDefinition::Interface(iface)) = defs.get(&inst.def_name) {
            walk_module_static_inits(&iface.items, defs, elab, &inst.path, 0, &mut out);
        }
    }
    // Package-scope variables too (`string n = $sformatf("%m.notifier")` in
    // UVM's polling package). The assignment runs inside a block named with
    // the package as its absolute `%m` root, so `%m` reads `pkg`.
    let mut pkgs: Vec<&std::rc::Rc<ast::module::PackageDeclaration>> = defs
        .values()
        .filter_map(|d| match d {
            SourceDefinition::Package(p) => Some(p),
            _ => None,
        })
        .collect();
    pkgs.sort_by(|a, b| a.name.name.cmp(&b.name.name));
    for p in pkgs {
        for item in &p.items {
            let ast::decl::PackageItem::Data(dd) = item else {
                continue;
            };
            for d in &dd.declarators {
                let Some(init) = &d.init else { continue };
                if !d.dimensions.is_empty()
                    || !contains_simtime_syscall(init)
                    || !elab_classifies_const(init, elab, "")
                {
                    continue;
                }
                use ast::stmt::{Statement, StatementKind};
                let assign = Statement::new(
                    StatementKind::BlockingAssign {
                        lvalue: make_bare_ident(&d.name.name, d.name.span),
                        rvalue: init.clone(),
                    },
                    d.name.span,
                );
                let scoped = Statement::new(
                    StatementKind::SeqBlock {
                        name: Some(ast::Identifier {
                            name: format!("{}{}", compiler::simulator::M_ROOT_MARK, p.name.name),
                            span: d.name.span,
                        }),
                        stmts: vec![assign],
                    },
                    d.name.span,
                );
                out.push(elaborate::InitialBlock {
                    stmt: Statement::new(
                        StatementKind::SeqBlock {
                            name: None,
                            stmts: vec![scoped],
                        },
                        d.name.span,
                    ),
                    scope: String::new(),
                });
            }
        }
    }
    elab.static_init_blocks.extend(out);
}

/// IEEE 1800-2017 §18.5.1 — re-install out-of-class constraint bodies
/// (`constraint ClassName::name { … }`) that elaboration lost.
///
/// Elaboration DOES install those bodies into the class's constraint
/// prototype, but `inline_instantiations` afterwards repopulates the class
/// table from the raw AST (`elab.classes.insert(name, elaborate_class(c))`),
/// and the AST `ClassDeclaration` carries only the empty `constraint c;`
/// prototype — so the body is dropped again and the constraint never reaches
/// the solver (an `unique_a inside {[1:10]}` written out-of-class simply did
/// not constrain anything).
///
/// The bodies only exist in the parsed descriptions, which elaboration
/// consumes, so recover them by re-parsing. Gated on the design actually
/// having an out-of-class constraint whose class-side prototype is still
/// body-less, so the common case pays nothing.
fn reinstall_ooc_constraint_bodies(
    sources: &[String],
    source_paths: &[String],
    include_dirs: &[String],
    defines: &[(String, Option<String>)],
    elab: &mut elaborate::ElaboratedModule,
) {
    let needed: Vec<(String, String)> = elab
        .out_of_class_constraints
        .iter()
        .filter(|(cn, nn)| {
            elab.classes
                .get(cn)
                .and_then(|cd| cd.constraints.get(nn))
                .is_some_and(|c| c.items.is_empty())
        })
        .cloned()
        .collect();
    if needed.is_empty() {
        return;
    }
    let mut pp = preprocessor::Preprocessor::new();
    for dir in include_dirs {
        pp.add_include_dir(std::path::PathBuf::from(dir));
    }
    for (name, val) in defines {
        pp.define(
            name.clone(),
            preprocessor::MacroDef {
                name: name.clone(),
                params: None,
                body: val.clone().unwrap_or_default(),
            },
        );
    }
    for (i, source) in sources.iter().enumerate() {
        let path = source_paths.get(i).map(std::path::PathBuf::from);
        let pre = pp.preprocess_file(source, path.as_deref());
        let tokens = lexer::Lexer::new(&pre).tokenize();
        let mut parser = sv_parser::parse::Parser::new(tokens);
        let src_ast = parser.parse_source_text();
        for d in &src_ast.descriptions {
            let ast::Description::OutOfClassConstraint {
                class_name,
                constraint_name,
                items,
            } = d
            else {
                continue;
            };
            if items.is_empty()
                || !needed
                    .iter()
                    .any(|(cn, nn)| cn == class_name && nn == constraint_name)
            {
                continue;
            }
            if let Some(cd) = elab
                .classes
                .get_mut(class_name)
                .map(std::sync::Arc::make_mut)
            {
                if let Some(con) = cd.constraints.get_mut(constraint_name) {
                    con.items = items.clone();
                    con.has_body = true;
                }
            }
        }
    }
}

/// `XEZIM_MEM_CENSUS`: the elaborated design as elaboration hands it over —
/// serialized size per field (a stand-in for heap size that ranks them) and
/// the sizes of the AST node types that make up most of it.
fn elab_census(elab: &elaborate::ElaboratedModule) {
    fn ser<T: serde::Serialize>(v: &T) -> usize {
        bincode::serialized_size(v).unwrap_or(0) as usize
    }
    let mut rows: Vec<(&str, usize)> = vec![
        ("signals", ser(&elab.signals)),
        ("always_blocks", ser(&elab.always_blocks)),
        ("initial_blocks", ser(&elab.initial_blocks)),
        ("continuous_assigns", ser(&elab.continuous_assigns)),
        ("functions", ser(&elab.functions)),
        ("tasks", ser(&elab.tasks)),
        ("classes", ser(&elab.classes)),
        ("var_decl_types", ser(&elab.var_decl_types)),
        ("parameters", ser(&elab.parameters)),
        ("instances", ser(&elab.instances)),
        ("nets", ser(&elab.nets)),
        ("port_aliases", ser(&elab.port_aliases)),
        ("decl_sites", ser(&elab.decl_sites)),
        ("source_texts", ser(&elab.source_texts)),
        ("arrays", ser(&elab.arrays)),
        ("two_state_signals", ser(&elab.two_state_signals)),
    ];
    rows.sort_by_key(|(_, b)| std::cmp::Reverse(*b));
    eprintln!(
        "[MEM-CENSUS] === elaborated design (serialized MB) — {} signals, {} always, {} pending always, {} pending initial, {} pending CA ===",
        elab.signals.len(),
        elab.always_blocks.len(),
        elab.pending_always.len(),
        elab.pending_initial.len(),
        elab.pending_cont_assign.len()
    );
    for (name, b) in rows.iter().take(12) {
        eprintln!("[MEM-CENSUS] {:>9.1} MB  {}", *b as f64 / 1048576.0, name);
    }
    use std::mem::size_of;
    eprintln!(
        "[MEM-CENSUS] node sizes: Expression={} ExprKind={} HierarchicalIdentifier={} HierPathSegment={} Identifier={} Statement={} StatementKind={} Span={} Signal={} Value={}",
        size_of::<ast::expr::Expression>(),
        size_of::<ast::expr::ExprKind>(),
        size_of::<ast::expr::HierarchicalIdentifier>(),
        size_of::<ast::expr::HierPathSegment>(),
        size_of::<ast::Identifier>(),
        size_of::<ast::stmt::Statement>(),
        size_of::<ast::stmt::StatementKind>(),
        size_of::<ast::Span>(),
        size_of::<elaborate::Signal>(),
        size_of::<xezim_core::Value>(),
    );
}

/// Simulate a single source string.

/// Realtime-clock stopwatch for HUMAN-FACING phase/profile reports.
///
/// Native `SystemTime` is CLOCK_REALTIME and matches external harnesses. On
/// wasm32-unknown-unknown, use the browser clock because `SystemTime::now()`
/// panics on that target. These spans only affect printed profile data;
/// scheduling keeps `Instant` and simulation semantics are unchanged.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy)]
pub struct WallTimer(std::time::SystemTime);

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy)]
pub struct WallTimer(f64);

impl WallTimer {
    pub fn now() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self(std::time::SystemTime::now())
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self(js_sys::Date::now())
        }
    }
    pub fn elapsed(&self) -> std::time::Duration {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.0.elapsed().unwrap_or_default()
        }
        #[cfg(target_arch = "wasm32")]
        {
            std::time::Duration::from_secs_f64(((js_sys::Date::now() - self.0).max(0.0)) / 1000.0)
        }
    }
}

pub fn simulate(source: &str, max_time: u64) -> Result<compiler::Simulator, String> {
    #[cfg(target_arch = "wasm32")]
    {
        return simulate_inline(source, max_time);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        simulate_multi(
            &[source.to_string()],
            max_time,
            None,
            &[],
            &[],
            None,
            false,
            None,
            None,
            &[],
            &[],
            None,
            &[],
            0,
            u64::MAX,
            None,
            &[],
            None,
            None,
            None,
            None,
            false,
        )
    }
}

/// Compile and simulate on the calling thread. Intended for single-threaded
/// hosts such as wasm32; browser callers should invoke this only from a
/// dedicated Worker. Native callers retain [`simulate_multi`]'s large-stack
/// worker-thread policy.
pub fn simulate_inline(source: &str, max_time: u64) -> Result<compiler::Simulator, String> {
    simulate_multi_inline(&[source.to_string()], max_time, None)
}

/// Multi-source form of [`simulate_inline`], with an optional top module.
/// Native callers that need an isolated, stackless call can use this API;
/// native [`simulate_multi`] keeps its existing worker-thread behavior.
pub fn simulate_multi_inline(
    sources: &[String],
    max_time: u64,
    top_module_name: Option<&str>,
) -> Result<compiler::Simulator, String> {
    simulate_multi_inner(
        sources,
        max_time,
        top_module_name,
        &[],
        &[],
        None,
        false,
        None,
        None,
        &[],
        &[],
        None,
        &[],
        0,
        u64::MAX,
        None,
        &[],
        None,
        None,
        None,
        None,
        false,
        false,
        None,
        None,
        None,
        None,
    )
}

/// Compile and simulate a named browser/project snapshot on the calling
/// thread. `sources` is the ordered set of SV compilation units;
/// `virtual_files` contains those files plus include and runtime data files.
/// All paths are project-relative and are rejected if they are absolute or
/// escape through `..`. The ordinary native entry points remain filesystem
/// backed unless callers explicitly select this API.
pub fn simulate_project_inline(
    sources: &[(String, String)],
    virtual_files: &[(String, Vec<u8>)],
    max_time: u64,
    top_module_name: Option<&str>,
    max_project_bytes: usize,
    vcd_writer: Box<dyn std::io::Write + Send>,
) -> Result<compiler::Simulator, String> {
    simulate_project_inline_with_output_limit(
        sources,
        virtual_files,
        max_time,
        top_module_name,
        max_project_bytes,
        None,
        vcd_writer,
    )
}

/// Compile and simulate a browser/project snapshot with a bounded captured
/// console output budget. Bounded capture coalesces its retained messages into
/// one transcript record to keep per-message allocation overhead bounded. This
/// opt-in API leaves native entry points and their output behavior unchanged.
pub fn simulate_project_inline_with_output_limit(
    sources: &[(String, String)],
    virtual_files: &[(String, Vec<u8>)],
    max_time: u64,
    top_module_name: Option<&str>,
    max_project_bytes: usize,
    output_limit: Option<usize>,
    vcd_writer: Box<dyn std::io::Write + Send>,
) -> Result<compiler::Simulator, String> {
    simulate_project_inline_with_plusargs(
        sources,
        virtual_files,
        max_time,
        top_module_name,
        max_project_bytes,
        output_limit,
        &[],
        vcd_writer,
    )
}

/// Same as [`simulate_project_inline_with_output_limit`], with the run's
/// plusargs (`+name`, `+name=value`, `+seed=n`) visible to `$test$plusargs`
/// and `$value$plusargs`. This is the browser equivalent of the native
/// command line's `+` arguments.
#[allow(clippy::too_many_arguments)]
pub fn simulate_project_inline_with_plusargs(
    sources: &[(String, String)],
    virtual_files: &[(String, Vec<u8>)],
    max_time: u64,
    top_module_name: Option<&str>,
    max_project_bytes: usize,
    output_limit: Option<usize>,
    plusargs: &[String],
    vcd_writer: Box<dyn std::io::Write + Send>,
) -> Result<compiler::Simulator, String> {
    let source_texts: Vec<String> = sources.iter().map(|(_, text)| text.clone()).collect();
    let source_paths: Vec<String> = sources.iter().map(|(path, _)| path.clone()).collect();
    let rewritten_sources: Vec<String> = source_texts
        .iter()
        .map(|source| intra_delay::rewrite_intra_assignment_delays(source))
        .collect();
    let pre = xezim_core::preprocess_design_with_virtual_files(
        &rewritten_sources,
        &source_paths,
        &[],
        &[],
        virtual_files,
    )?;
    if !pre.errors.is_empty() {
        return Err(pre.errors.join("\n"));
    }
    let sim = simulate_multi_inner(
        &source_texts,
        max_time,
        top_module_name,
        &[],
        &source_paths,
        None,
        false,
        None,
        None,
        &[],
        plusargs,
        None,
        &[],
        0,
        u64::MAX,
        None,
        &[],
        None,
        None,
        None,
        None,
        false,
        true,
        Some((virtual_files.to_vec(), max_project_bytes)),
        Some(vcd_writer),
        Some(pre),
        output_limit,
    )?;
    Ok(sim)
}

/// Public entry point: runs the whole compile+simulate on a worker thread
/// with a LARGE stack. The parser, elaborator, and statement interpreter are
/// all deeply recursive; an embedder's thread (a 2 MiB `cargo test` thread,
/// a debug build's frames) otherwise overflows on real designs — CI's debug
/// UVM suites aborted with "fatal runtime error: stack overflow". Same
/// policy as the binary's `main()`: XEZIM_STACK_MB overrides (0 = run on
/// the caller's thread); the memory is virtual, committed only as used.
#[allow(clippy::too_many_arguments)]
pub fn simulate_multi(
    sources: &[String],
    max_time: u64,
    top_module_name: Option<&str>,
    include_dirs: &[String],
    source_paths: &[String],
    settle_limit: Option<u32>,
    activity_mon: bool,
    sdf_file: Option<&str>,
    sdf_select: Option<xezim_core::sdf::DelaySelect>,
    defines: &[(String, Option<String>)],
    plusargs: &[String],
    xtrace_file: Option<&str>,
    xtrace_scopes: &[String],
    xtrace_from_ns: u64,
    xtrace_to_ns: u64,
    fst_file: Option<&str>,
    fst_scopes: &[String],
    emit_hypergraph: Option<&str>,
    load_partition: Option<&str>,
    write_profile: Option<&str>,
    profile_input: Option<&str>,
    collapse_islands: bool,
) -> Result<compiler::Simulator, String> {
    #[cfg(target_arch = "wasm32")]
    {
        return simulate_multi_inner(
            sources,
            max_time,
            top_module_name,
            include_dirs,
            source_paths,
            settle_limit,
            activity_mon,
            sdf_file,
            sdf_select,
            defines,
            plusargs,
            xtrace_file,
            xtrace_scopes,
            xtrace_from_ns,
            xtrace_to_ns,
            fst_file,
            fst_scopes,
            emit_hypergraph,
            load_partition,
            write_profile,
            profile_input,
            collapse_islands,
            false,
            None,
            None,
            None,
            None,
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let stack_mb: usize = std::env::var("XEZIM_STACK_MB")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1024);
        if stack_mb == 0 {
            return simulate_multi_inner(
                sources,
                max_time,
                top_module_name,
                include_dirs,
                source_paths,
                settle_limit,
                activity_mon,
                sdf_file,
                sdf_select,
                defines,
                plusargs,
                xtrace_file,
                xtrace_scopes,
                xtrace_from_ns,
                xtrace_to_ns,
                fst_file,
                fst_scopes,
                emit_hypergraph,
                load_partition,
                write_profile,
                profile_input,
                collapse_islands,
                false,
                None,
                None,
                None,
                None,
            );
        }
        // `Simulator` is not auto-`Send`: it carries raw pointers into its OWN
        // allocations (`vpi_argv` → `vpi_arg_cstrings`, leaked DPI scope boxes)
        // and thread-local registrations that are re-established per call. A
        // one-shot OWNERSHIP TRANSFER of the fully built value out of the worker
        // is sound — nothing on the worker retains a reference, and the caller
        // uses it single-threaded exactly as before this wrapper existed.
        struct SendResult(Result<compiler::Simulator, String>);
        // SAFETY: see above — transfer-once of a self-contained value.
        unsafe impl Send for SendResult {}
        std::thread::scope(|scope| {
            std::thread::Builder::new()
                .name("xezim-sim".to_string())
                .stack_size(stack_mb * 1024 * 1024)
                .spawn_scoped(scope, || {
                    SendResult(simulate_multi_inner(
                        sources,
                        max_time,
                        top_module_name,
                        include_dirs,
                        source_paths,
                        settle_limit,
                        activity_mon,
                        sdf_file,
                        sdf_select,
                        defines,
                        plusargs,
                        xtrace_file,
                        xtrace_scopes,
                        xtrace_from_ns,
                        xtrace_to_ns,
                        fst_file,
                        fst_scopes,
                        emit_hypergraph,
                        load_partition,
                        write_profile,
                        profile_input,
                        collapse_islands,
                        false,
                        None,
                        None,
                        None,
                        None,
                    ))
                })
                .expect("spawn simulation worker")
                .join()
                .map(|r| r.0)
                .unwrap_or_else(|_| Err("simulation worker panicked".to_string()))
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn simulate_multi_inner(
    sources: &[String],
    max_time: u64,
    top_module_name: Option<&str>,
    include_dirs: &[String],
    source_paths: &[String],
    settle_limit: Option<u32>,
    activity_mon: bool,
    sdf_file: Option<&str>,
    sdf_select: Option<xezim_core::sdf::DelaySelect>,
    defines: &[(String, Option<String>)],
    plusargs: &[String],
    xtrace_file: Option<&str>,
    xtrace_scopes: &[String],
    xtrace_from_ns: u64,
    xtrace_to_ns: u64,
    fst_file: Option<&str>,
    fst_scopes: &[String],
    emit_hypergraph: Option<&str>,
    load_partition: Option<&str>,
    write_profile: Option<&str>,
    profile_input: Option<&str>,
    collapse_islands: bool,
    force_wave: bool,
    virtual_files: Option<(Vec<(String, Vec<u8>)>, usize)>,
    vcd_writer: Option<Box<dyn std::io::Write + Send>>,
    preprocessed: Option<xezim_core::PreprocessedSources>,
    output_capture_limit: Option<usize>,
) -> Result<compiler::Simulator, String> {
    let total_start = WallTimer::now();
    let compilation_start = WallTimer::now();
    rss_trace("start");
    // IEEE 1800-2017 §9.4.5: the parser discards intra-assignment delays
    // (`lhs = #d rhs`); canonicalize them into a marker call the simulator
    // implements (see `intra_delay`) before parsing.
    let raw_sources = sources;
    let sources: Vec<String> = sources
        .iter()
        .map(|s| intra_delay::rewrite_intra_assignment_delays(s))
        .collect();
    // The driver's own preprocessing pass (main.rs), reusable when the
    // rewrite above left every source untouched.
    let mut pre = match preprocessed {
        Some(pre) => Some(pre),
        None => take_preprocessed(raw_sources, source_paths, include_dirs, defines)
            .filter(|_| sources.as_slice() == raw_sources),
    };
    let cache = design_cache_config();
    if cache.is_some() && pre.is_none() {
        // One pass serves both the cache key and (on a miss) elaboration.
        pre = Some(xezim_core::preprocess_design(
            &sources,
            source_paths,
            include_dirs,
            defines,
        ));
    }
    let mut cache_pp_texts: Vec<String> = Vec::new();
    let mut cache_line_maps = Vec::new();
    let cache_key = cache.as_ref().map(|config| {
        let (key, texts, maps) = design_cache_key(
            config,
            &sources,
            source_paths,
            top_module_name,
            include_dirs,
            defines,
            pre.as_ref(),
        );
        cache_pp_texts = texts;
        cache_line_maps = maps;
        key
    });
    let cached_elab = cache
        .as_ref()
        .zip(cache_key.as_deref())
        .and_then(|(config, key)| read_design_cache(config, key));

    let elab = if let Some(mut elab) = cached_elab {
        drop(sources);
        // The artifact skips the (large) preprocessed texts; refill them from
        // the cache-key pass so runtime diagnostics keep `file:line`
        // resolution on cache hits. `source_files` / `src_file_of_module`
        // travel inside the artifact.
        match pre.take() {
            Some(p) => {
                elab.source_texts = p.texts;
                elab.source_line_maps = p.line_maps;
            }
            None => {
                elab.source_texts = std::mem::take(&mut cache_pp_texts);
                elab.source_line_maps = std::mem::take(&mut cache_line_maps);
            }
        }
        if elab.source_files.is_empty() {
            elab.source_files = source_paths.to_vec();
        }
        // A cache HIT skips elaboration, so its diagnostics (implicit-net
        // warnings, port-width lint, unresolved-module notes, width-underflow)
        // would silently vanish — replay the ones captured on the cold run.
        for line in &elab.elab_diagnostics {
            eprintln!("{}", line);
        }
        elab
    } else {
        // Capture elaboration diagnostics so a future warm hit can replay them.
        if cache.is_some() {
            xezim_core::elab_diag_capture_begin();
        }
        // Fresh per-kind duplicate counters for this elaboration, so a second
        // run in the same process/thread reports its own first five.
        xezim_core::elab_diag_reset_counts();
        let (definitions, mut elab) = xezim_core::parse_and_elaborate_multi_preprocessed(
            &sources,
            top_module_name,
            include_dirs,
            source_paths,
            defines,
            pre.take(),
        )?;
        rss_trace("parse+elaborate");
        let phases = std::env::var_os("XEZIM_COMPILE_PHASES").is_some();
        let mut phase_t = WallTimer::now();
        let mut phase = |label: &str| {
            if phases {
                eprintln!(
                    "[ELAB-PHASE] {}: {:.1}ms",
                    label,
                    phase_t.elapsed().as_secs_f64() * 1000.0
                );
                phase_t = WallTimer::now();
            }
        };
        if std::env::var_os("XEZIM_MEM_CENSUS").is_some() {
            elab_census(&elab);
        }

        // §18.5.1: recover any out-of-class constraint body that the class-table
        // repopulation in `inline_instantiations` dropped.
        reinstall_ooc_constraint_bodies(&sources, source_paths, include_dirs, defines, &mut elab);
        // The rewritten source texts are dead from here on (the elaborated
        // design keeps its own preprocessed copies for diagnostics); don't
        // carry a second copy of the whole design through simulation.
        drop(sources);

        // Second-pass `should_fail` lint (additive — reuses the elaboration above,
        // no extra cost; does not alter elaborate/simulate behavior). Rejecting
        // here makes `:type: simulation` should_fail tests exit non-zero too, not
        // just the `--compile` path.
        {
            let dv: Vec<&SourceDefinition> = definitions.values().collect();
            let lint = should_fail_lint::lint_should_fail(&dv, &elab);
            if !lint.is_empty() {
                return Err(lint.join("; "));
            }
        }

        // §6.21: static initializers calling simulation-time system functions
        // were const-folded to garbage by elaboration — re-issue them as time-0
        // static-init assignments before the AST is dropped (issue #26).
        defer_static_syscall_inits(&definitions, &mut elab);
        phase("constraint bodies, lint, static inits");

        // Drop the parsed AST before constructing runtime state, and hand
        // its pages back at once: the allocator would otherwise keep them
        // resident for its purge delay, right while `Simulator::new` makes
        // its largest fresh allocations, and peak RSS would carry both.
        drop(definitions);
        xezim_core::release_free_memory();
        rss_trace("parsed AST dropped");
        phase("parsed AST dropped");

        if let Some((config, key)) = cache.as_ref().zip(cache_key.as_deref()) {
            // Pending rewrite contexts are intentionally omitted from the
            // artifact format; materialize them before publishing a complete
            // elaborated design.
            elab.materialize_pending();
            // Fold the captured elaboration diagnostics into the artifact so a
            // later warm hit replays them instead of running silent.
            elab.elab_diagnostics = xezim_core::elab_diag_capture_take();
            write_design_cache(config, key, &elab);
        }
        elab
    };

    let mut sim = compiler::Simulator::new(elab, max_time);
    rss_trace("simulator constructed");
    let project_vfs_enabled = virtual_files.is_some();
    if let Some((files, max_bytes)) = virtual_files {
        sim.set_virtual_files(files, max_bytes)?;
    }
    sim.set_output_capture_limit(output_capture_limit);
    sim.set_stdout_capture_only(project_vfs_enabled);
    if let Some(writer) = vcd_writer {
        sim.set_vcd_writer(writer)?;
    }
    if let Some((config, key)) = cache.as_ref().zip(cache_key.as_deref()) {
        sim.set_prepared_comb_cache_path(Some(config.directory.join(format!("{}.xezcomb", key))));
    }
    if let Some(limit) = settle_limit {
        sim.settle_limit = limit;
    }
    sim.activity_mon = activity_mon;
    sim.xtrace_file = xtrace_file.map(|s| s.to_string());
    sim.xtrace_scopes = xtrace_scopes.to_vec();
    sim.xtrace_from_ns = xtrace_from_ns;
    sim.xtrace_to_ns = xtrace_to_ns;
    sim.fst_file = fst_file.map(|s| s.to_string());
    sim.fst_scopes = fst_scopes.to_vec();
    sim.set_plusargs(plusargs);
    // Default argv for vpi_get_vlog_info — the real CLI passes the
    // full tokenized list via set_args() in main.rs. Here we hand
    // back just "xezim" + plusargs so UVM's tool banner works for
    // library users that never go through the binary.
    let mut argv: Vec<String> = vec!["xezim".to_string()];
    argv.extend(plusargs.iter().cloned());
    sim.set_args(&argv);

    // Stored regardless of --sdf: a runtime `$sdf_annotate` honors the
    // CLI-selected min/typ/max (or the +mindelays/+typdelays/+maxdelays
    // plusargs, which main.rs folds into sdf_select).
    sim.sdf_select = sdf_select;
    if let Some(sdf_path) = sdf_file {
        let sdf_content = std::fs::read_to_string(sdf_path)
            .map_err(|e| format!("Cannot read SDF file '{}': {}", sdf_path, e))?;
        let sdf = xezim_core::sdf::parse_sdf(&sdf_content)
            .map_err(|e| format!("SDF parse error in '{}': {}", sdf_path, e))?;
        let select = sdf_select.unwrap_or(xezim_core::sdf::DelaySelect::Typ);
        // SDF values scale to the simulation tick (the finest precision in
        // the design), like every other delay.
        let annotation = xezim_core::sdf::annotate_sdf(&sdf, sim.tick_s, select);
        sim.sdf_annotation = Some(annotation);
    }
    if force_wave {
        sim.enable_wave_support();
    }
    sim.compile();
    rss_trace("compiled");
    // A compile-time failure (e.g. §6.18 illegal non-class to class-handle
    // assignment) aborts before any block is scheduled; surface it as `Err` so
    // library callers see the compile error instead of a bogus run.
    if !sim.compile_errors.is_empty() {
        return Err(sim.compile_errors.join("; "));
    }
    chatter!(
        "[PHASE] compilation: {:.1}ms",
        compilation_start.elapsed().as_secs_f64() * 1000.0
    );
    // Startup-cost measurement: stop once the design is ready to simulate,
    // before any time-0 process runs.
    if std::env::var_os("XEZIM_EXIT_AFTER_COMPILE").is_some() {
        std::process::exit(0);
    }

    if let Some(path) = emit_hypergraph {
        let t = crate::time::Instant::now();
        // Phase-2 profile-guided emission: --profile-input takes
        // precedence; falls back to static weights without it.
        let prof = if let Some(pp) = profile_input {
            match compiler::simulator::Phase2Profile::load_from_file(pp) {
                Ok(p) => {
                    eprintln!(
                        "[PART] using profile {} ({} blocks, {} signals)",
                        pp,
                        p.edge_block_exec_ns.len(),
                        p.signal_toggle_count.len()
                    );
                    Some(p)
                }
                Err(e) => {
                    eprintln!(
                        "[PART] failed to load profile {}: {} — falling back to static",
                        pp, e
                    );
                    None
                }
            }
        } else {
            None
        };
        // Phase-3 island analysis (optional). Computes which blocks
        // MUST be co-located across cores (async-reset cones, comb
        // SCCs) and collapses them into super-vertices in the emitted
        // hypergraph. Without --collapse-islands, every block is its
        // own vertex (Phase 1/2 behavior).
        let islands = if collapse_islands {
            Some(sim.compute_phase3_islands())
        } else {
            None
        };
        let result = sim.emit_edge_block_hypergraph_full(path, prof.as_ref(), islands.as_deref());
        match result {
            Ok((nv, ne)) => eprintln!(
                "[PART] hypergraph written to {} ({} vertices, {} hyperedges, weights={}, islands={}) in {:.1}ms",
                path,
                nv,
                ne,
                if prof.is_some() { "profile" } else { "static" },
                if islands.is_some() { "phase3" } else { "off" },
                t.elapsed().as_secs_f64() * 1000.0
            ),
            Err(e) => eprintln!("[PART] failed to write hypergraph to {}: {}", path, e),
        }
    }
    if let Some(path) = load_partition {
        let t = crate::time::Instant::now();
        match sim.load_partition_file(path) {
            Ok((n, parts)) => eprintln!(
                "[PART] loaded partition from {} ({} assignments, k={}) in {:.1}ms",
                path,
                n,
                parts,
                t.elapsed().as_secs_f64() * 1000.0
            ),
            Err(e) => eprintln!("[PART] failed to load partition from {}: {}", path, e),
        }
    }

    let simulation_start = WallTimer::now();
    sim.simulate();
    rss_trace("simulated");
    if std::env::var_os("XEZIM_MEM_CENSUS").is_some() {
        eprintln!(
            "[MEM-CENSUS] end of run: runtime name->value map {} entries; {}",
            sim.signals.len(),
            sim.class_heap_census()
        );
    }
    chatter!(
        "[PHASE] simulation: {:.1}ms",
        simulation_start.elapsed().as_secs_f64() * 1000.0
    );

    if let Some(path) = write_profile {
        let t = crate::time::Instant::now();
        match sim.write_phase2_profile(path) {
            Ok(()) => eprintln!(
                "[PART] profile written to {} in {:.1}ms (set XEZIM_EDGE_BLOCK_STATS=1 to populate)",
                path,
                t.elapsed().as_secs_f64() * 1000.0
            ),
            Err(e) => eprintln!("[PART] failed to write profile to {}: {}", path, e),
        }
    }

    let total_elapsed = total_start.elapsed();
    chatter!(
        "[PHASE] total: {:.1}ms",
        total_elapsed.as_secs_f64() * 1000.0
    );
    chatter!("------------------------------");
    // The result line itself is the CLI's to print, on stdout (main.rs). Printing
    // it here too put it on BOTH streams, so it appeared twice in any terminal
    // or merged log.
    Ok(sim)
}
