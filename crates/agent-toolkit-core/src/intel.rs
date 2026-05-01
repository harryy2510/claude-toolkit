use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub struct RepoIntel {
    pub summary_markdown: String,
    pub file_count: usize,
}

#[derive(Debug, Clone)]
struct RepoAnalysis {
    files: Vec<FileIntel>,
    package: Option<PackageInfo>,
    frameworks: Vec<String>,
    local_edges: Vec<ImportEdge>,
    reverse_import_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone)]
struct FileIntel {
    path: PathBuf,
    normalized_path: String,
    extension: String,
    line_count: usize,
    imports: Vec<String>,
    exports: Vec<String>,
    components: Vec<ComponentIntel>,
    routes: Vec<RouteIntel>,
    api_endpoints: Vec<ApiEndpoint>,
    sql_objects: Vec<SqlObject>,
    env_vars: Vec<String>,
    is_test: bool,
    is_generated: bool,
}

#[derive(Debug, Clone)]
struct PackageInfo {
    scripts: Vec<String>,
    dependencies: Vec<String>,
    dev_dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
struct ComponentIntel {
    name: String,
    props: Vec<String>,
}

#[derive(Debug, Clone)]
struct RouteIntel {
    framework: String,
    route: String,
    kind: String,
}

#[derive(Debug, Clone)]
struct ApiEndpoint {
    method: String,
    route: String,
}

#[derive(Debug, Clone)]
struct SqlObject {
    kind: String,
    name: String,
}

#[derive(Debug, Clone)]
struct ImportEdge {
    from: String,
    to: String,
}

pub fn write_repo_intel(root: &Path) -> std::io::Result<RepoIntel> {
    let intel = build_repo_intel(root)?;
    let analysis = analyze_repo(root)?;
    let intel_dir = root.join(".agents/intel");
    fs::create_dir_all(&intel_dir)?;

    let articles = render_articles(&analysis);
    for (filename, contents) in articles {
        fs::write(intel_dir.join(filename), contents)?;
    }
    fs::write(intel_dir.join("summary.md"), &intel.summary_markdown)?;
    fs::write(intel_dir.join("repo.json"), render_repo_json(&analysis))?;

    Ok(intel)
}

pub fn build_repo_intel(root: &Path) -> std::io::Result<RepoIntel> {
    let analysis = analyze_repo(root)?;
    let summary_markdown = render_index(&analysis);

    Ok(RepoIntel {
        summary_markdown,
        file_count: analysis.files.len(),
    })
}

fn analyze_repo(root: &Path) -> std::io::Result<RepoAnalysis> {
    let files = collect_source_files(root)?;
    let package = fs::read_to_string(root.join("package.json"))
        .ok()
        .map(|contents| parse_package_info(&contents));
    let mut analyzed = Vec::with_capacity(files.len());
    for file in files {
        let full_path = root.join(&file);
        let contents = fs::read_to_string(&full_path).unwrap_or_default();
        let normalized_path = normalize_path(&file);
        let extension = file
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("")
            .to_string();
        let exports = extract_exports(&contents, &extension);
        let components = extract_components(&contents, &extension);
        let routes = detect_routes(&normalized_path, &contents);
        let api_endpoints = detect_api_endpoints(&normalized_path, &contents);
        let sql_objects = extract_sql_objects(&contents, &extension);
        let env_vars = extract_env_vars(&contents);

        analyzed.push(FileIntel {
            path: file,
            normalized_path: normalized_path.clone(),
            extension: extension.clone(),
            line_count: contents.lines().count(),
            imports: extract_imports(&contents, &extension),
            exports,
            components,
            routes,
            api_endpoints,
            sql_objects,
            env_vars,
            is_test: is_test_file(&normalized_path),
            is_generated: is_generated_file(&normalized_path),
        });
    }

    analyzed.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));

    let local_edges = build_local_edges(&analyzed);
    let mut reverse_import_counts: BTreeMap<String, usize> = BTreeMap::new();
    for edge in &local_edges {
        *reverse_import_counts.entry(edge.to.clone()).or_default() += 1;
    }
    let frameworks = detect_frameworks(root, package.as_ref(), &analyzed);

    Ok(RepoAnalysis {
        files: analyzed,
        package,
        frameworks,
        local_edges,
        reverse_import_counts,
    })
}

fn render_articles(analysis: &RepoAnalysis) -> Vec<(&'static str, String)> {
    vec![
        ("index.md", render_index(analysis)),
        ("overview.md", render_overview(analysis)),
        ("tasks.md", render_tasks(analysis)),
        ("tooling.md", render_tooling(analysis)),
        ("routes.md", render_routes(analysis)),
        ("api.md", render_api(analysis)),
        ("components.md", render_components(analysis)),
        ("data.md", render_data(analysis)),
        ("graph.md", render_graph(analysis)),
        ("dependencies.md", render_dependencies(analysis)),
        ("symbols.md", render_symbols(analysis)),
        ("files.md", render_files(analysis)),
        ("env.md", render_env(analysis)),
        ("testing.md", render_testing(analysis)),
    ]
}

fn render_index(analysis: &RepoAnalysis) -> String {
    let mut output = String::from("# Repository Intelligence\n\n");
    output.push_str("## Preferred Context\n\n");
    output.push_str("- Use this generated repo intelligence wiki first, then read the source files it points to.\n");
    output.push_str("- This index is deterministic and local-only. It is a map, not a substitute for reading implementation before editing.\n\n");

    output.push_str("## Articles\n\n");
    for (file, description) in [
        ("overview.md", "architecture, scale, high-impact files"),
        (
            "tasks.md",
            "task-oriented read paths so agents know where to start",
        ),
        ("tooling.md", "scripts, configs, package dependencies"),
        ("routes.md", "framework route files and route-like modules"),
        (
            "api.md",
            "API handlers, server functions, endpoint declarations",
        ),
        ("components.md", "UI components and prop surfaces"),
        ("data.md", "SQL schema, migrations, Supabase/data files"),
        ("graph.md", "import graph, blast radius, central modules"),
        ("dependencies.md", "external imports and package usage"),
        ("symbols.md", "exported symbols by source file"),
        ("files.md", "full source-like file inventory with tags"),
        ("env.md", "environment variable usage by file"),
        ("testing.md", "tests, coverage signals, test-adjacent files"),
    ] {
        output.push_str(&format!("- [`{file}`](./{file}) — {description}\n"));
    }

    output.push_str("\n## Quick Stats\n\n");
    output.push_str(&format!("- Source-like files: {}\n", analysis.files.len()));
    output.push_str(&format!(
        "- Frameworks/signals: {}\n",
        display_or_none(&analysis.frameworks)
    ));
    output.push_str(&format!("- Routes: {}\n", count_routes(analysis)));
    output.push_str(&format!(
        "- API endpoints/modules: {}\n",
        count_api(analysis)
    ));
    output.push_str(&format!("- Components: {}\n", count_components(analysis)));
    output.push_str(&format!("- SQL objects: {}\n", count_sql_objects(analysis)));
    output.push_str(&format!("- Env vars: {}\n", env_var_index(analysis).len()));
    output.push_str(&format!(
        "- Local import edges: {}\n\n",
        analysis.local_edges.len()
    ));

    push_read_first(&mut output);
    output
}

