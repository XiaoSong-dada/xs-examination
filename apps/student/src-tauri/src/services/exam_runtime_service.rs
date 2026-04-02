use anyhow::{Context, Result};
use calamine::{open_workbook_auto, Reader};
use crate::network::protocol::AnswerItem;
use serde::Deserialize;
use sha2::Digest;
use std::collections::HashMap;
use std::path::Path;
use tauri::Manager;

<<<<<<< fix-规范student代码分层
=======
use crate::db::entities::{exam_question_assets, exam_sessions, exam_snapshots, local_answers, sync_outbox};
use crate::network::protocol::PaperPackageManifestPayload;
>>>>>>> v0.1.1-dev
use crate::schemas::control_protocol::{ApplyTeacherEndpointsPayload, DistributeExamPaperPayload};
use crate::schemas::exam_runtime_schema::{CurrentExamBundleDto, LocalAnswerDto};
use crate::utils::datetime::now_ms;
use crate::repos::{exam_session_repo, exam_snapshot_repo, local_answer_repo, sync_outbox_repo};


pub struct ExamRuntimeService;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PendingAnswerSyncPayload {
    exam_id: String,
    student_id: String,
    question_id: String,
    answer: String,
    revision: Option<i64>,
    timestamp: Option<i64>,
}

impl ExamRuntimeService {
    pub async fn prepare_exam_package_receive(
        app_handle: &tauri::AppHandle,
        manifest: &PaperPackageManifestPayload,
        package_path: &str,
    ) -> Result<()> {
        let state = app_handle.state::<crate::state::AppState>();
        let ts = now_ms();

        let existing_session = exam_sessions::Entity::find_by_id(manifest.session_id.clone())
            .one(&state.db)
            .await?;
        match existing_session {
            Some(row) => {
                let mut model: exam_sessions::ActiveModel = row.into();
                model.exam_id = Set(manifest.exam_id.clone());
                model.student_id = Set(manifest.student_id.clone());
                model.exam_title = Set(manifest.exam_title.clone());
                model.assignment_status = Set(manifest.assignment_status.clone());
                model.ends_at = Set(manifest.end_time);
                model.paper_version = Set(manifest.paper_version.clone());
                model.updated_at = Set(ts);
                model.update(&state.db).await?;
            }
            None => {
                let model = exam_sessions::ActiveModel {
                    id: Set(manifest.session_id.clone()),
                    exam_id: Set(manifest.exam_id.clone()),
                    student_id: Set(manifest.student_id.clone()),
                    student_no: Set("".to_string()),
                    student_name: Set("".to_string()),
                    assigned_ip_addr: Set("".to_string()),
                    assigned_device_name: Set(None),
                    exam_title: Set(manifest.exam_title.clone()),
                    status: Set("waiting".to_string()),
                    assignment_status: Set(manifest.assignment_status.clone()),
                    started_at: Set(None),
                    ends_at: Set(manifest.end_time),
                    paper_version: Set(manifest.paper_version.clone()),
                    encryption_nonce: Set(None),
                    last_synced_at: Set(None),
                    created_at: Set(ts),
                    updated_at: Set(ts),
                };
                model.insert(&state.db).await?;
            }
        }

        let existing_snapshot = exam_snapshots::Entity::find_by_id(manifest.session_id.clone())
            .one(&state.db)
            .await?;
        match existing_snapshot {
            Some(row) => {
                let mut model: exam_snapshots::ActiveModel = row.into();
                model.package_path = Set(Some(package_path.to_string()));
                model.package_status = Set(Some("receiving".to_string()));
                model.package_batch_id = Set(Some(manifest.batch_id.clone()));
                model.package_sha256 = Set(Some(manifest.sha256.clone()));
                model.package_received_at = Set(None);
                model.assets_sync_status = Set(Some("pending".to_string()));
                model.assets_synced_at = Set(None);
                model.updated_at = Set(ts);
                model.update(&state.db).await?;
            }
            None => {
                let model = exam_snapshots::ActiveModel {
                    session_id: Set(manifest.session_id.clone()),
                    exam_meta: Set(Vec::new()),
                    questions_payload: Set(Vec::new()),
                    downloaded_at: Set(ts),
                    expires_at: Set(manifest.end_time),
                    package_path: Set(Some(package_path.to_string())),
                    package_status: Set(Some("receiving".to_string())),
                    package_batch_id: Set(Some(manifest.batch_id.clone())),
                    package_sha256: Set(Some(manifest.sha256.clone())),
                    package_received_at: Set(None),
                    assets_sync_status: Set(Some("pending".to_string())),
                    assets_synced_at: Set(None),
                    updated_at: Set(ts),
                };
                model.insert(&state.db).await?;
            }
        }

        Ok(())
    }

