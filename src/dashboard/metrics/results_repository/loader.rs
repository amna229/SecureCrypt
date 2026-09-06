//! Evaluation results loading.
//!
//! This module loads and aggregates evaluation, handshake, transfer and
//! resource data from either PostgreSQL or the temporary in-memory backend.

use crate::dashboard::database::storage::DashboardStorage;

use chrono::{Local, NaiveDate};

use sqlx::query_as;

use std::collections::HashMap;

use uuid::Uuid;

use super::{
    DerivedMetrics, EvaluationGroup, EvaluationSummary, GroupKey, ResultsData, ResultsRepository,
    build_handshake_summary, build_resource_summary, build_transfer_summary, calculate_derived,
};

impl ResultsRepository {
    /// Loads and aggregates evaluation results for a given date.
    ///
    /// PostgreSQL uses the existing SQL-based loading process.
    /// The in-memory backend reconstructs the same result structure
    /// directly from the data collected during the current execution.
    pub async fn load(&self, date: NaiveDate) -> Result<ResultsData, sqlx::Error> {
        match &self.storage {
            DashboardStorage::Postgres(pool) => self.load_from_postgres(pool, date).await,

            DashboardStorage::Memory(_) => self.load_from_memory(date).await,
        }
    }

    /// Loads results from PostgreSQL.
    async fn load_from_postgres(
        &self,
        pool: &sqlx::PgPool,
        date: NaiveDate,
    ) -> Result<ResultsData, sqlx::Error> {
        let time_zone = std::env::var("TIME_ZONE").unwrap_or_else(|_| "Europe/Madrid".to_string());

        let evaluation_rows = query_as::<
            _,
            (
                Uuid,
                String,
                i32,
                chrono::DateTime<chrono::Utc>,
                Option<chrono::DateTime<chrono::Utc>>,
                String,
                String,
                i64,
                String,
                i32,
            ),
        >(
            "SELECT DISTINCT
                    e.id,
                    e.crypto_mode,
                    e.num_clients,
                    e.started_at,
                    e.finished_at,
                    e.status,
                    t.operation,
                    t.file_size,
                    t.file_size_unit,
                    t.num_files
             FROM evaluation e
             JOIN transfer t
               ON t.evaluation_id = e.id
             WHERE e.started_at >= (
                 $1::date::timestamp AT TIME ZONE $2
             )
               AND e.started_at < (
                 ($1::date + INTERVAL '1 day')::timestamp
                 AT TIME ZONE $2
             )
             ORDER BY
                 e.started_at DESC,
                 t.operation,
                 t.file_size,
                 t.file_size_unit,
                 t.num_files",
        )
        .bind(date)
        .bind(&time_zone)
        .fetch_all(pool)
        .await?;

        let mut groups: HashMap<GroupKey, EvaluationGroup> = HashMap::new();

        let mut evaluation_groups: HashMap<Uuid, Vec<GroupKey>> = HashMap::new();

        let mut evaluation_modes: HashMap<Uuid, String> = HashMap::new();

        for (
            id,
            crypto_mode,
            num_clients,
            started_at,
            finished_at,
            status,
            operation,
            file_size,
            file_size_unit,
            num_files,
        ) in evaluation_rows
        {
            let key = GroupKey {
                num_clients,
                operation: operation.clone(),
                file_size,
                file_size_unit: file_size_unit.clone(),
                num_files,
            };

            groups
                .entry(key.clone())
                .or_insert_with(|| EvaluationGroup {
                    num_clients,
                    operation: operation.clone(),
                    file_size,
                    file_size_unit: file_size_unit.clone(),
                    num_files,
                    evaluations: Vec::new(),
                    handshakes: Vec::new(),
                    transfers: Vec::new(),
                    resources: Vec::new(),
                    derived: DerivedMetrics {
                        handshake_overhead_percent: None,
                        transfer_overhead_percent: None,
                        cpu_overhead_percent: None,
                        memory_overhead_percent: None,
                        throughput_change_percent: None,
                    },
                });

            let evaluation_entry = evaluation_groups.entry(id).or_default();

            if !evaluation_entry.contains(&key) {
                evaluation_entry.push(key.clone());
            }

            evaluation_modes
                .entry(id)
                .or_insert_with(|| crypto_mode.clone());

            if let Some(group) = groups.get_mut(&key) {
                let already_present = group
                    .evaluations
                    .iter()
                    .any(|evaluation| evaluation.id == id);

                if !already_present {
                    group.evaluations.push(EvaluationSummary {
                        id,
                        crypto_mode,
                        num_clients,
                        started_at,
                        finished_at,
                        status,
                    });
                }
            }
        }

        let handshake_rows = query_as::<_, (Uuid, String, i64)>(
            "SELECT
                    h.evaluation_id,
                    h.kx_group,
                    h.handshake_duration_ms
                 FROM handshake h
                 JOIN evaluation e
                   ON e.id = h.evaluation_id
                 WHERE h.success = TRUE
                   AND h.kx_group IS NOT NULL
                   AND e.started_at >= (
                       $1::date::timestamp AT TIME ZONE $2
                   )
                   AND e.started_at < (
                       ($1::date + INTERVAL '1 day')::timestamp
                       AT TIME ZONE $2
                   )",
        )
        .bind(date)
        .bind(&time_zone)
        .fetch_all(pool)
        .await?;

        let mut handshake_values: HashMap<(GroupKey, String, String), Vec<i64>> = HashMap::new();

        for (evaluation_id, kx_group, duration_ms) in handshake_rows {
            let keys = match evaluation_groups.get(&evaluation_id) {
                Some(keys) => keys,
                None => continue,
            };

            let crypto_mode = match evaluation_modes.get(&evaluation_id) {
                Some(mode) => mode,
                None => continue,
            };

            for key in keys {
                handshake_values
                    .entry((key.clone(), crypto_mode.clone(), kx_group.clone()))
                    .or_default()
                    .push(duration_ms);
            }
        }

        for ((key, crypto_mode, kx_group), values) in handshake_values {
            if let Some(group) = groups.get_mut(&key) {
                group
                    .handshakes
                    .push(build_handshake_summary(crypto_mode, kx_group, values));
            }
        }

        let transfer_rows = query_as::<_, (Uuid, String, i64, String, i32, String, i64, i64, f64)>(
            "SELECT
                    t.evaluation_id,
                    t.operation,
                    t.file_size,
                    t.file_size_unit,
                    t.num_files,
                    COALESCE(t.kx_group, 'unknown'),
                    t.duration_ms,
                    COALESCE(t.bytes_transferred, 0),
                    t.throughput_mbps
                 FROM transfer t
                 JOIN evaluation e
                   ON e.id = t.evaluation_id
                 WHERE t.success = TRUE
                   AND t.duration_ms IS NOT NULL
                   AND t.throughput_mbps IS NOT NULL
                   AND e.started_at >= (
                       $1::date::timestamp AT TIME ZONE $2
                   )
                   AND e.started_at < (
                       ($1::date + INTERVAL '1 day')::timestamp
                       AT TIME ZONE $2
                   )",
        )
        .bind(date)
        .bind(&time_zone)
        .fetch_all(pool)
        .await?;

        let mut transfer_values: HashMap<(GroupKey, String, String), Vec<(i64, i64, f64)>> =
            HashMap::new();

        for (
            evaluation_id,
            operation,
            file_size,
            file_size_unit,
            num_files,
            kx_group,
            duration_ms,
            bytes_transferred,
            throughput_mbps,
        ) in transfer_rows
        {
            let num_clients = match evaluation_groups
                .get(&evaluation_id)
                .and_then(|keys| keys.first())
                .map(|key| key.num_clients)
            {
                Some(value) => value,
                None => continue,
            };

            let crypto_mode = match evaluation_modes.get(&evaluation_id) {
                Some(mode) => mode.clone(),
                None => continue,
            };

            let key = GroupKey {
                num_clients,
                operation,
                file_size,
                file_size_unit,
                num_files,
            };

            if !groups.contains_key(&key) {
                continue;
            }

            transfer_values
                .entry((key, crypto_mode, kx_group))
                .or_default()
                .push((duration_ms, bytes_transferred, throughput_mbps));
        }

        for ((key, crypto_mode, kx_group), values) in transfer_values {
            if let Some(group) = groups.get_mut(&key) {
                group
                    .transfers
                    .push(build_transfer_summary(crypto_mode, kx_group, values));
            }
        }

        let resource_rows = query_as::<_, (Uuid, String, f64, f64, i64)>(
            "SELECT
                    r.evaluation_id,
                    r.role,
                    r.cpu_avg_percent,
                    r.cpu_peak_percent,
                    r.memory_peak_bytes
                 FROM resource r
                 JOIN evaluation e
                   ON e.id = r.evaluation_id
                 WHERE e.started_at >= (
                     $1::date::timestamp AT TIME ZONE $2
                 )
                   AND e.started_at < (
                       ($1::date + INTERVAL '1 day')::timestamp
                       AT TIME ZONE $2
                   )",
        )
        .bind(date)
        .bind(&time_zone)
        .fetch_all(pool)
        .await?;

        let mut resource_values: HashMap<(GroupKey, String, String), Vec<(f64, f64, i64)>> =
            HashMap::new();

        for (evaluation_id, role, cpu_avg_percent, cpu_peak_percent, memory_peak_bytes) in
            resource_rows
        {
            let keys = match evaluation_groups.get(&evaluation_id) {
                Some(keys) => keys,
                None => continue,
            };

            let crypto_mode = match evaluation_modes.get(&evaluation_id) {
                Some(mode) => mode.clone(),
                None => continue,
            };

            for key in keys {
                resource_values
                    .entry((key.clone(), crypto_mode.clone(), role.clone()))
                    .or_default()
                    .push((cpu_avg_percent, cpu_peak_percent, memory_peak_bytes));
            }
        }

        for ((key, crypto_mode, role), values) in resource_values {
            if let Some(group) = groups.get_mut(&key) {
                group
                    .resources
                    .push(build_resource_summary(crypto_mode, role, values));
            }
        }

        self.finalize_groups(groups)
    }