fn render_overview(analysis: &RepoAnalysis) -> String {
    let mut output = String::from("# Overview\n\n");
    output.push_str("## Stack Signals\n\n");
    for framework in &analysis.frameworks {
        output.push_str(&format!("- {framework}\n"));
    }
    if analysis.frameworks.is_empty() {
        output.push_str("- No framework signals detected from package metadata or file layout.\n");
    }

    output.push_str("\n## Scale\n\n");
    output.push_str(&format!("- Source-like files: {}\n", analysis.files.len()));
    output.push_str(&format!(
        "- UI components: {}\n",
        count_components(analysis)
    ));
    output.push_str(&format!("- Routes: {}\n", count_routes(analysis)));
    output.push_str(&format!(
        "- API endpoints/modules: {}\n",
        count_api(analysis)
    ));
    output.push_str(&format!("- SQL objects: {}\n", count_sql_objects(analysis)));
    output.push_str(&format!("- Tests: {}\n", count_tests(analysis)));

    output.push_str("\n## Source Areas\n\n");
    for (area, count) in top_areas(analysis, 20) {
        output.push_str(&format!("- `{area}`: {count}\n"));
    }

    output.push_str("\n## High-Impact Files\n\n");
    for (path, count) in top_reverse_imports(analysis, 20) {
        output.push_str(&format!("- `{path}` — imported by {count} local files\n"));
    }

    output.push_str("\n## Largest Non-Generated Files\n\n");
    for file in largest_files(analysis, 20) {
        output.push_str(&format!(
            "- `{}` — {} lines\n",
            file.normalized_path, file.line_count
        ));
    }

    output.push_str("\n## Exported Symbols Sample\n\n");
    let mut exported = 0usize;
    for file in analysis
        .files
        .iter()
        .filter(|file| !file.exports.is_empty())
        .take(30)
    {
        output.push_str(&format!(
            "- `{}` — {}\n",
            file.normalized_path,
            file.exports.join(", ")
        ));
        exported += 1;
    }
    if exported == 0 {
        output.push_str("- No exported symbols detected.\n");
    }

    let generated: Vec<&FileIntel> = analysis
        .files
        .iter()
        .filter(|file| file.is_generated)
        .take(20)
        .collect();
    if !generated.is_empty() {
        output.push_str("\n## Generated Files Detected\n\n");
        for file in generated {
            output.push_str(&format!("- `{}`\n", file.normalized_path));
        }
    }

    output
}

fn render_tasks(analysis: &RepoAnalysis) -> String {
    let mut output = String::from("# Task Read Paths\n\n");
    output.push_str("Use this file to avoid broad project scans. Pick the task type, read the listed intel article, then read the source files named there.\n\n");

    output.push_str("## Universal Start\n\n");
    output.push_str("- Read `overview.md` for stack, scale, high-impact files, generated files, and large modules.\n");
    output.push_str(
        "- Read `tooling.md` before running commands or changing package/config surfaces.\n",
    );
    output.push_str("- Read `graph.md` before touching a high-impact shared module.\n\n");

    output.push_str("## By Task Type\n\n");
    for (task, article, reason) in [
        (
            "UI component or screen work",
            "components.md, routes.md, graph.md",
            "Find component ownership, route entrypoints, props, and shared UI dependencies.",
        ),
        (
            "Route behavior or navigation",
            "routes.md, api.md, graph.md",
            "Find route files, server/API dependencies, and shared modules used by the route.",
        ),
        (
            "API/server function work",
            "api.md, data.md, env.md, graph.md",
            "Find server modules, schema/data dependencies, env requirements, and blast radius.",
        ),
        (
            "Database or migration work",
            "data.md, env.md, testing.md",
            "Find SQL objects, migration files, data helpers, and test scripts.",
        ),
        (
            "Auth/config/secrets work",
            "env.md, tooling.md, api.md",
            "Find env names and files that reference them without exposing values.",
        ),
        (
            "Refactor/shared helper work",
            "graph.md, symbols.md, files.md",
            "Find import fan-in/fan-out, exported symbols, and all files in the affected area.",
        ),
        (
            "Test work",
            "testing.md, files.md, graph.md",
            "Find existing tests, test scripts, and nearby source/test boundaries.",
        ),
    ] {
        output.push_str(&format!("- **{task}**: `{article}` — {reason}\n"));
    }

    output.push_str("\n## Highest Blast-Radius Files\n\n");
    for (path, count) in top_reverse_imports(analysis, 12) {
        output.push_str(&format!(
            "- `{path}` — read before changing; imported by {count} local files\n"
        ));
    }

    output
}

fn render_tooling(analysis: &RepoAnalysis) -> String {
    let mut output = String::from("# Tooling\n\n");
    if let Some(package) = &analysis.package {
        if !package.scripts.is_empty() {
            output.push_str("## Package Scripts\n\n");
            for script in &package.scripts {
                output.push_str(&format!("- `bun {script}`\n"));
            }
            output.push('\n');
        }
        if !package.dependencies.is_empty() {
            output.push_str("## Dependencies\n\n");
            for dependency in package.dependencies.iter().take(80) {
                output.push_str(&format!("- `{dependency}`\n"));
            }
            output.push('\n');
        }
        if !package.dev_dependencies.is_empty() {
            output.push_str("## Dev Dependencies\n\n");
            for dependency in package.dev_dependencies.iter().take(80) {
                output.push_str(&format!("- `{dependency}`\n"));
            }
            output.push('\n');
        }
    } else {
        output.push_str("No `package.json` detected.\n\n");
    }

    output.push_str("## Config Files\n\n");
    for file in analysis.files.iter().filter(|file| is_config_file(file)) {
        output.push_str(&format!("- `{}`\n", file.normalized_path));
    }

    output
}

fn render_routes(analysis: &RepoAnalysis) -> String {
    let mut output = String::from("# Routes\n\n");
    let mut routes: Vec<(&FileIntel, &RouteIntel)> = analysis
        .files
        .iter()
        .flat_map(|file| file.routes.iter().map(move |route| (file, route)))
        .collect();
    routes.sort_by(|left, right| {
        left.1
            .route
            .cmp(&right.1.route)
            .then_with(|| left.0.normalized_path.cmp(&right.0.normalized_path))
    });

    if routes.is_empty() {
        output.push_str("No framework route files detected.\n");
        return output;
    }

    for (file, route) in routes {
        output.push_str(&format!(
            "- `{}` — {} `{}` ({})\n",
            file.normalized_path, route.framework, route.route, route.kind
        ));
    }
    output
}

fn render_api(analysis: &RepoAnalysis) -> String {
    let mut output = String::from("# API And Server Surface\n\n");
    let mut endpoint_count = 0usize;
    for file in &analysis.files {
        if file.api_endpoints.is_empty() && !is_api_module(&file.normalized_path) {
            continue;
        }
        output.push_str(&format!("- `{}`", file.normalized_path));
        if !file.api_endpoints.is_empty() {
            let endpoints: Vec<String> = file
                .api_endpoints
                .iter()
                .map(|endpoint| format!("{} {}", endpoint.method, endpoint.route))
                .collect();
            output.push_str(&format!(" — {}", endpoints.join(", ")));
            endpoint_count += endpoints.len();
        }
        output.push('\n');
    }

    if endpoint_count == 0
        && !analysis
            .files
            .iter()
            .any(|file| is_api_module(&file.normalized_path))
    {
        output.push_str("No API handlers or API-like modules detected.\n");
    }
    output
}

fn render_components(analysis: &RepoAnalysis) -> String {
    let mut output = String::from("# Components\n\n");
    let mut components: Vec<(&FileIntel, &ComponentIntel)> = analysis
        .files
        .iter()
        .flat_map(|file| {
            file.components
                .iter()
                .map(move |component| (file, component))
        })
        .collect();
    components.sort_by(|left, right| {
        left.1
            .name
            .cmp(&right.1.name)
            .then_with(|| left.0.normalized_path.cmp(&right.0.normalized_path))
    });

    if components.is_empty() {
        output.push_str("No UI component exports detected.\n");
        return output;
    }

    for (file, component) in components {
        if component.props.is_empty() {
            output.push_str(&format!(
                "- **{}** — `{}`\n",
                component.name, file.normalized_path
            ));
        } else {
            output.push_str(&format!(
                "- **{}** — props: {} — `{}`\n",
                component.name,
                component.props.join(", "),
                file.normalized_path
            ));
        }
    }
    output
}