    pub async fn mark_exam_package_received(
        app_handle: &tauri::AppHandle,
        session_id: &str,
        batch_id: &str,
        package_path: &str,
        sha256: &str,
    ) -> Result<()> {
        let state = app_handle.state::<crate::state::AppState>();
        let ts = now_ms();
        let row = exam_snapshots::Entity::find_by_id(session_id.to_string())
            .one(&state.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("考试快照不存在，无法标记试卷包接收"))?;

        let mut model: exam_snapshots::ActiveModel = row.into();
        model.package_path = Set(Some(package_path.to_string()));
        model.package_status = Set(Some("received".to_string()));
        model.package_batch_id = Set(Some(batch_id.to_string()));
        model.package_sha256 = Set(Some(sha256.to_string()));
        model.package_received_at = Set(Some(ts));
        model.updated_at = Set(ts);
        model.update(&state.db).await?;

        Ok(())
    }

    async fn materialize_exam_package_if_needed(
        app_handle: &tauri::AppHandle,
        snapshot: &exam_snapshots::Model,
        session: &exam_sessions::Model,
    ) -> Result<()> {
        let package_status = snapshot.package_status.clone().unwrap_or_default();
        if package_status == "receiving" {
            return Err(anyhow::anyhow!("试卷包尚未接收完成，暂不可开始考试"));
        }
        if package_status != "received" {
            return Ok(());
        }

        let package_path = snapshot
            .package_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("试卷包路径缺失"))?;
        let package_sha256 = snapshot
            .package_sha256
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("试卷包校验值缺失"))?;

        let bytes = std::fs::read(package_path)
            .with_context(|| format!("读取试卷包失败: {}", package_path))?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(&bytes);
        let actual_sha = format!("{:x}", hasher.finalize());
        if actual_sha != *package_sha256 {
            return Err(anyhow::anyhow!("试卷包校验失败"));
        }

        let app_data_dir = app_handle.path().app_data_dir()?;
        let expand_dir = app_data_dir
            .join("paper_packages")
            .join(&session.id)
            .join("expanded");
        if expand_dir.exists() {
            let _ = std::fs::remove_dir_all(&expand_dir);
        }
        std::fs::create_dir_all(&expand_dir)?;

        let zip_file = std::fs::File::open(package_path)?;
        let mut archive = zip::ZipArchive::new(zip_file)?;
        let mut extracted_assets: Vec<(String, std::path::PathBuf)> = Vec::new();
        let mut xlsx_path = None;

        for idx in 0..archive.len() {
            let mut entry = archive.by_index(idx)?;
            let archive_name = entry.name().replace('\\', "/");
            let out_path = expand_dir.join(
                entry
                    .enclosed_name()
                    .ok_or_else(|| anyhow::anyhow!("试卷包路径非法"))?,
            );

            if entry.is_dir() {
                std::fs::create_dir_all(&out_path)?;
                continue;
            }

            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut out_file = std::fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out_file)?;

            let lower = archive_name.to_ascii_lowercase();
            if lower == "question_bank.xlsx" || lower.ends_with(".xlsx") {
                xlsx_path = Some(out_path.clone());
            }
            if lower.starts_with("assets/content/") || lower.starts_with("assets/options/") {
                extracted_assets.push((archive_name, out_path.clone()));
            }
        }

        let xlsx = xlsx_path.ok_or_else(|| anyhow::anyhow!("试卷包缺少 question_bank.xlsx"))?;
        let mut mapping: HashMap<String, String> = HashMap::new();
        let mut asset_rows: Vec<exam_question_assets::ActiveModel> = Vec::new();
        let ts = now_ms();

        for (archive_name, src_path) in extracted_assets {
            let lower = archive_name.to_ascii_lowercase();
            let scope = if lower.starts_with("assets/content/") {
                "content"
            } else {
                "options"
            };

            let target_dir = app_data_dir
                .join("uploads")
                .join("images")
                .join("exam")
                .join(&session.id)
                .join(scope);
            std::fs::create_dir_all(&target_dir)?;

            let file_name = format!(
                "{}_{}",
                uuid::Uuid::new_v4().simple(),
                src_path.file_name().and_then(|v| v.to_str()).unwrap_or("asset.bin")
            );
            let target_path = target_dir.join(&file_name);
            std::fs::copy(&src_path, &target_path)?;

            let relative_path = format!("uploads/images/exam/{}/{}/{}", session.id, scope, file_name);
            mapping.insert(archive_name, relative_path.clone());
            asset_rows.push(exam_question_assets::ActiveModel {
                id: Set(uuid::Uuid::new_v4().to_string()),
                session_id: Set(session.id.clone()),
                exam_id: Set(session.exam_id.clone()),
                question_id: Set(String::new()),
                scope: Set(scope.to_string()),
                asset_local_path: Set(relative_path),
                source_archive_path: Set(Some(src_path.to_string_lossy().to_string())),
                checksum: Set(None),
                created_at: Set(ts),
                updated_at: Set(ts),
            });
        }

        let payload = parse_questions_payload_from_xlsx(&xlsx, &mapping)?;
        let payload_text = serde_json::to_string(&payload)?;

        let state = app_handle.state::<crate::state::AppState>();
        let row = exam_snapshots::Entity::find_by_id(session.id.clone())
            .one(&state.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("考试快照不存在"))?;
        let mut model: exam_snapshots::ActiveModel = row.into();
        model.questions_payload = Set(payload_text.into_bytes());
        model.package_status = Set(Some("extracted".to_string()));
        model.assets_sync_status = Set(Some("synced".to_string()));
        model.assets_synced_at = Set(Some(ts));
        model.updated_at = Set(ts);
        model.update(&state.db).await?;

        if !asset_rows.is_empty() {
            exam_question_assets::Entity::insert_many(asset_rows)
                .exec(&state.db)
                .await?;
        }

        Ok(())
    }

    pub async fn flush_pending_answer_sync(
        app_handle: &tauri::AppHandle,
        max_count: usize,
    ) -> Result<usize> {
        let state = app_handle.state::<crate::state::AppState>();
        let Some(sender) = state.ws_sender() else {
            return Ok(0);
        };

        let rows = sync_outbox_repo::get_pending_answer_syncs(&state.db, max_count).await?;

        let mut flushed = 0usize;
        for row in rows {
            let session_status = exam_session_repo::get_session_by_id(&state.db, &row.session_id)
                .await?
                .map(|session| session.status)
                .unwrap_or_else(|| "waiting".to_string());

            if session_status == "ended" {
                sync_outbox_repo::mark_outbox_failed(&state.db, row, "考试已结束，停止继续同步答案", now_ms()).await?;
                continue;
            }

            let payload: PendingAnswerSyncPayload = match serde_json::from_slice(&row.payload) {
                Ok(v) => v,
                Err(err) => {
                    sync_outbox_repo::mark_outbox_failed(&state.db, row, &format!("payload parse failed: {}", err), now_ms()).await?;
                    continue;
                }
            };

            let answer_item = AnswerItem {
                question_id: payload.question_id.clone(),
                answer: payload.answer.clone(),
                revision: payload.revision,
                answer_updated_at: payload.timestamp,
            };

            let message = crate::network::ws_client::build_answer_sync_message(
                &payload.exam_id,
                &payload.student_id,
                Some(&row.session_id),
                vec![answer_item],
                "incremental",
                Some(&format!("outbox-{}", row.id)),
            )?;

            if sender.send(message).is_err() {
                break;
            }

            sync_outbox_repo::mark_outbox_sent(&state.db, row, now_ms()).await?;
            flushed += 1;
        }

        Ok(flushed)
    }

    /// 发送当前会话答案同步消息，支持 full/incremental/final 三种同步模式。
    ///
    /// # 参数
    /// * `app_handle` - Tauri 应用句柄。
    /// * `sync_mode` - 同步模式，例如 `full`、`incremental`、`final`。
    /// * `batch_id` - 同步批次标识；用于教师端聚合 ACK。
    /// * `allow_empty` - 是否允许在无答案时仍发送空载荷同步消息。
    ///
    /// # 返回值
    /// 返回 `Ok(true)` 表示消息已成功入发送通道；`Ok(false)` 表示当前无可发送会话或未连接。
    pub async fn send_current_session_answer_sync(
        app_handle: &tauri::AppHandle,
        sync_mode: &str,
        batch_id: Option<&str>,
        allow_empty: bool,
    ) -> Result<bool> {
        let state = app_handle.state::<crate::state::AppState>();
        let Some(sender) = state.ws_sender() else {
            return Ok(false);
        };

        let Some((session_id, exam_id, student_id, answers)) =
            Self::get_current_session_answer_sync_bundle(app_handle).await?
        else {
            return Ok(false);
        };

        if !allow_empty && answers.is_empty() {
            return Ok(false);
        }

        if sync_mode == "full" && !state.should_send_full_sync(&session_id, now_ms(), 5_000) {
            println!(
                "[ws-client] skip duplicated full answer sync session_id={} within cooldown",
                session_id
            );
            return Ok(false);
        }

        let payload_answers: Vec<AnswerItem> = answers
            .into_iter()
            .map(|item| AnswerItem {
                question_id: item.question_id,
                answer: item.answer,
                revision: Some(item.revision),
                answer_updated_at: Some(item.updated_at),
            })
            .collect();

        let message = crate::network::ws_client::build_answer_sync_message(
            &exam_id,
            &student_id,
            Some(&session_id),
            payload_answers,
            sync_mode,
            batch_id,
        )?;

        if sender.send(message).is_err() {
            return Err(anyhow::anyhow!("答案同步发送失败：发送通道不可用"));
        }

        Ok(true)
    }

    pub async fn get_current_session_answer_sync_bundle(
        app_handle: &tauri::AppHandle,
    ) -> Result<Option<(String, String, String, Vec<LocalAnswerDto>)>> {
        let state = app_handle.state::<crate::state::AppState>();
        let sessions = exam_session_repo::get_all_sessions(&state.db).await?;

        if sessions.is_empty() {
            return Ok(None);
        }

        let target_student_id = state.reconnect_target().map(|(_, sid)| sid);
        let session = if let Some(target_sid) = target_student_id {
            sessions
                .iter()
                .find(|item| item.student_id == target_sid)
                .cloned()
                .unwrap_or_else(|| sessions[0].clone())
        } else {
            sessions[0].clone()
        };

        let answers = local_answer_repo::get_answers_by_session_id(&state.db, &session.id).await?;
        let items = local_answer_repo::answers_to_dtos(answers);

        Ok(Some((session.id, session.exam_id, session.student_id, items)))
    }

    pub async fn mark_answers_synced(
        app_handle: &tauri::AppHandle,
        exam_id: &str,
        student_id: &str,
        session_id: Option<&str>,
        question_ids: &[String],
        acked_at: i64,
    ) -> Result<usize> {
        let state = app_handle.state::<crate::state::AppState>();

        let resolved_session_id = if let Some(value) = session_id {
            value.to_string()
        } else {
            let row = exam_session_repo::get_session_by_exam_and_student(&state.db, exam_id, student_id)
                .await?;

            let Some(session) = row else {
                return Ok(0);
            };
            session.id
        };

        let synced_count = local_answer_repo::mark_answers_synced(&state.db, &resolved_session_id, question_ids, acked_at).await?;
        sync_outbox_repo::mark_outbox_synced(&state.db, &resolved_session_id, question_ids, acked_at).await?;

        Ok(synced_count)
    }

    pub async fn mark_answers_failed(
        app_handle: &tauri::AppHandle,
        exam_id: &str,
        student_id: &str,
        session_id: Option<&str>,
        question_ids: &[String],
        failed_at: i64,
        error_message: &str,
    ) -> Result<usize> {
        let state = app_handle.state::<crate::state::AppState>();

        let resolved_session_id = if let Some(value) = session_id {
            value.to_string()
        } else {
            let row = exam_session_repo::get_session_by_exam_and_student(&state.db, exam_id, student_id)
                .await?;

            let Some(session) = row else {
                return Ok(0);
            };
            session.id
        };

        let failed_count = local_answer_repo::mark_answers_failed(&state.db, &resolved_session_id, question_ids, failed_at).await?;
        sync_outbox_repo::mark_outbox_failed_batch(&state.db, &resolved_session_id, question_ids, error_message, failed_at).await?;

        Ok(failed_count)
    }

    pub async fn get_current_session_answers(
        app_handle: &tauri::AppHandle,
    ) -> Result<Vec<LocalAnswerDto>> {
        let state = app_handle.state::<crate::state::AppState>();
        let sessions = exam_session_repo::get_all_sessions(&state.db).await?;

        let Some(session) = sessions.first() else {
            return Ok(Vec::new());
        };

        let answers = local_answer_repo::get_answers_by_session_id(&state.db, &session.id).await?;
        let items = local_answer_repo::answers_to_dtos(answers);

        Ok(items)
    }

    pub async fn upsert_connected_session(
        app_handle: &tauri::AppHandle,
        payload: &ApplyTeacherEndpointsPayload,
    ) -> Result<bool> {
        let (
            Some(session_id),
            Some(exam_id),
            Some(exam_title),
            Some(student_no),
            Some(student_name),
            Some(assigned_ip_addr),
        ) = (
            payload.session_id.clone(),
            payload.exam_id.clone(),
            payload.exam_title.clone(),
            payload.student_no.clone(),
            payload.student_name.clone(),
            payload.assigned_ip_addr.clone(),
        ) else {
            return Ok(false);
        };

        if session_id.trim().is_empty()
            || exam_id.trim().is_empty()
            || exam_title.trim().is_empty()
            || student_no.trim().is_empty()
            || student_name.trim().is_empty()
            || assigned_ip_addr.trim().is_empty()
        {
            return Ok(false);
        }

        let state = app_handle.state::<crate::state::AppState>();
        let ts = now_ms();

        exam_session_repo::upsert_connected_session(&state.db, payload, ts).await?;

        Ok(true)
    }

    pub async fn upsert_distribution(
        app_handle: &tauri::AppHandle,
        payload: &DistributeExamPaperPayload,
    ) -> Result<()> {
        let state = app_handle.state::<crate::state::AppState>();
        let ts = now_ms();

<<<<<<< fix-规范student代码分层
        let target_session_id = exam_session_repo::upsert_distribution(&state.db, payload, ts).await?;
        exam_snapshot_repo::upsert_snapshot(&state.db, &target_session_id, payload, ts).await?;
=======
        let existing_same_exam = exam_sessions::Entity::find()
            .filter(exam_sessions::Column::ExamId.eq(payload.exam_id.clone()))
            .order_by_desc(exam_sessions::Column::UpdatedAt)
            .one(&state.db)
            .await?;

        let target_session_id = existing_same_exam
            .as_ref()
            .map(|item| item.id.clone())
            .unwrap_or_else(|| payload.session_id.clone());

        // 第二阶段策略：命中同 exam_id 时，保留本地 exam_sessions 基础信息，不做覆盖更新。
        // 但仍需刷新状态与更新时间，确保前端读取到的最新会话可关联快照。
        if let Some(row) = existing_same_exam {
            let mut model: exam_sessions::ActiveModel = row.into();
            model.status = Set("waiting".to_string());
            model.ends_at = Set(payload.end_time);
            model.paper_version = Set(payload.paper_version.clone());
            model.updated_at = Set(ts);
            model.update(&state.db).await?;
        } else {
            let existing_session = exam_sessions::Entity::find_by_id(payload.session_id.clone())
                .one(&state.db)
                .await?;

            match existing_session {
                Some(row) => {
                    let mut model: exam_sessions::ActiveModel = row.into();
                    model.exam_id = Set(payload.exam_id.clone());
                    model.student_id = Set(payload.student_id.clone());
                    model.student_no = Set(payload.student_no.clone());
                    model.student_name = Set(payload.student_name.clone());
                    model.assigned_ip_addr = Set(payload.assigned_ip_addr.clone());
                    model.exam_title = Set(payload.exam_title.clone());
                    model.status = Set("waiting".to_string());
                    model.assignment_status = Set(payload.assignment_status.clone());
                    model.started_at = Set(None);
                    model.ends_at = Set(payload.end_time);
                    model.paper_version = Set(payload.paper_version.clone());
                    model.encryption_nonce = Set(None);
                    model.updated_at = Set(ts);
                    model.update(&state.db).await?;
                }
                None => {
                    let model = exam_sessions::ActiveModel {
                        id: Set(payload.session_id.clone()),
                        exam_id: Set(payload.exam_id.clone()),
                        student_id: Set(payload.student_id.clone()),
                        student_no: Set(payload.student_no.clone()),
                        student_name: Set(payload.student_name.clone()),
                        assigned_ip_addr: Set(payload.assigned_ip_addr.clone()),
                        assigned_device_name: Set(None),
                        exam_title: Set(payload.exam_title.clone()),
                        status: Set("waiting".to_string()),
                        assignment_status: Set(payload.assignment_status.clone()),
                        started_at: Set(None),
                        ends_at: Set(payload.end_time),
                        paper_version: Set(payload.paper_version.clone()),
                        encryption_nonce: Set(None),
                        last_synced_at: Set(None),
                        created_at: Set(ts),
                        updated_at: Set(ts),
                    };
                    model.insert(&state.db).await?;
                }
            }
        }

        let existing_snapshot = exam_snapshots::Entity::find_by_id(target_session_id.clone())
            .one(&state.db)
            .await?;
        match existing_snapshot {
            Some(row) => {
                let mut model: exam_snapshots::ActiveModel = row.into();
                model.exam_meta = Set(payload.exam_meta.clone().into_bytes());
                model.questions_payload = Set(payload.questions_payload.clone().into_bytes());
                model.downloaded_at = Set(payload.downloaded_at);
                model.expires_at = Set(payload.expires_at);
                model.package_status = Set(Some("legacy_ready".to_string()));
                model.package_path = Set(None);
                model.package_batch_id = Set(None);
                model.package_sha256 = Set(None);
                model.package_received_at = Set(None);
                model.assets_sync_status = Set(Some("pending".to_string()));
                model.assets_synced_at = Set(None);
                model.updated_at = Set(ts);
                model.update(&state.db).await?;
            }
            None => {
                let model = exam_snapshots::ActiveModel {
                    session_id: Set(target_session_id),
                    exam_meta: Set(payload.exam_meta.clone().into_bytes()),
                    questions_payload: Set(payload.questions_payload.clone().into_bytes()),
                    downloaded_at: Set(payload.downloaded_at),
                    expires_at: Set(payload.expires_at),
                    package_path: Set(None),
                    package_status: Set(Some("legacy_ready".to_string())),
                    package_batch_id: Set(None),
                    package_sha256: Set(None),
                    package_received_at: Set(None),
                    assets_sync_status: Set(Some("pending".to_string())),
                    assets_synced_at: Set(None),
                    updated_at: Set(ts),
                };
                model.insert(&state.db).await?;
            }
        }
>>>>>>> v0.1.1-dev

        Ok(())
    }

    pub async fn mark_exam_started(
        app_handle: &tauri::AppHandle,
        exam_id: &str,
        student_id: &str,
        start_time: i64,
        end_time: Option<i64>,
    ) -> Result<bool> {
        let state = app_handle.state::<crate::state::AppState>();
        let rows = exam_session_repo::get_all_sessions(&state.db).await?;

        let filtered_rows: Vec<_> = rows
            .into_iter()
            .filter(|row| row.exam_id == exam_id && row.student_id == student_id)
            .collect();

        if filtered_rows.is_empty() {
            return Ok(false);
        }

        // Prefer a session that already has a snapshot to ensure frontend can enter exam view.
        let mut selected = filtered_rows[0].clone();
        for row in filtered_rows {
            let snapshot = exam_snapshot_repo::get_snapshot_by_session_id(&state.db, &row.id)
                .await?;
            if snapshot.is_some() {
                selected = row;
                break;
            }
        }

<<<<<<< fix-规范student代码分层
        exam_session_repo::mark_session_started(&state.db, selected, start_time, end_time, now_ms()).await?;
=======
        if let Some(snapshot) = exam_snapshots::Entity::find_by_id(selected.id.clone())
            .one(&state.db)
            .await?
        {
            Self::materialize_exam_package_if_needed(app_handle, &snapshot, &selected).await?;

            let refreshed_snapshot = exam_snapshots::Entity::find_by_id(selected.id.clone())
                .one(&state.db)
                .await?;
            if let Some(refreshed) = refreshed_snapshot {
                if refreshed.questions_payload.is_empty() {
                    return Err(anyhow::anyhow!("试卷数据尚未准备完成，拒绝开始考试"));
                }
            }
        }

        let mut model: exam_sessions::ActiveModel = selected.into();
        model.status = Set("active".to_string());
        model.started_at = Set(Some(start_time));
        model.ends_at = Set(end_time);
        model.updated_at = Set(now_ms());
        model.update(&state.db).await?;
>>>>>>> v0.1.1-dev

        Ok(true)
    }

    /// 将指定考试会话标记为已结束。
    ///
    /// # 参数
    /// * `app_handle` - Tauri 应用句柄。
    /// * `exam_id` - 考试 ID。
    /// * `student_id` - 学生 ID。
    /// * `end_time` - 结束时间戳（毫秒）。
    ///
    /// # 返回值
    /// 若命中本地会话并成功更新状态，返回 `Ok(true)`；否则返回 `Ok(false)`。
    pub async fn mark_exam_ended(
        app_handle: &tauri::AppHandle,
        exam_id: &str,
        student_id: &str,
        end_time: i64,
    ) -> Result<bool> {
        let state = app_handle.state::<crate::state::AppState>();
        let session = exam_session_repo::get_session_by_exam_and_student(&state.db, exam_id, student_id)
            .await?;

        let Some(session) = session else {
            return Ok(false);
        };

        exam_session_repo::mark_session_ended(&state.db, session, end_time, now_ms()).await?;

        Ok(true)
    }

    pub async fn get_current_exam_bundle(
        app_handle: &tauri::AppHandle,
    ) -> Result<CurrentExamBundleDto> {
        let state = app_handle.state::<crate::state::AppState>();
        let sessions = exam_session_repo::get_all_sessions(&state.db).await?;

        let target_student_id = state
            .reconnect_target()
            .map(|(_, student_id)| student_id)
            .filter(|id| !id.trim().is_empty());

        let candidate_sessions: Vec<_> = if let Some(student_id) = target_student_id {
            let filtered: Vec<_> = sessions
                .iter()
                .filter(|item| item.student_id == student_id)
                .cloned()
                .collect();
            if filtered.is_empty() {
                sessions.clone()
            } else {
                filtered
            }
        } else {
            sessions.clone()
        };

        let Some(default_session) = candidate_sessions.first().cloned() else {
            return Ok(CurrentExamBundleDto {
                session: None,
                snapshot: None,
            });
        };

        // Prefer the most recently updated session that already has a snapshot.
        let mut selected_session = default_session;
        let mut selected_snapshot: Option<_> = None;

        for session in candidate_sessions {
            let snapshot = exam_snapshot_repo::get_snapshot_by_session_id(&state.db, &session.id)
                .await?;
            if snapshot.is_some() {
                selected_session = session;
                selected_snapshot = snapshot;
                break;
            }
        }

        if selected_snapshot.is_none() {
            selected_snapshot = exam_snapshot_repo::get_snapshot_by_session_id(&state.db, &selected_session.id)
                .await?;
        }

        Ok(CurrentExamBundleDto {
<<<<<<< fix-规范student代码分层
            session: Some(exam_session_repo::session_to_dto(selected_session)),
            snapshot: selected_snapshot.map(exam_snapshot_repo::snapshot_to_dto),
=======
            session: Some(ExamSessionDto {
                id: selected_session.id.clone(),
                exam_id: selected_session.exam_id,
                student_id: selected_session.student_id,
                student_no: selected_session.student_no,
                student_name: selected_session.student_name,
                assigned_ip_addr: selected_session.assigned_ip_addr,
                assigned_device_name: selected_session.assigned_device_name,
                exam_title: selected_session.exam_title,
                status: selected_session.status,
                assignment_status: selected_session.assignment_status,
                started_at: selected_session.started_at,
                ends_at: selected_session.ends_at,
                paper_version: selected_session.paper_version,
                last_synced_at: selected_session.last_synced_at,
                created_at: selected_session.created_at,
                updated_at: selected_session.updated_at,
            }),
            snapshot: selected_snapshot.map(|item| ExamSnapshotDto {
                session_id: item.session_id,
                exam_meta: String::from_utf8_lossy(&item.exam_meta).to_string(),
                questions_payload: String::from_utf8_lossy(&item.questions_payload).to_string(),
                downloaded_at: item.downloaded_at,
                expires_at: item.expires_at,
                package_path: item.package_path,
                package_status: item.package_status,
                package_batch_id: item.package_batch_id,
                package_sha256: item.package_sha256,
                package_received_at: item.package_received_at,
                assets_sync_status: item.assets_sync_status,
                assets_synced_at: item.assets_synced_at,
                updated_at: item.updated_at,
            }),
>>>>>>> v0.1.1-dev
        })
    }
}