    /// Loads results from the temporary in-memory backend.
    /// Loads results from the temporary in-memory backend.
    async fn load_from_memory(&self, date: NaiveDate) -> Result<ResultsData, sqlx::Error> {
        let (evaluations, handshakes, transfers, resources) = match &self.storage {
            DashboardStorage::Memory(database) => {
                let database = database.lock().await;

                (
                    database.evaluations.clone(),
                    database.handshakes.clone(),
                    database.transfers.clone(),
                    database.resources.clone(),
                )
            }

            DashboardStorage::Postgres(_) => {
                unreachable!()
            }
        };

        let evaluations: HashMap<Uuid, _> = evaluations
            .into_iter()
            .filter(|evaluation| evaluation.started_at.with_timezone(&Local).date_naive() == date)
            .map(|evaluation| (evaluation.id, evaluation))
            .collect();

        let mut groups: HashMap<GroupKey, EvaluationGroup> = HashMap::new();

        let mut evaluation_groups: HashMap<Uuid, Vec<GroupKey>> = HashMap::new();

        let mut evaluation_modes: HashMap<Uuid, String> = HashMap::new();

        /*
         * Creates groups from transfer configurations.
         */
        for transfer in &transfers {
            let evaluation = match evaluations.get(&transfer.evaluation_id) {
                Some(evaluation) => evaluation,
                None => continue,
            };

            let key = GroupKey {
                num_clients: evaluation.num_clients,
                operation: transfer.operation.clone(),
                file_size: transfer.file_size,
                file_size_unit: transfer.file_size_unit.clone(),
                num_files: transfer.num_files,
            };

            groups
                .entry(key.clone())
                .or_insert_with(|| EvaluationGroup {
                    num_clients: evaluation.num_clients,

                    operation: transfer.operation.clone(),

                    file_size: transfer.file_size,

                    file_size_unit: transfer.file_size_unit.clone(),

                    num_files: transfer.num_files,

                    evaluations: Vec::new(),

                    handshakes: Vec::new(),

                    transfers: Vec::new(),

                    resources: Vec::new(),

                    derived: DerivedMetrics {
                        handshake_overhead_percent: None,

                        transfer_overhead_percent: None,

                        cpu_overhead_percent: None,

                        memory_overhead_percent: None,

                        throughput_change_percent: None,
                    },
                });

            let entry = evaluation_groups.entry(evaluation.id).or_default();

            if !entry.contains(&key) {
                entry.push(key.clone());
            }

            evaluation_modes
                .entry(evaluation.id)
                .or_insert_with(|| evaluation.crypto_mode.clone());

            if let Some(group) = groups.get_mut(&key) {
                let already_present = group
                    .evaluations
                    .iter()
                    .any(|existing| existing.id == evaluation.id);

                if !already_present {
                    group.evaluations.push(EvaluationSummary {
                        id: evaluation.id,

                        crypto_mode: evaluation.crypto_mode.clone(),

                        num_clients: evaluation.num_clients,

                        started_at: evaluation.started_at,

                        finished_at: evaluation.finished_at,

                        status: evaluation.status.clone(),
                    });
                }
            }
        }

        /*
         * Evaluations that only contain handshake measurements
         * do not have a transfer configuration from which a group
         * could be created.
         *
         * Create a dedicated handshake group for them.
         */
        for handshake in &handshakes {
            if !handshake.success {
                continue;
            }

            if !evaluations.contains_key(&handshake.evaluation_id) {
                continue;
            }

            let evaluation = match evaluations.get(&handshake.evaluation_id) {
                Some(evaluation) => evaluation,
                None => continue,
            };

            let has_transfer_group = evaluation_groups
                .get(&evaluation.id)
                .map(|keys| !keys.is_empty())
                .unwrap_or(false);

            if has_transfer_group {
                continue;
            }

            let key = GroupKey {
                num_clients: evaluation.num_clients,

                operation: "handshake".to_string(),

                file_size: 0,

                file_size_unit: "B".to_string(),

                num_files: 0,
            };

            groups
                .entry(key.clone())
                .or_insert_with(|| EvaluationGroup {
                    num_clients: evaluation.num_clients,

                    operation: "handshake".to_string(),

                    file_size: 0,

                    file_size_unit: "B".to_string(),

                    num_files: 0,

                    evaluations: Vec::new(),

                    handshakes: Vec::new(),

                    transfers: Vec::new(),

                    resources: Vec::new(),

                    derived: DerivedMetrics {
                        handshake_overhead_percent: None,

                        transfer_overhead_percent: None,

                        cpu_overhead_percent: None,

                        memory_overhead_percent: None,

                        throughput_change_percent: None,
                    },
                });

            let entry = evaluation_groups.entry(evaluation.id).or_default();

            if !entry.contains(&key) {
                entry.push(key.clone());
            }

            evaluation_modes
                .entry(evaluation.id)
                .or_insert_with(|| evaluation.crypto_mode.clone());

            if let Some(group) = groups.get_mut(&key) {
                let already_present = group
                    .evaluations
                    .iter()
                    .any(|existing| existing.id == evaluation.id);

                if !already_present {
                    group.evaluations.push(EvaluationSummary {
                        id: evaluation.id,

                        crypto_mode: evaluation.crypto_mode.clone(),

                        num_clients: evaluation.num_clients,

                        started_at: evaluation.started_at,

                        finished_at: evaluation.finished_at,

                        status: evaluation.status.clone(),
                    });
                }
            }
        }

        /*
         * Aggregate handshake metrics.
         */
        let mut handshake_values: HashMap<(GroupKey, String, String), Vec<i64>> = HashMap::new();

        for handshake in handshakes {
            if !handshake.success {
                continue;
            }

            let duration_ms = match handshake.handshake_duration_ms {
                Some(value) => value,
                None => continue,
            };

            let kx_group = match handshake.kx_group {
                Some(value) => value,
                None => continue,
            };

            let keys = match evaluation_groups.get(&handshake.evaluation_id) {
                Some(keys) => keys,
                None => continue,
            };

            let crypto_mode = match evaluation_modes.get(&handshake.evaluation_id) {
                Some(mode) => mode.clone(),
                None => continue,
            };

            for key in keys {
                handshake_values
                    .entry((key.clone(), crypto_mode.clone(), kx_group.clone()))
                    .or_default()
                    .push(duration_ms);
            }
        }

        for ((key, crypto_mode, kx_group), values) in handshake_values {
            if let Some(group) = groups.get_mut(&key) {
                group
                    .handshakes
                    .push(build_handshake_summary(crypto_mode, kx_group, values));
            }
        }

        /*
         * Aggregate transfer metrics.
         */
        let mut transfer_values: HashMap<(GroupKey, String, String), Vec<(i64, i64, f64)>> =
            HashMap::new();

        for transfer in transfers {
            let evaluation = match evaluations.get(&transfer.evaluation_id) {
                Some(evaluation) => evaluation,
                None => continue,
            };

            if transfer.success != Some(true) {
                continue;
            }

            let duration_ms = match transfer.duration_ms {
                Some(value) => value,
                None => continue,
            };

            let throughput_mbps = match transfer.throughput_mbps {
                Some(value) => value,
                None => continue,
            };

            let bytes_transferred = transfer.bytes_transferred.unwrap_or(0);

            let kx_group = transfer.kx_group.unwrap_or_else(|| "unknown".to_string());

            let key = GroupKey {
                num_clients: evaluation.num_clients,

                operation: transfer.operation.clone(),

                file_size: transfer.file_size,

                file_size_unit: transfer.file_size_unit.clone(),

                num_files: transfer.num_files,
            };

            if !groups.contains_key(&key) {
                continue;
            }

            let crypto_mode = evaluation.crypto_mode.clone();

            transfer_values
                .entry((key, crypto_mode, kx_group))
                .or_default()
                .push((duration_ms, bytes_transferred, throughput_mbps));
        }

        for ((key, crypto_mode, kx_group), values) in transfer_values {
            if let Some(group) = groups.get_mut(&key) {
                group
                    .transfers
                    .push(build_transfer_summary(crypto_mode, kx_group, values));
            }
        }

        /*
         * Aggregate resource metrics.
         */
        let mut resource_values: HashMap<(GroupKey, String, String), Vec<(f64, f64, i64)>> =
            HashMap::new();

        for resource in resources {
            if !evaluations.contains_key(&resource.evaluation_id) {
                continue;
            }

            let keys = match evaluation_groups.get(&resource.evaluation_id) {
                Some(keys) => keys,
                None => continue,
            };

            let crypto_mode = match evaluation_modes.get(&resource.evaluation_id) {
                Some(mode) => mode.clone(),
                None => continue,
            };

            for key in keys {
                resource_values
                    .entry((key.clone(), crypto_mode.clone(), resource.role.clone()))
                    .or_default()
                    .push((
                        resource.cpu_avg_percent,
                        resource.cpu_peak_percent,
                        resource.memory_peak_bytes,
                    ));
            }
        }

        for ((key, crypto_mode, role), values) in resource_values {
            if let Some(group) = groups.get_mut(&key) {
                group
                    .resources
                    .push(build_resource_summary(crypto_mode, role, values));
            }
        }

        self.finalize_groups(groups)
    }