fn render_data(analysis: &RepoAnalysis) -> String {
    let mut output = String::from("# Data And Database\n\n");
    let mut sql_files = 0usize;
    for file in &analysis.files {
        if file.extension != "sql" && !is_data_file(&file.normalized_path) {
            continue;
        }
        if file.extension == "sql" {
            sql_files += 1;
        }
        output.push_str(&format!("- `{}`", file.normalized_path));
        if !file.sql_objects.is_empty() {
            let objects: Vec<String> = file
                .sql_objects
                .iter()
                .take(8)
                .map(|object| format!("{} {}", object.kind, object.name))
                .collect();
            output.push_str(&format!(" — {}", objects.join(", ")));
        }
        output.push('\n');
    }
    if sql_files == 0 {
        output.push_str("No SQL files detected.\n");
    }
    output
}

fn render_graph(analysis: &RepoAnalysis) -> String {
    let mut output = String::from("# Import Graph\n\n");
    output.push_str("## High-Impact Files\n\n");
    for (path, count) in top_reverse_imports(analysis, 40) {
        output.push_str(&format!("- `{path}` — imported by {count} local files\n"));
    }

    output.push_str("\n## Import-Heavy Files\n\n");
    let mut import_heavy: Vec<&FileIntel> = analysis
        .files
        .iter()
        .filter(|file| !file.imports.is_empty())
        .collect();
    import_heavy.sort_by(|left, right| {
        right
            .imports
            .len()
            .cmp(&left.imports.len())
            .then_with(|| left.normalized_path.cmp(&right.normalized_path))
    });
    for file in import_heavy.into_iter().take(40) {
        output.push_str(&format!(
            "- `{}` — {} imports\n",
            file.normalized_path,
            file.imports.len()
        ));
    }

    output.push_str("\n## Local Edges Sample\n\n");
    for edge in analysis.local_edges.iter().take(120) {
        output.push_str(&format!("- `{}` -> `{}`\n", edge.from, edge.to));
    }
    output
}

fn render_dependencies(analysis: &RepoAnalysis) -> String {
    let mut output = String::from("# Dependencies\n\n");
    if let Some(package) = &analysis.package {
        output.push_str("## Package Manifest\n\n");
        if !package.dependencies.is_empty() {
            output.push_str("### Dependencies\n\n");
            for dependency in &package.dependencies {
                output.push_str(&format!("- `{dependency}`\n"));
            }
            output.push('\n');
        }
        if !package.dev_dependencies.is_empty() {
            output.push_str("### Dev Dependencies\n\n");
            for dependency in &package.dev_dependencies {
                output.push_str(&format!("- `{dependency}`\n"));
            }
            output.push('\n');
        }
    }

    output.push_str("## External Import Usage\n\n");
    let external = external_import_usage(analysis);
    if external.is_empty() {
        output.push_str("No external imports detected.\n");
        return output;
    }

    for (package, files) in external.into_iter().take(80) {
        output.push_str(&format!("- `{package}` — used by {} files\n", files.len()));
        for file in files.iter().take(8) {
            output.push_str(&format!("  - `{file}`\n"));
        }
    }

    output
}

fn render_symbols(analysis: &RepoAnalysis) -> String {
    let mut output = String::from("# Exported Symbols\n\n");
    let mut files: Vec<&FileIntel> = analysis
        .files
        .iter()
        .filter(|file| !file.exports.is_empty())
        .collect();
    files.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));

    if files.is_empty() {
        output.push_str("No exported symbols detected.\n");
        return output;
    }

    for file in files {
        output.push_str(&format!(
            "- `{}` — {}\n",
            file.normalized_path,
            file.exports.join(", ")
        ));
    }
    output
}

fn render_files(analysis: &RepoAnalysis) -> String {
    let mut output = String::from("# File Inventory\n\n");
    output.push_str("Every source-like file discovered by the repo intelligence scanner. Tags are heuristic and meant to guide read paths.\n\n");

    let mut by_area: BTreeMap<String, Vec<&FileIntel>> = BTreeMap::new();
    for file in &analysis.files {
        by_area.entry(area_key(&file.path)).or_default().push(file);
    }

    for (area, mut files) in by_area {
        files.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));
        output.push_str(&format!("## `{area}`\n\n"));
        for file in files {
            output.push_str(&format!(
                "- `{}` — {} lines{}\n",
                file.normalized_path,
                file.line_count,
                display_file_tags(file)
            ));
        }
        output.push('\n');
    }

    output
}

fn render_env(analysis: &RepoAnalysis) -> String {
    let mut output = String::from("# Environment Variables\n\n");
    let env = env_var_index(analysis);
    if env.is_empty() {
        output.push_str("No environment variable names detected in source.\n");
        return output;
    }

    for (var, files) in env {
        output.push_str(&format!("- `{var}`\n"));
        for file in files.iter().take(12) {
            output.push_str(&format!("  - `{file}`\n"));
        }
    }
    output
}

fn render_testing(analysis: &RepoAnalysis) -> String {
    let mut output = String::from("# Testing\n\n");
    let tests: Vec<&FileIntel> = analysis.files.iter().filter(|file| file.is_test).collect();
    if tests.is_empty() {
        output.push_str("No test files detected.\n");
    } else {
        output.push_str(&format!("Detected {} test files.\n\n", tests.len()));
        for file in tests {
            output.push_str(&format!("- `{}`\n", file.normalized_path));
        }
    }

    output.push_str("\n## Test Scripts\n\n");
    if let Some(package) = &analysis.package {
        for script in package.scripts.iter().filter(|script| {
            script.contains("test")
                || script.contains("check")
                || script.contains("lint")
                || script.contains("format")
                || script.contains("type")
        }) {
            output.push_str(&format!("- `bun {script}`\n"));
        }
    }
    output
}

fn render_repo_json(analysis: &RepoAnalysis) -> String {
    let mut output = String::from("{\n");
    output.push_str("\t\"schemaVersion\": 3,\n");
    output.push_str(&format!("\t\"fileCount\": {},\n", analysis.files.len()));
    output.push_str(&format!(
        "\t\"frameworks\": {},\n",
        json_string_array(&analysis.frameworks)
    ));
    output.push_str(&format!("\t\"routeCount\": {},\n", count_routes(analysis)));
    output.push_str(&format!(
        "\t\"componentCount\": {},\n",
        count_components(analysis)
    ));
    output.push_str(&format!("\t\"apiCount\": {},\n", count_api(analysis)));
    output.push_str(&format!(
        "\t\"sqlObjectCount\": {},\n",
        count_sql_objects(analysis)
    ));
    output.push_str(&format!(
        "\t\"envVarCount\": {},\n",
        env_var_index(analysis).len()
    ));
    output.push_str(&format!(
        "\t\"localImportEdgeCount\": {},\n",
        analysis.local_edges.len()
    ));
    output.push_str(&format!(
        "\t\"packageScripts\": {},\n",
        json_string_array(
            &analysis
                .package
                .as_ref()
                .map(|package| package.scripts.clone())
                .unwrap_or_default()
        )
    ));
    output.push_str(&format!(
        "\t\"topAreas\": {},\n",
        render_json_pairs(top_areas(analysis, 40), "area", "fileCount")
    ));
    output.push_str(&format!(
        "\t\"highImpactFiles\": {},\n",
        render_json_pairs(top_reverse_imports(analysis, 80), "path", "importedBy")
    ));
    output.push_str(&format!(
        "\t\"generatedFiles\": {},\n",
        json_string_array(
            &analysis
                .files
                .iter()
                .filter(|file| file.is_generated)
                .map(|file| file.normalized_path.clone())
                .collect::<Vec<_>>()
        )
    ));
    output.push_str(&format!(
        "\t\"testFiles\": {},\n",
        json_string_array(
            &analysis
                .files
                .iter()
                .filter(|file| file.is_test)
                .map(|file| file.normalized_path.clone())
                .collect::<Vec<_>>()
        )
    ));
    output.push_str(&format!(
        "\t\"envVars\": {},\n",
        json_string_array(&env_var_index(analysis).keys().cloned().collect::<Vec<_>>())
    ));
    output.push_str(&format!(
        "\t\"routes\": {},\n",
        render_routes_json(analysis, 200)
    ));
    output.push_str(&format!(
        "\t\"apiModules\": {},\n",
        render_api_json(analysis, 200)
    ));
    output.push_str(&format!(
        "\t\"components\": {},\n",
        render_components_json(analysis, 300)
    ));
    output.push_str(&format!(
        "\t\"sqlObjects\": {},\n",
        render_sql_json(analysis, 300)
    ));
    output.push_str(&format!(
        "\t\"files\": {}\n",
        render_files_json(analysis, 1000)
    ));
    output.push_str("}\n");
    output
}

