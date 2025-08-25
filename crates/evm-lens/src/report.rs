use std::fs::File;
use std::io::Write;
use std::path::Path;

use color_eyre::Result;
use evm_lens_core::storage::{DiffEntry, SeverityGrade, Summary, Provenance, StorageType};
use serde::Serialize;

#[derive(Serialize)]
struct JsonReport<'a> {
    diffs: &'a [DiffEntry],
    summary: &'a Summary,
}

pub fn write_json_report(path: &Path, diffs: &[DiffEntry], summary: &Summary) -> Result<()> {
    let report = JsonReport { diffs, summary };
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn write_html_report(path: &Path, diffs: &[DiffEntry], summary: &Summary) -> Result<()> {
    let mut f = File::create(path)?;
    writeln!(
        f,
        "<html><head><meta charset=\"utf-8\"><title>evm-lens storage diff</title>"
    )?;
    writeln!(
        f,
        "<style>body{{font-family:system-ui,Arial,sans-serif}} table{{border-collapse:collapse;width:100%}} th,td{{border:1px solid #ddd;padding:6px}} th{{background:#f5f5f5;text-align:left;position:sticky;top:0;z-index:1}} tr:nth-child(even){{background:#fafafa}} .mono{{font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,\\\"Liberation Mono\\\",monospace}} .ok{{color:#2e7d32}} .risk{{color:#ff8f00}} .break{{color:#c62828}} .chip{{display:inline-block;padding:2px 8px;border-radius:12px;font-size:12px}} .chip.ok{{background:#e8f5e9;color:#2e7d32}} .chip.risk{{background:#fff8e1;color:#ff8f00}} .chip.break{{background:#ffebee;color:#c62828}} .badge{{display:inline-block;padding:2px 6px;border-radius:6px;font-size:12px;margin-right:4px}} .badge.meta{{background:#e3f2fd;color:#1565c0}} .badge.heur{{background:#fff3e0;color:#ef6c00}}</style>"
    )?;
    writeln!(f, "</head><body>")?;
    writeln!(f, "<h2>Storage Diff</h2>")?;
    let max_grade_class = match summary.max_grade { SeverityGrade::Ok => "ok", SeverityGrade::Risk => "risk", SeverityGrade::Break => "break" };
    writeln!(f, "<p>Added: {} &nbsp; Removed: {} &nbsp; TypeChanged: {} &nbsp; PackingChanged: {} &nbsp; Same: {} &nbsp; Max grade: <span class=\"chip {}\">{:?}</span></p>", summary.added, summary.removed, summary.type_changed, summary.packing_changed, summary.same, max_grade_class, summary.max_grade)?;

    writeln!(f, "<table>")?;
    writeln!(
        f,
        "<tr><th>Slot</th><th>Old</th><th>New</th><th>Status</th><th>Grade</th><th>Provenance</th></tr>"
    )?;
    for d in diffs {
        let grade_class = match d.grade {
            SeverityGrade::Ok => "ok",
            SeverityGrade::Risk => "risk",
            SeverityGrade::Break => "break",
        };
        let (old_desc, old_unknown) = match d.old.as_ref() {
            Some(e) => (format!("{:?}", e.r#type), matches!(e.r#type, StorageType::Unknown)),
            None => ("—".to_string(), false),
        };
        let (new_desc, new_unknown) = match d.new.as_ref() {
            Some(e) => (format!("{:?}", e.r#type), matches!(e.r#type, StorageType::Unknown)),
            None => ("—".to_string(), false),
        };
        let tip = "No compiler metadata available; conservative heuristic left type unknown.";
        let old_html = if old_unknown { format!("<span title=\"{}\">{}</span>", tip, old_desc) } else { old_desc };
        let new_html = if new_unknown { format!("<span title=\"{}\">{}</span>", tip, new_desc) } else { new_desc };

        let badge = |p: Option<Provenance>| -> String {
            match p {
                Some(Provenance::CompilerMetadata) => "<span class=\"badge meta\">Metadata</span>".to_string(),
                Some(Provenance::HeuristicTrace) => "<span class=\"badge heur\">Heuristic</span>".to_string(),
                None => "—".to_string(),
            }
        };
        let prov = format!("{} / {}", badge(d.provenance_old), badge(d.provenance_new));
        let slot_cell = format!("{} (0x{:x})", d.slot, d.slot);
        writeln!(
            f,
            "<tr><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td>{:?}</td><td class=\"{}\">{:?}</td><td>{}</td></tr>",
            slot_cell, old_html, new_html, d.status, grade_class, d.grade, prov
        )?;
    }
    writeln!(f, "</table>")?;
    writeln!(f, "</body></html>")?;
    Ok(())
}