    /// Sorts groups and their summaries and calculates derived metrics.
    fn finalize_groups(
        &self,
        mut groups: HashMap<GroupKey, EvaluationGroup>,
    ) -> Result<ResultsData, sqlx::Error> {
        let mut groups = groups.into_values().collect::<Vec<_>>();

        for group in &mut groups {
            group
                .evaluations
                .sort_by(|a, b| b.started_at.cmp(&a.started_at));

            group.handshakes.sort_by(|a, b| {
                a.crypto_mode
                    .cmp(&b.crypto_mode)
                    .then(a.kx_group.cmp(&b.kx_group))
            });

            group.transfers.sort_by(|a, b| {
                a.crypto_mode
                    .cmp(&b.crypto_mode)
                    .then(a.kx_group.cmp(&b.kx_group))
            });

            group
                .resources
                .sort_by(|a, b| a.crypto_mode.cmp(&b.crypto_mode).then(a.role.cmp(&b.role)));

            group.derived = calculate_derived(group);
        }

        groups.sort_by(|a, b| {
            let a_date = a
                .evaluations
                .first()
                .map(|evaluation| evaluation.started_at);

            let b_date = b
                .evaluations
                .first()
                .map(|evaluation| evaluation.started_at);

            b_date.cmp(&a_date)
        });

        Ok(ResultsData { groups })
    }
}