fn collect_source_files(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();

            if entry_path.is_dir() {
                if should_skip_dir(&name) {
                    continue;
                }
                stack.push(entry_path);
                continue;
            }

            if is_source_file(&entry_path) {
                files.push(
                    entry_path
                        .strip_prefix(root)
                        .unwrap_or(&entry_path)
                        .to_path_buf(),
                );
            }
        }
    }

    files.sort();
    Ok(files)
}

fn should_skip_dir(name: &str) -> bool {
    if name.starts_with('.') && name != ".github" {
        return true;
    }

    matches!(
        name,
        ".git"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | ".next"
            | ".nuxt"
            | ".svelte-kit"
            | ".turbo"
            | ".cache"
            | ".wrangler"
    )
}

fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some(
            "ts" | "tsx"
                | "js"
                | "jsx"
                | "mjs"
                | "cjs"
                | "mts"
                | "cts"
                | "vue"
                | "svelte"
                | "astro"
                | "rs"
                | "go"
                | "py"
                | "sql"
                | "md"
                | "json"
                | "toml"
                | "yml"
                | "yaml"
        )
    )
}

fn parse_package_info(contents: &str) -> PackageInfo {
    PackageInfo {
        scripts: filter_external_intel_tooling(extract_object_keys(contents, "scripts")),
        dependencies: filter_external_intel_tooling(extract_object_keys(contents, "dependencies")),
        dev_dependencies: filter_external_intel_tooling(extract_object_keys(
            contents,
            "devDependencies",
        )),
    }
}

fn filter_external_intel_tooling(items: Vec<String>) -> Vec<String> {
    items
        .into_iter()
        .filter(|item| {
            let lower = item.to_ascii_lowercase();
            !(lower.contains("code") && lower.contains("sight"))
        })
        .collect()
}

fn detect_frameworks(
    root: &Path,
    package: Option<&PackageInfo>,
    files: &[FileIntel],
) -> Vec<String> {
    let mut frameworks = BTreeSet::new();
    let deps: BTreeSet<&str> = package
        .map(|package| {
            package
                .dependencies
                .iter()
                .chain(package.dev_dependencies.iter())
                .map(String::as_str)
                .collect()
        })
        .unwrap_or_default();

    for (dependency, framework) in [
        ("next", "Next.js"),
        ("@tanstack/react-start", "TanStack Start"),
        ("@tanstack/react-router", "TanStack Router"),
        ("@remix-run/react", "Remix"),
        ("astro", "Astro"),
        ("@sveltejs/kit", "SvelteKit"),
        ("nuxt", "Nuxt"),
        ("vue", "Vue"),
        ("react", "React"),
        ("solid-js", "Solid"),
        ("@angular/core", "Angular"),
        ("vite", "Vite"),
        ("hono", "Hono"),
        ("express", "Express"),
        ("fastify", "Fastify"),
        ("@trpc/server", "tRPC"),
        ("@supabase/supabase-js", "Supabase"),
        ("prisma", "Prisma"),
        ("drizzle-orm", "Drizzle"),
        ("wrangler", "Cloudflare Workers"),
    ] {
        if deps.contains(dependency) {
            frameworks.insert(framework.to_string());
        }
    }

    for (path, framework) in [
        ("next.config.js", "Next.js"),
        ("next.config.ts", "Next.js"),
        ("remix.config.js", "Remix"),
        ("astro.config.mjs", "Astro"),
        ("svelte.config.js", "SvelteKit"),
        ("nuxt.config.ts", "Nuxt"),
        ("vite.config.ts", "Vite"),
        ("wrangler.toml", "Cloudflare Workers"),
        ("supabase/config.toml", "Supabase"),
    ] {
        if root.join(path).exists() {
            frameworks.insert(framework.to_string());
        }
    }

    if files
        .iter()
        .any(|file| file.normalized_path.starts_with("src/routes/"))
    {
        frameworks.insert("File-based routes".to_string());
    }

    frameworks.into_iter().collect()
}

fn extract_imports(contents: &str, extension: &str) -> Vec<String> {
    let mut imports = Vec::new();
    match extension {
        "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "mts" | "cts" => {
            for line in contents.lines() {
                let line = line.trim();
                if line.starts_with("import ")
                    || line.starts_with("export ") && line.contains(" from ")
                {
                    if let Some(specifier) = extract_quoted_specifier(line) {
                        imports.push(specifier);
                    }
                }
                if line.contains("import(") {
                    if let Some(specifier) = extract_quoted_specifier(line) {
                        imports.push(specifier);
                    }
                }
            }
        }
        "rs" => {
            for line in contents.lines() {
                let line = line.trim();
                if let Some(rest) = line.strip_prefix("use ") {
                    imports.push(rest.trim_end_matches(';').to_string());
                }
            }
        }
        _ => {}
    }
    imports.sort();
    imports.dedup();
    imports
}

fn extract_exports(contents: &str, extension: &str) -> Vec<String> {
    let mut exports = Vec::new();
    for line in contents.lines() {
        let line = line.trim_start();
        match extension {
            "ts" | "tsx" | "js" | "jsx" | "mjs" | "mts" | "cts" | "vue" | "svelte" | "astro" => {
                extract_typescript_export(line, &mut exports)
            }
            "rs" => extract_rust_export(line, &mut exports),
            _ => {}
        }
        if exports.len() >= 16 {
            break;
        }
    }
    exports
}

fn extract_components(contents: &str, extension: &str) -> Vec<ComponentIntel> {
    if !matches!(extension, "tsx" | "jsx" | "vue" | "svelte" | "astro") {
        return Vec::new();
    }

    let props_by_type = extract_props_by_type(contents);
    let mut components = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim_start();
        for prefix in ["export function ", "function "] {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                let name = read_symbol(rest);
                if is_component_name(&name) {
                    components.push(ComponentIntel {
                        props: component_props(&name, rest, &props_by_type),
                        name,
                    });
                }
            }
        }
        for prefix in ["export const ", "const "] {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                let name = read_symbol(rest);
                if is_component_name(&name)
                    && (rest.contains("=>")
                        || rest.contains("forwardRef")
                        || rest.contains("memo(")
                        || rest.contains("React."))
                {
                    components.push(ComponentIntel {
                        props: component_props(&name, rest, &props_by_type),
                        name,
                    });
                }
            }
        }
    }

    components.sort_by(|left, right| left.name.cmp(&right.name));
    components.dedup_by(|left, right| left.name == right.name);
    components
}

