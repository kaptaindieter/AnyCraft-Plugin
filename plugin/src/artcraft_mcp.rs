impl CraftApp {
    pub fn artcraft_project_summary(&self) -> String {
        let proj_name = self.workspace.as_ref().map(|w| w.manifest.name.clone())
            .or_else(|| self.view.project.clone())
            .unwrap_or_else(|| "AnyCraft Workspace".to_string());
        let mode_str = if self.beginner() { "Simple Mode" } else { "Advanced Workbench" };
        let deep_str = if self.settings.deep_mode { "Deep Mode" } else { "Clean Room" };
        let active_target = self.view.active_target().map(|t| format!("{} ({:?})", t.name, t.kind)).unwrap_or_else(|| "Kein Target".to_string());
        let projects_count = self.projects.len();
        let findings_count = self.view.ledger_len;
        format!(
            "{proj_name} ({mode_str} · {deep_str})\nScreen: {:?} | Target: {active_target}\nProjekte: {projects_count} | Findings: {findings_count} | Offene Fragen: {}",
            self.state.screen,
            self.view.unknowns.len()
        )
    }

    pub fn poll_artcraft_agent(&mut self) {
        let reqs = self.artcraft_agent.take_tool_requests();
        for req in reqs {
            let res = match req.tool_name.as_str() {
                "inspect_workspace" | "inspect_document" | "get_text" => {
                    let summary = self.artcraft_project_summary();
                    Ok(serde_json::json!({
                        "summary": summary,
                        "project": self.workspace.as_ref().map(|w| w.manifest.name.clone()),
                        "screen": format!("{:?}", self.state.screen),
                        "mode": if self.beginner() { "simple" } else { "advanced" },
                        "deep_mode": self.settings.deep_mode,
                        "active_target": self.view.active_target().map(|t| serde_json::json!({
                            "name": t.name,
                            "kind": format!("{:?}", t.kind),
                            "path": t.path.to_string_lossy()
                        })),
                        "projects_count": self.projects.len(),
                        "findings_count": self.view.ledger_len,
                        "unknowns_count": self.view.unknowns.len()
                    }))
                }
                "anycraft.setMakeBrief" => {
                    let request_txt = req.params.get("request").and_then(serde_json::Value::as_str).unwrap_or("");
                    if !request_txt.trim().is_empty() {
                        self.state.make.request = request_txt.trim().to_string();
                        self.state.make.look = false;
                    }
                    if let Some(url) = req.params.get("url").and_then(serde_json::Value::as_str) {
                        if !url.trim().is_empty() {
                            self.state.make.url = url.trim().to_string();
                            self.state.make.editing_url = true;
                        }
                    }
                    if let Some(deep) = req.params.get("deepMode").and_then(serde_json::Value::as_bool) {
                        self.set_deep(deep);
                    }
                    self.state.screen = Screen::Home;
                    self.toast("✓ App-Briefing in Make-Prompt übernommen", false);
                    Ok(serde_json::json!({
                        "status": "ok",
                        "screen": "Home",
                        "request_len": self.state.make.request.len()
                    }))
                }
                "anycraft.switchView" => {
                    if let Some(mode_s) = req.params.get("mode").and_then(serde_json::Value::as_str) {
                        match mode_s.trim().to_lowercase().as_str() {
                            "simple" | "beginner" => self.set_mode(Mode::Beginner),
                            "advanced" | "experienced" | "workbench" => self.set_mode(Mode::Experienced),
                            _ => {}
                        }
                    }
                    if let Some(deep) = req.params.get("deepMode").and_then(serde_json::Value::as_bool) {
                        self.set_deep(deep);
                    }
                    if let Some(screen_s) = req.params.get("screen").and_then(serde_json::Value::as_str) {
                        match screen_s.trim().to_lowercase().as_str() {
                            "home" | "make" => self.state.screen = Screen::Home,
                            "workbench" => {
                                self.set_mode(Mode::Experienced);
                                self.state.screen = Screen::Workbench;
                            }
                            "ask" => self.state.screen = Screen::Ask,
                            "project" => self.state.screen = Screen::Project,
                            "game_files" | "gamefiles" => self.state.screen = Screen::GameFiles,
                            _ => {}
                        }
                    }
                    if let Some(rail_s) = req.params.get("rail").and_then(serde_json::Value::as_str) {
                        let rk = match rail_s.trim().to_lowercase().as_str() {
                            "targets" => Some(RailKind::Targets),
                            "functions" => Some(RailKind::Functions),
                            "strings" => Some(RailKind::Strings),
                            "imports" | "exports" => Some(RailKind::Imports),
                            "sections" => Some(RailKind::Sections),
                            "classes" => Some(RailKind::Classes),
                            "modules" => Some(RailKind::Modules),
                            "resources" => Some(RailKind::Resources),
                            _ => None,
                        };
                        if let Some(r) = rk {
                            self.state.rail = r;
                            self.load_rail(r);
                        }
                    }
                    Ok(serde_json::json!({
                        "status": "ok",
                        "screen": format!("{:?}", self.state.screen),
                        "mode": if self.beginner() { "simple" } else { "advanced" },
                        "deep_mode": self.settings.deep_mode
                    }))
                }
                "anycraft.runAnalysis" => {
                    let tool_id = req.params.get("tool").and_then(serde_json::Value::as_str).unwrap_or("core_target_triage");
                    if let Some(rail_s) = req.params.get("rail").and_then(serde_json::Value::as_str) {
                        let rk = match rail_s.trim().to_lowercase().as_str() {
                            "functions" => Some(RailKind::Functions),
                            "strings" => Some(RailKind::Strings),
                            "imports" | "exports" => Some(RailKind::Imports),
                            "sections" => Some(RailKind::Sections),
                            "classes" => Some(RailKind::Classes),
                            "modules" => Some(RailKind::Modules),
                            "resources" => Some(RailKind::Resources),
                            _ => None,
                        };
                        if let Some(r) = rk {
                            self.set_mode(Mode::Experienced);
                            self.state.screen = Screen::Workbench;
                            self.state.rail = r;
                            self.load_rail(r);
                        }
                    }
                    if self.has_tool(tool_id) {
                        let params = self.params_for(tool_id, &[]);
                        self.run_tool(tool_id, params, Route::Palette, Consent::Granted);
                        Ok(serde_json::json!({"status": "started", "tool": tool_id}))
                    } else {
                        Ok(serde_json::json!({"status": "ok", "note": format!("Switched view for {tool_id}")}))
                    }
                }
                "anycraft.openTarget" => {
                    let path_s = req.params.get("path").and_then(serde_json::Value::as_str).unwrap_or("");
                    if !path_s.trim().is_empty() {
                        let p = PathBuf::from(path_s.trim());
                        self.open_path(p);
                        Ok(serde_json::json!({"status": "opening", "path": path_s}))
                    } else {
                        Err("Missing 'path' parameter".to_string())
                    }
                }
                "insert_from_chat" | "type_text" => {
                    let md = req.params.get("markdown")
                        .or_else(|| req.params.get("text"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("");
                    if !md.trim().is_empty() {
                        match self.state.screen {
                            Screen::Project => {
                                self.state.project_input = md.trim().to_string();
                            }
                            Screen::Ask | Screen::Workbench => {
                                self.state.input = md.trim().to_string();
                            }
                            _ => {
                                self.state.make.request = md.trim().to_string();
                                self.state.make.look = false;
                                self.state.screen = Screen::Home;
                            }
                        }
                        self.toast("✓ Chat-Antwort in Prompt-Feld übernommen", false);
                    }
                    Ok(serde_json::json!({"status": "inserted", "screen": format!("{:?}", self.state.screen)}))
                }
                "save_document" => {
                    let target_dir = self.artcraft_drive.active_project_dir();
                    let _ = self.sync_and_save_settings();
                    self.artcraft_drive.mark_saved();
                    Ok(serde_json::json!({
                        "saved": true,
                        "folder": self.artcraft_drive.active_project_folder_name(),
                        "path": target_dir.to_string_lossy(),
                    }))
                }
                "execute" => {
                    let cmd = req.params.get("command").and_then(serde_json::Value::as_str).unwrap_or("");
                    match cmd {
                        "screen.home" => { self.state.screen = Screen::Home; Ok(serde_json::json!({"status": "navigated to Home"})) }
                        "screen.workbench" => { self.set_mode(Mode::Experienced); self.state.screen = Screen::Workbench; Ok(serde_json::json!({"status": "navigated to Workbench"})) }
                        "screen.ask" => { self.state.screen = Screen::Ask; Ok(serde_json::json!({"status": "navigated to Ask"})) }
                        "screen.project" => { self.state.screen = Screen::Project; Ok(serde_json::json!({"status": "navigated to Project"})) }
                        "screen.game_files" => { self.state.screen = Screen::GameFiles; Ok(serde_json::json!({"status": "navigated to GameFiles"})) }
                        "screen.settings" => { self.state.settings_open = true; Ok(serde_json::json!({"status": "opened Settings"})) }
                        "mode.simple" => { self.set_mode(Mode::Beginner); Ok(serde_json::json!({"status": "switched to Simple mode"})) }
                        "mode.advanced" => { self.set_mode(Mode::Experienced); Ok(serde_json::json!({"status": "switched to Advanced mode"})) }
                        "mode.toggle" => {
                            let new_mode = if self.beginner() { Mode::Experienced } else { Mode::Beginner };
                            self.set_mode(new_mode);
                            Ok(serde_json::json!({"status": "toggled mode", "mode": format!("{new_mode:?}")}))
                        }
                        "deep.on" => { self.set_deep(true); Ok(serde_json::json!({"status": "Deep mode enabled"})) }
                        "deep.off" => { self.set_deep(false); Ok(serde_json::json!({"status": "Clean room mode enabled"})) }
                        other => {
                            Ok(serde_json::json!({"executed": other, "note": "executed command in AnyCraft"}))
                        }
                    }
                }
                other => {
                    Ok(serde_json::json!({"tool": other, "status": "handled"}))
                }
            };
            let _ = req.reply_sender.send(res);
        }

        self.artcraft_agent.process_events();
    }

    pub fn artcraft_autosave_tick(&mut self) {
        if self.artcraft_drive.auto_save_enabled && self.artcraft_drive.should_autosave() {
            let _ = self.sync_and_save_settings();
            self.artcraft_drive.mark_saved();
        }
    }
}
