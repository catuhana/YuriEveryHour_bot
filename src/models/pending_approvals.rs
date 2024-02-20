use sqlx::PgExecutor;

#[derive(Debug, PartialEq, Eq)]
pub struct PendingApproval {
    pub submission_id: i32,
    pub message_id: i64,
    pub date: chrono::NaiveDateTime,
}

#[derive(Debug)]
pub struct AddPendingApproval {
    pub submission_id: i32,
    pub message_id: u64,
}

pub enum RemovePendingApproval {
    SubmissionId(i32),
    MessageId(u64),
}

pub trait PendingApprovalHelpers {
    async fn add_pending_approval(
        executor: impl PgExecutor,
        add_pending_approval: AddPendingApproval,
    ) -> anyhow::Result<PendingApproval>;

    async fn remove_pending_approval(
        executor: impl PgExecutor,
        remove_pending_approval: RemovePendingApproval,
    ) -> anyhow::Result<PendingApproval>;

    async fn remove_expired_approvals(
        executor: impl PgExecutor,
    ) -> anyhow::Result<Vec<PendingApproval>>;
}

impl PendingApprovalHelpers for PendingApproval {
    async fn add_pending_approval(
        executor: impl PgExecutor<'_>,
        add_pending_approval: AddPendingApproval,
    ) -> anyhow::Result<Self> {
        debug!("adding a new pending approval");

        let added_approval =
            sqlx::query_as!(PendingApproval,
            "INSERT INTO pending_approvals (submission_id, message_id) VALUES ($1, $2) RETURNING *",
            add_pending_approval.submission_id,
            add_pending_approval.message_id as i64,
        )
            .fetch_one(executor)
            .await?;

        debug!(
            "added a new pending approval with: `submission_id`: {submission_id}, `message_id`: {message_id}",
            submission_id = added_approval.submission_id,
            message_id = added_approval.message_id
        );
        Ok(added_approval)
    }

    async fn remove_pending_approval(
        executor: impl PgExecutor<'_>,
        remove_pending_approval: RemovePendingApproval,
    ) -> anyhow::Result<Self> {
        debug!("removing a pending approval");
        let removed_approval = match remove_pending_approval {
            RemovePendingApproval::MessageId(message_id) => {
                sqlx::query_as!(
                    PendingApproval,
                    "DELETE FROM pending_approvals WHERE message_id = $1 RETURNING *",
                    message_id as i64
                )
                .fetch_one(executor)
                .await?
            }
            RemovePendingApproval::SubmissionId(submission_id) => {
                sqlx::query_as!(
                    PendingApproval,
                    "DELETE FROM pending_approvals WHERE submission_id = $1 RETURNING *",
                    submission_id
                )
                .fetch_one(executor)
                .await?
            }
        };

        debug!(
            "removed a pending approval with: `submission_id`: {submission_id}, `message_id`: {message_id}",
            submission_id = removed_approval.submission_id,
            message_id = removed_approval.message_id
        );
        Ok(removed_approval)
    }

    async fn remove_expired_approvals(executor: impl PgExecutor<'_>) -> anyhow::Result<Vec<Self>> {
        debug!("removing expired approvals");

        let expired_approvals = sqlx::query_as!(
            PendingApproval,
            r#"
            WITH deleted_approvals AS
                (DELETE FROM pending_approvals WHERE date < NOW() - INTERVAL '1 day' RETURNING *)
            SELECT * FROM deleted_approvals
            "#
        )
        .fetch_all(executor)
        .await?;

        debug!(
            "removed expired approvals with: `submission_id`s: {submission_ids}",
            submission_ids = expired_approvals
                .iter()
                .map(|approval| approval.submission_id.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );
        Ok(expired_approvals)
    }
}

type PendingApprovals = Vec<PendingApproval>;

pub trait PendingApprovalsHelpers {
    async fn add_pending_approval(
        &mut self,
        executor: impl PgExecutor,
        add_pending_approval: AddPendingApproval,
    ) -> anyhow::Result<()>;

    async fn remove_pending_approval(
        &mut self,
        executor: impl PgExecutor,
        remove_pending_approval: RemovePendingApproval,
    ) -> anyhow::Result<()>;

    async fn populate_pending_approvals(&mut self, executor: impl PgExecutor)
        -> anyhow::Result<()>;

    // async fn depopulate_expired_approvals(
    //     &mut self,
    //     executor: impl PgExecutor,
    // ) -> anyhow::Result<()>;
}

impl PendingApprovalsHelpers for PendingApprovals {
    async fn add_pending_approval(
        &mut self,
        executor: impl PgExecutor<'_>,
        add_pending_approval: AddPendingApproval,
    ) -> anyhow::Result<()> {
        let added_approval =
            PendingApproval::add_pending_approval(executor, add_pending_approval).await?;
        self.push(added_approval);

        Ok(())
    }

    async fn remove_pending_approval(
        &mut self,
        executor: impl PgExecutor<'_>,
        remove_approval: RemovePendingApproval,
    ) -> anyhow::Result<()> {
        let removed_approval =
            PendingApproval::remove_pending_approval(executor, remove_approval).await?;
        self.retain(|pending_approval| *pending_approval != removed_approval);

        Ok(())
    }

    async fn populate_pending_approvals(
        &mut self,
        executor: impl PgExecutor<'_>,
    ) -> anyhow::Result<()> {
        debug!("populating pending approvals");

        self.extend(
            sqlx::query_as!(PendingApproval, "SELECT * FROM pending_approvals")
                .fetch_all(executor)
                .await?,
        );

        debug!("populated pending approvals");
        Ok(())
    }

    // async fn depopulate_expired_approvals(
    //     &mut self,
    //     executor: impl PgExecutor<'_>,
    // ) -> anyhow::Result<()> {
    //     debug!("depopulating expired approvals");

    //     let deleted_approvals = PendingApproval::remove_expired_approvals(executor).await?;
    //     self.retain(|approval| !deleted_approvals.contains(approval));

    //     debug!("depopulated expired approvals");
    //     Ok(())
    // }
}