fn detect_routes(path: &str, contents: &str) -> Vec<RouteIntel> {
    let mut routes = Vec::new();

    if path.starts_with("src/routes/") || path.starts_with("app/routes/") {
        routes.push(RouteIntel {
            framework: if path.starts_with("src/routes/") {
                "TanStack/File Routes".to_string()
            } else {
                "Remix".to_string()
            },
            route: route_from_file_path(path),
            kind: "route file".to_string(),
        });
    }
    if path.starts_with("app/") && path.ends_with("/page.tsx") {
        routes.push(RouteIntel {
            framework: "Next.js App Router".to_string(),
            route: next_app_route(path, "/page.tsx"),
            kind: "page".to_string(),
        });
    }
    if path.starts_with("app/") && path.ends_with("/route.ts") {
        routes.push(RouteIntel {
            framework: "Next.js App Router".to_string(),
            route: next_app_route(path, "/route.ts"),
            kind: "route handler".to_string(),
        });
    }
    if path.starts_with("pages/") && matches_route_extension(path) {
        routes.push(RouteIntel {
            framework: "Next.js Pages Router".to_string(),
            route: pages_route(path, "pages/"),
            kind: if path.starts_with("pages/api/") {
                "api route".to_string()
            } else {
                "page".to_string()
            },
        });
    }
    if path.starts_with("src/pages/") && matches_route_extension(path) {
        routes.push(RouteIntel {
            framework: "Astro".to_string(),
            route: pages_route(path, "src/pages/"),
            kind: "page".to_string(),
        });
    }
    if path.starts_with("src/routes/")
        && (path.ends_with("+page.svelte") || path.ends_with("+server.ts"))
    {
        routes.push(RouteIntel {
            framework: "SvelteKit".to_string(),
            route: sveltekit_route(path),
            kind: if path.ends_with("+server.ts") {
                "server route".to_string()
            } else {
                "page".to_string()
            },
        });
    }

    if contents.contains("createFileRoute(") || contents.contains("createRootRoute(") {
        let route = extract_route_literal(contents).unwrap_or_else(|| route_from_file_path(path));
        routes.push(RouteIntel {
            framework: "TanStack Router".to_string(),
            route,
            kind: "declared route".to_string(),
        });
    }

    routes
}

fn detect_api_endpoints(path: &str, contents: &str) -> Vec<ApiEndpoint> {
    let mut endpoints = Vec::new();
    for method in ["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS", "HEAD"] {
        if contents.contains(&format!("export const {method}"))
            || contents.contains(&format!("export async function {method}"))
            || contents.contains(&format!("export function {method}"))
        {
            endpoints.push(ApiEndpoint {
                method: method.to_string(),
                route: route_from_file_path(path),
            });
        }
    }

    for method in ["get", "post", "put", "patch", "delete", "options", "all"] {
        for line in contents.lines() {
            let line = line.trim();
            for prefix in [
                &format!("app.{method}("),
                &format!("router.{method}("),
                &format!("server.{method}("),
            ] {
                if line.contains(prefix) {
                    if let Some(route) = extract_quoted_specifier(line) {
                        endpoints.push(ApiEndpoint {
                            method: method.to_uppercase(),
                            route,
                        });
                    }
                }
            }
        }
    }

    if endpoints.is_empty() && is_api_module(path) {
        endpoints.push(ApiEndpoint {
            method: "MODULE".to_string(),
            route: path.to_string(),
        });
    }

    endpoints
}

fn extract_sql_objects(contents: &str, extension: &str) -> Vec<SqlObject> {
    if extension != "sql" {
        return Vec::new();
    }
    let mut objects = Vec::new();
    for line in contents.lines() {
        let line = line.trim();
        let lower = line.to_ascii_lowercase();
        for (needle, kind) in [
            ("create table", "table"),
            ("create or replace function", "function"),
            ("create function", "function"),
            ("create policy", "policy"),
            ("create index", "index"),
            ("create trigger", "trigger"),
            ("create view", "view"),
        ] {
            if lower.starts_with(needle) {
                let name = line
                    .split_whitespace()
                    .skip(needle.split_whitespace().count())
                    .find(|part| {
                        !matches!(part.to_ascii_lowercase().as_str(), "if" | "not" | "exists")
                    })
                    .unwrap_or("")
                    .trim_matches(|character: char| {
                        character == '"' || character == '(' || character == ';'
                    })
                    .to_string();
                if !name.is_empty() {
                    objects.push(SqlObject {
                        kind: kind.to_string(),
                        name,
                    });
                }
            }
        }
        if objects.len() >= 20 {
            break;
        }
    }
    objects
}

fn extract_env_vars(contents: &str) -> Vec<String> {
    let mut vars = BTreeSet::new();
    for needle in ["process.env.", "Bun.env.", "import.meta.env."] {
        for segment in contents.split(needle).skip(1) {
            let name = read_symbol(segment);
            if is_env_var_name(&name) {
                vars.insert(name);
            }
        }
    }

    for segment in contents.split("env(").skip(1) {
        if let Some(name) = extract_quoted_specifier(segment) {
            if is_env_var_name(&name) {
                vars.insert(name);
            }
        }
    }

    vars.into_iter().collect()
}

fn build_local_edges(files: &[FileIntel]) -> Vec<ImportEdge> {
    let file_set: HashSet<String> = files
        .iter()
        .map(|file| file.normalized_path.clone())
        .collect();
    let mut edges = Vec::new();

    for file in files {
        for import in &file.imports {
            if !(import.starts_with("./")
                || import.starts_with("../")
                || import.starts_with("@/")
                || import.starts_with("~/"))
            {
                continue;
            }
            if let Some(target) = resolve_local_import(&file.normalized_path, import, &file_set) {
                edges.push(ImportEdge {
                    from: file.normalized_path.clone(),
                    to: target,
                });
            }
        }
    }

    edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.to.cmp(&right.to))
    });
    edges.dedup_by(|left, right| left.from == right.from && left.to == right.to);
    edges
}

fn resolve_local_import(from: &str, import: &str, file_set: &HashSet<String>) -> Option<String> {
    let base = if import.starts_with("./") || import.starts_with("../") {
        let parent = from
            .rsplit_once('/')
            .map(|(parent, _)| parent)
            .unwrap_or("");
        normalize_virtual_path(&format!("{parent}/{import}"))
    } else if let Some(rest) = import.strip_prefix("@/") {
        normalize_virtual_path(&format!("src/{rest}"))
    } else if let Some(rest) = import.strip_prefix("~/") {
        normalize_virtual_path(&format!("src/{rest}"))
    } else {
        return None;
    };
    let candidates = [
        base.clone(),
        format!("{base}.ts"),
        format!("{base}.tsx"),
        format!("{base}.js"),
        format!("{base}.jsx"),
        format!("{base}.mjs"),
        format!("{base}.cjs"),
        format!("{base}.vue"),
        format!("{base}.svelte"),
        format!("{base}.astro"),
        format!("{base}/index.ts"),
        format!("{base}/index.tsx"),
        format!("{base}/index.js"),
        format!("{base}/index.jsx"),
    ];
    candidates
        .into_iter()
        .find(|candidate| file_set.contains(candidate))
}

fn extract_typescript_export(line: &str, exports: &mut Vec<String>) {
    let Some(rest) = line.strip_prefix("export ") else {
        return;
    };
    let rest = rest.strip_prefix("default ").unwrap_or(rest);
    let rest = rest.strip_prefix("async ").unwrap_or(rest);

    if let Some(named) = rest.strip_prefix('{') {
        if let Some((members, _)) = named.split_once('}') {
            for member in members.split(',').take(16 - exports.len()) {
                let name = member
                    .trim()
                    .split_once(" as ")
                    .map(|(_, alias)| alias)
                    .unwrap_or_else(|| member.trim());
                push_symbol(exports, name);
            }
        }
        return;
    }

    for prefix in [
        "function ",
        "const ",
        "let ",
        "var ",
        "class ",
        "interface ",
        "type ",
        "enum ",
    ] {
        if let Some(symbol) = rest.strip_prefix(prefix) {
            push_symbol(exports, symbol);
            return;
        }
    }
}

fn extract_rust_export(line: &str, exports: &mut Vec<String>) {
    let Some(rest) = line.strip_prefix("pub ") else {
        return;
    };
    for prefix in ["fn ", "struct ", "enum ", "trait ", "mod ", "const "] {
        if let Some(symbol) = rest.strip_prefix(prefix) {
            push_symbol(exports, symbol);
            return;
        }
    }
}