fn parse_questions_payload_from_xlsx(
    xlsx_path: &Path,
    asset_mapping: &HashMap<String, String>,
) -> Result<Vec<serde_json::Value>> {
    let mut workbook = open_workbook_auto(xlsx_path)?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| anyhow::anyhow!("xlsx 中不存在工作表"))??;

    let mut rows = range.rows();
    let headers = rows
        .next()
        .ok_or_else(|| anyhow::anyhow!("xlsx 表头为空"))?
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<String>>();

    let mut payloads = Vec::new();
    for (idx, row) in rows.enumerate() {
        let row_map = headers
            .iter()
            .enumerate()
            .map(|(col_idx, header)| {
                let value = row.get(col_idx).map(|v| v.to_string()).unwrap_or_default();
                (header.to_string(), value)
            })
            .collect::<HashMap<String, String>>();

        let seq = row_map
            .get("序号")
            .or_else(|| row_map.get("seq"))
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or((idx + 1) as i32);
        let question_id = row_map
            .get("题目ID")
            .or_else(|| row_map.get("id"))
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let qtype = row_map
            .get("题型")
            .or_else(|| row_map.get("type"))
            .cloned()
            .unwrap_or_default();
        let content = row_map
            .get("题目内容")
            .or_else(|| row_map.get("题干"))
            .or_else(|| row_map.get("content"))
            .cloned()
            .unwrap_or_default();
        let answer = row_map
            .get("答案")
            .or_else(|| row_map.get("answer"))
            .cloned()
            .unwrap_or_default();
        if qtype.trim().is_empty() || content.trim().is_empty() || answer.trim().is_empty() {
            continue;
        }

        let content_images_raw = row_map
            .get("题干图片")
            .or_else(|| row_map.get("content_image_paths"))
            .cloned()
            .unwrap_or_default();
        let content_images: Vec<String> = serde_json::from_str::<Vec<String>>(&content_images_raw)
            .unwrap_or_default()
            .into_iter()
            .map(|v| {
                let normalized = v.replace('\\', "/");
                asset_mapping.get(&normalized).cloned().unwrap_or(normalized)
            })
            .collect();

        let options_raw = row_map
            .get("选项")
            .or_else(|| row_map.get("options"))
            .cloned()
            .unwrap_or_default();

        let options_value = if let Ok(mut options) = serde_json::from_str::<Vec<serde_json::Value>>(&options_raw) {
            for option in &mut options {
                let images = option
                    .get("image_paths")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let mapped = images
                    .into_iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .map(|v| {
                        let normalized = v.replace('\\', "/");
                        asset_mapping.get(&normalized).cloned().unwrap_or(normalized)
                    })
                    .map(serde_json::Value::String)
                    .collect::<Vec<_>>();
                if let Some(obj) = option.as_object_mut() {
                    obj.insert("image_paths".to_string(), serde_json::Value::Array(mapped));
                }
            }
            serde_json::to_string(&options).unwrap_or_default()
        } else {
            options_raw
        };

        payloads.push(serde_json::json!({
            "id": question_id,
            "seq": seq,
            "type": qtype,
            "content": content,
            "contentImagePaths": content_images,
            "options": options_value,
            "answer": answer,
            "score": row_map
                .get("分值")
                .or_else(|| row_map.get("score"))
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(0),
            "explanation": row_map
                .get("解析")
                .or_else(|| row_map.get("explanation"))
                .cloned()
                .unwrap_or_default()
        }));
    }

    Ok(payloads)
}
