/// UNO team member name for an agent id such as `core:security-reviewer` → `Epsilon`.
/// Unknown ids are returned without their plugin prefix.
pub fn agent_name(raw: &str) -> String {
    let id = raw.rsplit(':').next().unwrap_or(raw);
    let greek = match id {
        "intake-triager" => "Alpha",
        "codebase-explorer" => "Beta",
        "ui-designer" => "Gamma",
        "test-writer" => "Delta",
        "security-reviewer" => "Epsilon",
        "release-manager" => "Zeta",
        "incident-scribe" => "Eta",
        "doc-writer" => "Theta",
        "devops-engineer" => "Iota",
        "data-engineer" => "Kappa",
        "architect" => "Lambda",
        _ => return id.to_string(),
    };
    greek.to_string()
}