fn extract_props_by_type(contents: &str) -> HashMap<String, Vec<String>> {
    let mut props = HashMap::new();
    let lines: Vec<&str> = contents.lines().collect();
    let mut index = 0usize;
    while index < lines.len() {
        let line = lines[index].trim();
        let type_name = line
            .strip_prefix("type ")
            .or_else(|| line.strip_prefix("interface "))
            .map(read_symbol);
        let Some(type_name) = type_name else {
            index += 1;
            continue;
        };
        if !type_name.ends_with("Props") {
            index += 1;
            continue;
        }
        let mut fields = Vec::new();
        index += 1;
        while index < lines.len() {
            let field_line = lines[index].trim();
            if field_line.starts_with('}') || field_line.starts_with("};") {
                break;
            }
            if let Some((name, _)) = field_line.split_once(':') {
                let name = name.trim().trim_end_matches('?');
                if !name.is_empty()
                    && name
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '_')
                {
                    fields.push(name.to_string());
                }
            }
            index += 1;
        }
        props.insert(type_name, fields);
    }
    props
}

fn component_props(
    component_name: &str,
    declaration_tail: &str,
    props_by_type: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    let preferred = format!("{component_name}Props");
    if let Some(props) = props_by_type.get(&preferred) {
        return props.clone();
    }
    for (name, props) in props_by_type {
        if declaration_tail.contains(name) {
            return props.clone();
        }
    }
    Vec::new()
}

fn push_symbol(exports: &mut Vec<String>, raw: &str) {
    let symbol = read_symbol(raw);
    if !symbol.is_empty() && !exports.contains(&symbol) {
        exports.push(symbol);
    }
}

fn read_symbol(raw: &str) -> String {
    raw.trim_start()
        .chars()
        .take_while(|character| {
            character.is_ascii_alphanumeric() || *character == '_' || *character == '$'
        })
        .collect()
}

fn extract_quoted_specifier(line: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let Some((_, rest)) = line.split_once(quote) else {
            continue;
        };
        let Some((specifier, _)) = rest.split_once(quote) else {
            continue;
        };
        return Some(specifier.to_string());
    }
    None
}

fn extract_route_literal(contents: &str) -> Option<String> {
    for needle in ["createFileRoute(", "createRoute("] {
        if let Some(segment) = contents.split(needle).nth(1) {
            if let Some(route) = extract_quoted_specifier(segment) {
                return Some(route);
            }
        }
    }
    None
}

fn is_component_name(name: &str) -> bool {
    name.chars()
        .next()
        .map(|character| character.is_ascii_uppercase())
        .unwrap_or(false)
        && name.chars().any(|character| character.is_ascii_lowercase())
}

fn is_env_var_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
        })
}

fn is_test_file(path: &str) -> bool {
    path.contains("__tests__/")
        || path.contains(".test.")
        || path.contains(".spec.")
        || path.ends_with("_test.rs")
        || path.starts_with("test/")
        || path.starts_with("tests/")
}

fn is_generated_file(path: &str) -> bool {
    path.ends_with(".d.ts")
        || path.contains(".gen.")
        || path.contains("/generated")
        || path.ends_with("routeTree.gen.ts")
        || path.ends_with("worker-configuration.d.ts")
}

fn is_config_file(file: &FileIntel) -> bool {
    let path = file.normalized_path.as_str();
    path == "package.json"
        || path == "Cargo.toml"
        || path == "tsconfig.json"
        || path == "vite.config.ts"
        || path == "vitest.config.ts"
        || path == "next.config.ts"
        || path == "astro.config.mjs"
        || path == "svelte.config.js"
        || path == "nuxt.config.ts"
        || path == "wrangler.toml"
        || path == "components.json"
        || path.starts_with(".github/workflows/")
        || path.ends_with("config.toml")
        || path.ends_with("config.yml")
        || path.ends_with("config.yaml")
}

fn is_api_module(path: &str) -> bool {
    path.starts_with("src/api/")
        || path.starts_with("app/api/")
        || path.starts_with("pages/api/")
        || path.starts_with("server/api/")
        || path.starts_with("supabase/functions/")
        || path.contains("/server/")
        || path.ends_with("functions.ts")
        || path.ends_with("functions.tsx")
}

fn is_data_file(path: &str) -> bool {
    path.starts_with("supabase/")
        || path.starts_with("prisma/")
        || path.contains("schema")
        || path.contains("database")
        || path.contains("drizzle")
}

fn matches_route_extension(path: &str) -> bool {
    matches!(
        Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str()),
        Some("ts" | "tsx" | "js" | "jsx" | "vue" | "svelte" | "astro")
    )
}

fn route_from_file_path(path: &str) -> String {
    let path = path
        .trim_start_matches("src/routes/")
        .trim_start_matches("app/routes/")
        .trim_start_matches("routes/")
        .trim_start_matches("pages/")
        .trim_start_matches("src/pages/")
        .trim_start_matches("server/api/")
        .trim_start_matches("pages/api/")
        .trim_start_matches("app/api/");
    let path = strip_known_extension(path)
        .trim_end_matches("/index")
        .trim_end_matches(".index")
        .replace("_", "")
        .replace("$", ":");
    if path.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", path.replace('.', "/"))
    }
}

fn next_app_route(path: &str, suffix: &str) -> String {
    let route = path
        .trim_start_matches("app/")
        .trim_end_matches(suffix)
        .split('/')
        .filter(|segment| !segment.starts_with('('))
        .collect::<Vec<_>>()
        .join("/");
    if route.is_empty() {
        "/".to_string()
    } else {
        format!("/{route}")
    }
}

fn pages_route(path: &str, prefix: &str) -> String {
    let route = strip_known_extension(path.trim_start_matches(prefix))
        .trim_end_matches("/index")
        .replace("[...", ":")
        .replace("[[...", ":")
        .replace('[', ":")
        .replace([']', ')'], "");
    if route.is_empty() {
        "/".to_string()
    } else {
        format!("/{route}")
    }
}

fn sveltekit_route(path: &str) -> String {
    let route = path
        .trim_start_matches("src/routes/")
        .replace("/+page.svelte", "")
        .replace("/+server.ts", "");
    if route.is_empty() {
        "/".to_string()
    } else {
        format!("/{route}")
    }
}

fn strip_known_extension(path: &str) -> &str {
    for extension in [
        ".tsx", ".ts", ".jsx", ".js", ".mjs", ".cjs", ".vue", ".svelte", ".astro",
    ] {
        if let Some(stripped) = path.strip_suffix(extension) {
            return stripped;
        }
    }
    path
}

fn area_key(path: &Path) -> String {
    let parts: Vec<String> = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            _ => None,
        })
        .collect();
    if parts.len() <= 1 {
        return ".".to_string();
    }
    match parts[0].as_str() {
        "src" | "docs" | "supabase" | ".github" | "scripts" | "crates" | "test" | "tests"
        | "app" | "pages" | "server" => format!("{}/{}", parts[0], parts[1]),
        _ => parts[0].clone(),
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn normalize_virtual_path(path: &str) -> String {
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

fn extract_object_keys(contents: &str, key: &str) -> Vec<String> {
    let Some(key_index) = contents.find(&format!("\"{key}\"")) else {
        return Vec::new();
    };
    let Some(open_offset) = contents[key_index..].find('{') else {
        return Vec::new();
    };
    let chars: Vec<char> = contents[key_index + open_offset..].chars().collect();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut current_string = String::new();
    let mut keys = Vec::new();

    for (index, character) in chars.iter().enumerate() {
        if in_string {
            if escaped {
                current_string.push(*character);
                escaped = false;
                continue;
            }
            if *character == '\\' {
                escaped = true;
                continue;
            }
            if *character == '"' {
                in_string = false;
                if depth == 1 && next_non_whitespace(&chars, index + 1) == Some(':') {
                    keys.push(current_string.clone());
                }
                current_string.clear();
                continue;
            }
            current_string.push(*character);
            continue;
        }

        match character {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    break;
                }
            }
            '"' => in_string = true,
            _ => {}
        }
    }

    keys
}

