use sqlx::PgExecutor;

use crate::models::pending_approvals::{
    AddPendingApproval, PendingApproval, PendingApprovalHelpers, RemovePendingApproval,
};

pub struct YuriData {
    pub pending_approvals: Vec<PendingApproval>,
}

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

impl PendingApprovalsHelpers for YuriData {
    async fn add_pending_approval(
        &mut self,
        executor: impl PgExecutor<'_>,
        add_pending_approval: AddPendingApproval,
    ) -> anyhow::Result<()> {
        let added_approval =
            PendingApproval::add_pending_approval(executor, add_pending_approval).await?;
        self.pending_approvals.push(added_approval);

        Ok(())
    }

    async fn remove_pending_approval(
        &mut self,
        executor: impl PgExecutor<'_>,
        remove_approval: RemovePendingApproval,
    ) -> anyhow::Result<()> {
        let removed_approval =
            PendingApproval::remove_pending_approval(executor, remove_approval).await?;
        self.pending_approvals
            .retain(|pending_approval| *pending_approval != removed_approval);

        Ok(())
    }

    async fn populate_pending_approvals(
        &mut self,
        executor: impl PgExecutor<'_>,
    ) -> anyhow::Result<()> {
        debug!("populating pending approvals");

        let pending_approvals = sqlx::query_as!(PendingApproval, "SELECT * FROM pending_approvals")
            .fetch_all(executor)
            .await?;

        if pending_approvals.is_empty() {
            debug!("no pending approvals found to populate");
            return Ok(());
        }

        self.pending_approvals.extend(pending_approvals);

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