fn next_non_whitespace(chars: &[char], start: usize) -> Option<char> {
    chars
        .iter()
        .skip(start)
        .copied()
        .find(|character| !character.is_whitespace())
}

fn env_var_index(analysis: &RepoAnalysis) -> BTreeMap<String, Vec<String>> {
    let mut env = BTreeMap::new();
    for file in &analysis.files {
        for var in &file.env_vars {
            env.entry(var.clone())
                .or_insert_with(Vec::new)
                .push(file.normalized_path.clone());
        }
    }
    env
}

fn external_import_usage(analysis: &RepoAnalysis) -> Vec<(String, Vec<String>)> {
    let mut usage: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for file in &analysis.files {
        for import in &file.imports {
            if import.starts_with("./")
                || import.starts_with("../")
                || import.starts_with("@/")
                || import.starts_with("~/")
            {
                continue;
            }
            let package = package_name_from_import(import);
            if package.is_empty() {
                continue;
            }
            usage
                .entry(package)
                .or_default()
                .insert(file.normalized_path.clone());
        }
    }

    let mut usage: Vec<(String, Vec<String>)> = usage
        .into_iter()
        .map(|(package, files)| (package, files.into_iter().collect()))
        .collect();
    usage.sort_by(|left, right| {
        right
            .1
            .len()
            .cmp(&left.1.len())
            .then_with(|| left.0.cmp(&right.0))
    });
    usage
}

fn package_name_from_import(import: &str) -> String {
    let mut parts = import.split('/');
    let Some(first) = parts.next() else {
        return String::new();
    };
    if first.starts_with('@') {
        let Some(second) = parts.next() else {
            return first.to_string();
        };
        format!("{first}/{second}")
    } else {
        first.to_string()
    }
}

fn display_file_tags(file: &FileIntel) -> String {
    let mut tags = Vec::new();
    if !file.routes.is_empty() {
        tags.push("route");
    }
    if !file.api_endpoints.is_empty() || is_api_module(&file.normalized_path) {
        tags.push("api");
    }
    if !file.components.is_empty() {
        tags.push("component");
    }
    if file.extension == "sql" || is_data_file(&file.normalized_path) {
        tags.push("data");
    }
    if file.is_test {
        tags.push("test");
    }
    if file.is_generated {
        tags.push("generated");
    }
    if !file.env_vars.is_empty() {
        tags.push("env");
    }
    if tags.is_empty() {
        String::new()
    } else {
        format!(" — tags: {}", tags.join(", "))
    }
}

fn top_areas(analysis: &RepoAnalysis, limit: usize) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for file in &analysis.files {
        *counts.entry(area_key(&file.path)).or_default() += 1;
    }
    let mut areas: Vec<(String, usize)> = counts.into_iter().collect();
    areas.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    areas.truncate(limit);
    areas
}

fn top_reverse_imports(analysis: &RepoAnalysis, limit: usize) -> Vec<(String, usize)> {
    let mut imports: Vec<(String, usize)> = analysis
        .reverse_import_counts
        .iter()
        .map(|(path, count)| (path.clone(), *count))
        .collect();
    imports.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    imports.truncate(limit);
    imports
}

fn largest_files(analysis: &RepoAnalysis, limit: usize) -> Vec<&FileIntel> {
    let mut files: Vec<&FileIntel> = analysis
        .files
        .iter()
        .filter(|file| !file.is_generated && file.line_count > 0)
        .collect();
    files.sort_by(|left, right| {
        right
            .line_count
            .cmp(&left.line_count)
            .then_with(|| left.normalized_path.cmp(&right.normalized_path))
    });
    files.truncate(limit);
    files
}

fn count_routes(analysis: &RepoAnalysis) -> usize {
    analysis.files.iter().map(|file| file.routes.len()).sum()
}

fn count_api(analysis: &RepoAnalysis) -> usize {
    analysis
        .files
        .iter()
        .filter(|file| !file.api_endpoints.is_empty() || is_api_module(&file.normalized_path))
        .count()
}

fn count_components(analysis: &RepoAnalysis) -> usize {
    analysis
        .files
        .iter()
        .map(|file| file.components.len())
        .sum()
}

fn count_sql_objects(analysis: &RepoAnalysis) -> usize {
    analysis
        .files
        .iter()
        .map(|file| file.sql_objects.len())
        .sum()
}

fn count_tests(analysis: &RepoAnalysis) -> usize {
    analysis.files.iter().filter(|file| file.is_test).count()
}

fn display_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "none detected".to_string()
    } else {
        items.join(", ")
    }
}

fn push_read_first(output: &mut String) {
    output.push_str("## How To Use\n\n");
    output
        .push_str("- Start with `overview.md`, then jump to the article that matches your task.\n");
    output.push_str("- For behavior changes, read the listed source files before editing.\n");
    output.push_str("- For broad refactors, inspect `graph.md` high-impact files first.\n");
    output.push_str("- For framework-specific work, prefer the matching route/API/component/data article over global search.\n");
}

fn json_string_array(items: &[String]) -> String {
    let parts: Vec<String> = items
        .iter()
        .map(|item| format!("\"{}\"", json_escape(item)))
        .collect();
    format!("[{}]", parts.join(", "))
}

fn render_json_pairs(items: Vec<(String, usize)>, key_name: &str, value_name: &str) -> String {
    let objects: Vec<String> = items
        .into_iter()
        .map(|(key, value)| {
            format!(
                "{{\"{key_name}\":\"{}\",\"{value_name}\":{value}}}",
                json_escape(&key)
            )
        })
        .collect();
    format!("[{}]", objects.join(", "))
}

fn render_routes_json(analysis: &RepoAnalysis, limit: usize) -> String {
    let mut items = Vec::new();
    for file in &analysis.files {
        for route in &file.routes {
            items.push(format!(
                "{{\"file\":\"{}\",\"framework\":\"{}\",\"route\":\"{}\",\"kind\":\"{}\"}}",
                json_escape(&file.normalized_path),
                json_escape(&route.framework),
                json_escape(&route.route),
                json_escape(&route.kind)
            ));
            if items.len() >= limit {
                return format!("[{}]", items.join(", "));
            }
        }
    }
    format!("[{}]", items.join(", "))
}

fn render_api_json(analysis: &RepoAnalysis, limit: usize) -> String {
    let mut items = Vec::new();
    for file in &analysis.files {
        if file.api_endpoints.is_empty() && !is_api_module(&file.normalized_path) {
            continue;
        }
        let endpoints: Vec<String> = file
            .api_endpoints
            .iter()
            .map(|endpoint| {
                format!(
                    "{{\"method\":\"{}\",\"route\":\"{}\"}}",
                    json_escape(&endpoint.method),
                    json_escape(&endpoint.route)
                )
            })
            .collect();
        items.push(format!(
            "{{\"file\":\"{}\",\"endpoints\":[{}]}}",
            json_escape(&file.normalized_path),
            endpoints.join(", ")
        ));
        if items.len() >= limit {
            break;
        }
    }
    format!("[{}]", items.join(", "))
}

fn render_components_json(analysis: &RepoAnalysis, limit: usize) -> String {
    let mut items = Vec::new();
    for file in &analysis.files {
        for component in &file.components {
            items.push(format!(
                "{{\"name\":\"{}\",\"file\":\"{}\",\"props\":{}}}",
                json_escape(&component.name),
                json_escape(&file.normalized_path),
                json_string_array(&component.props)
            ));
            if items.len() >= limit {
                return format!("[{}]", items.join(", "));
            }
        }
    }
    format!("[{}]", items.join(", "))
}

fn render_sql_json(analysis: &RepoAnalysis, limit: usize) -> String {
    let mut items = Vec::new();
    for file in &analysis.files {
        for object in &file.sql_objects {
            items.push(format!(
                "{{\"kind\":\"{}\",\"name\":\"{}\",\"file\":\"{}\"}}",
                json_escape(&object.kind),
                json_escape(&object.name),
                json_escape(&file.normalized_path)
            ));
            if items.len() >= limit {
                return format!("[{}]", items.join(", "));
            }
        }
    }
    format!("[{}]", items.join(", "))
}

fn render_files_json(analysis: &RepoAnalysis, limit: usize) -> String {
    let mut items = Vec::new();
    for file in analysis.files.iter().take(limit) {
        let mut tags = Vec::new();
        if !file.routes.is_empty() {
            tags.push("route".to_string());
        }
        if !file.api_endpoints.is_empty() || is_api_module(&file.normalized_path) {
            tags.push("api".to_string());
        }
        if !file.components.is_empty() {
            tags.push("component".to_string());
        }
        if file.extension == "sql" || is_data_file(&file.normalized_path) {
            tags.push("data".to_string());
        }
        if file.is_test {
            tags.push("test".to_string());
        }
        if file.is_generated {
            tags.push("generated".to_string());
        }
        if !file.env_vars.is_empty() {
            tags.push("env".to_string());
        }
        items.push(format!(
            "{{\"path\":\"{}\",\"lines\":{},\"tags\":{}}}",
            json_escape(&file.normalized_path),
            file.line_count,
            json_string_array(&tags)
        ));
    }
    format!("[{}]", items.join(", "))
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "agent-toolkit-intel-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn build_repo_intel_ignores_hidden_agent_tooling_dirs() {
        let root = temp_dir();
        fs::create_dir_all(root.join(".agents/intel")).unwrap();
        fs::create_dir_all(root.join(".codex")).unwrap();
        fs::write(root.join(".agents/intel/index.md"), "# Generated\n").unwrap();
        fs::write(root.join(".codex/config.toml"), "model = 'x'\n").unwrap();

        let intel = build_repo_intel(&root).unwrap();

        assert_eq!(intel.file_count, 0);
        assert!(intel.summary_markdown.contains("overview.md"));
        assert!(intel.summary_markdown.contains("graph.md"));
    }

    #[test]
    fn build_repo_intel_counts_framework_surface_and_hotspots() {
        let root = temp_dir();
        fs::create_dir_all(root.join("src/components")).unwrap();
        fs::create_dir_all(root.join("src/routes")).unwrap();
        fs::create_dir_all(root.join("supabase/migrations")).unwrap();
        fs::write(root.join("AGENTS.md"), "# Rules\n").unwrap();
        fs::write(
            root.join("package.json"),
            r#"{"scripts":{"check":"bun test"},"dependencies":{"react":"latest","@tanstack/react-start":"latest"}}"#,
        )
        .unwrap();
        fs::write(
            root.join("src/components/button.tsx"),
            "type ButtonProps = {\n\tlabel: string\n\tsize?: string\n}\nimport { cn } from '../cn'\nexport function Button(props: ButtonProps) {\n\treturn null\n}\n",
        )
        .unwrap();
        fs::write(
            root.join("src/routes/index.tsx"),
            "import { Button } from '../components/button'\nexport const Route = createFileRoute('/')({ component: Button })\n",
        )
        .unwrap();
        fs::write(
            root.join("supabase/migrations/001.sql"),
            "create table public.users (id uuid primary key);\n",
        )
        .unwrap();

        let intel = build_repo_intel(&root).unwrap();

        assert_eq!(intel.file_count, 5);
        assert!(intel.summary_markdown.contains("TanStack Start"));
        assert!(intel.summary_markdown.contains("Routes: 2"));
        assert!(intel.summary_markdown.contains("Components: 1"));
        assert!(intel.summary_markdown.contains("SQL objects: 1"));
    }

    #[test]
    fn write_repo_intel_creates_article_files() {
        let root = temp_dir();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/index.ts"), "export const value = 1\n").unwrap();

        write_repo_intel(&root).unwrap();

        for file in [
            "index.md",
            "overview.md",
            "tasks.md",
            "tooling.md",
            "routes.md",
            "api.md",
            "components.md",
            "data.md",
            "graph.md",
            "dependencies.md",
            "symbols.md",
            "files.md",
            "env.md",
            "testing.md",
            "summary.md",
            "repo.json",
        ] {
            assert!(root.join(".agents/intel").join(file).exists(), "{file}");
        }
    }

    #[test]
    fn extracts_package_object_keys() {
        let scripts = extract_object_keys(
            r#"{"scripts":{"check":"bun test","dev":"vite","nested":"echo { ok }"},"dependencies":{"ignored":"no"}}"#,
            "scripts",
        );

        assert_eq!(scripts, vec!["check", "dev", "nested"]);
    }

    #[test]
    fn package_name_from_import_handles_scopes() {
        assert_eq!(
            package_name_from_import("@tanstack/react-router/file-route"),
            "@tanstack/react-router"
        );
        assert_eq!(package_name_from_import("react/jsx-runtime"), "react");
    }

    #[test]
    fn filters_external_repo_intel_tooling_from_package_names() {
        let filtered = filter_external_intel_tooling(vec![
            format!("{}{}", "code", "sight"),
            "check".to_string(),
        ]);

        assert_eq!(filtered, vec!["check"]);
    }

    #[test]
    fn resolves_local_imports() {
        let files = vec![
            FileIntel {
                path: PathBuf::from("src/index.ts"),
                normalized_path: "src/index.ts".to_string(),
                extension: "ts".to_string(),
                line_count: 1,
                imports: vec!["./lib".to_string()],
                exports: Vec::new(),
                components: Vec::new(),
                routes: Vec::new(),
                api_endpoints: Vec::new(),
                sql_objects: Vec::new(),
                env_vars: Vec::new(),
                is_test: false,
                is_generated: false,
            },
            FileIntel {
                path: PathBuf::from("src/lib.ts"),
                normalized_path: "src/lib.ts".to_string(),
                extension: "ts".to_string(),
                line_count: 1,
                imports: Vec::new(),
                exports: Vec::new(),
                components: Vec::new(),
                routes: Vec::new(),
                api_endpoints: Vec::new(),
                sql_objects: Vec::new(),
                env_vars: Vec::new(),
                is_test: false,
                is_generated: false,
            },
        ];

        let edges = build_local_edges(&files);

        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].to, "src/lib.ts");
    }

    #[test]
    fn resolves_src_alias_imports() {
        let files = vec![
            FileIntel {
                path: PathBuf::from("src/index.ts"),
                normalized_path: "src/index.ts".to_string(),
                extension: "ts".to_string(),
                line_count: 1,
                imports: vec!["@/lib".to_string()],
                exports: Vec::new(),
                components: Vec::new(),
                routes: Vec::new(),
                api_endpoints: Vec::new(),
                sql_objects: Vec::new(),
                env_vars: Vec::new(),
                is_test: false,
                is_generated: false,
            },
            FileIntel {
                path: PathBuf::from("src/lib.ts"),
                normalized_path: "src/lib.ts".to_string(),
                extension: "ts".to_string(),
                line_count: 1,
                imports: Vec::new(),
                exports: Vec::new(),
                components: Vec::new(),
                routes: Vec::new(),
                api_endpoints: Vec::new(),
                sql_objects: Vec::new(),
                env_vars: Vec::new(),
                is_test: false,
                is_generated: false,
            },
        ];

        let edges = build_local_edges(&files);

        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].to, "src/lib.ts");
    }

    #[test]
    fn extracts_single_quoted_imports() {
        let imports = extract_imports("import { value } from './value'\n", "ts");

        assert_eq!(imports, vec!["./value"]);
    }

    #[test]
    fn component_names_exclude_all_caps_constants() {
        assert!(is_component_name("Button"));
        assert!(!is_component_name("ACTIVITY_LEVEL_ITEMS"));
    }
}
