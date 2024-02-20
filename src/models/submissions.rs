use sqlx::PgExecutor;

#[derive(sqlx::Type)]
#[sqlx(type_name = "submission_decision", rename_all = "lowercase")]
pub enum SubmissionDecision {
    Approved,
    Rejected,
}

#[derive(sqlx::FromRow)]
pub struct Submission {
    pub submission_id: i32,
    pub user_id: i64,

    pub artist: String,
    pub art_link: String,
    pub additional_information: Option<String>,

    pub sample_image_url: Option<String>,

    pub decision: Option<SubmissionDecision>,

    #[sqlx(default)]
    pub submission_date: chrono::NaiveDateTime,
    pub submission_decision_date: Option<chrono::NaiveDateTime>,
}

pub struct AddSubmission {
    pub user_id: u64,
    pub artist: String,
    pub art_link: String,
    pub additional_information: Option<String>,
    pub sample_image_url: Option<String>,
}

impl Default for Submission {
    fn default() -> Self {
        Self {
            submission_id: 0,
            user_id: 0,

            artist: String::default(),
            art_link: String::default(),
            additional_information: None,

            sample_image_url: None,

            decision: None,

            submission_date: chrono::Utc::now().naive_utc(),
            submission_decision_date: None,
        }
    }
}

// UserId might be used later, so let it stay.
// TODO: Remove if it has never been used for some time.
#[allow(dead_code)]
pub enum SubmissionIds {
    SubmissionId(i32),
    UserId(u64),
}

pub trait SubmissionHelpers {
    async fn add_submission(
        executor: impl PgExecutor,
        submission: AddSubmission,
    ) -> anyhow::Result<Submission>;

    // async fn remove_submission(
    //     database: &PgPool,
    //     submission_id: SubmissionId,
    // ) -> anyhow::Result<()>;

    async fn approve_submission(
        executor: impl PgExecutor,
        approve_submission: SubmissionIds,
    ) -> anyhow::Result<Submission>;

    async fn reject_submission(
        executor: impl PgExecutor,
        submission_id: SubmissionIds,
    ) -> anyhow::Result<Submission>;
}

impl SubmissionHelpers for Submission {
    async fn add_submission(
        executor: impl PgExecutor<'_>,
        submission: AddSubmission,
    ) -> anyhow::Result<Submission> {
        debug!("adding a new submission");

        let created_submission = sqlx::query_as!(
            Submission,
            r#"INSERT INTO submissions(user_id, artist, art_link, additional_information, sample_image_url)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING submission_id, user_id, artist, art_link, additional_information, sample_image_url, decision as "decision: SubmissionDecision", submission_date, submission_decision_date"#,
            submission.user_id as i64,
            submission.artist,
            submission.art_link,
            submission.additional_information,
            submission.sample_image_url
        )
            .fetch_one(executor)
            .await?;

        debug!(
            "added a new submission with: `submission_id`: {}",
            created_submission.submission_id
        );
        Ok(created_submission)
    }

    async fn approve_submission(
        executor: impl PgExecutor<'_>,
        approve_submission: SubmissionIds,
    ) -> anyhow::Result<Submission> {
        debug!("approving a submission");

        let approved_submission = match approve_submission {
            SubmissionIds::SubmissionId(submission_id) => {
                sqlx::query_as!(
                    Submission,
                    r#"UPDATE submissions SET decision = 'approved', submission_decision_date = NOW() WHERE submission_id = $1
                    RETURNING submission_id, user_id, artist, art_link, additional_information, sample_image_url, decision as "decision: SubmissionDecision", submission_date, submission_decision_date"#,
                    submission_id
                )
                .fetch_one(executor)
                .await?
            }
            SubmissionIds::UserId(user_id) => {
                sqlx::query_as!(
                    Submission,
                    r#"UPDATE submissions SET decision = 'approved', submission_decision_date = NOW() WHERE user_id = $1
                    RETURNING submission_id, user_id, artist, art_link, additional_information, sample_image_url, decision as "decision: SubmissionDecision", submission_date, submission_decision_date"#,
                    user_id as i64
                )
                .fetch_one(executor)
                .await?
            }
        };

        debug!(
            "approved a submission with: `submission_id`: {}",
            approved_submission.submission_id
        );
        Ok(approved_submission)
    }

    async fn reject_submission(
        executor: impl PgExecutor<'_>,
        submission_id: SubmissionIds,
    ) -> anyhow::Result<Submission> {
        debug!("rejecting a submission");

        let rejected_submission = match submission_id {
            SubmissionIds::SubmissionId(submission_id) => {
                sqlx::query_as!(
                    Submission,
                    r#"UPDATE submissions SET decision = 'rejected', submission_decision_date = NOW() WHERE submission_id = $1
                    RETURNING submission_id, user_id, artist, art_link, additional_information, sample_image_url, decision as "decision: SubmissionDecision", submission_date, submission_decision_date"#,
                    submission_id
                )
                .fetch_one(executor)
                .await?
            }
            SubmissionIds::UserId(user_id) => {
                sqlx::query_as!(
                    Submission,
                    r#"UPDATE submissions SET decision = 'rejected', submission_decision_date = NOW() WHERE user_id = $1
                    RETURNING submission_id, user_id, artist, art_link, additional_information, sample_image_url, decision as "decision: SubmissionDecision", submission_date, submission_decision_date"#,
                    user_id as i64
                )
                .fetch_one(executor)
                .await?
            }
        };

        debug!(
            "rejected a submission with: `submission_id`: {}, `user_id`: {}",
            rejected_submission.submission_id, rejected_submission.user_id
        );
        Ok(rejected_submission)
    }
}
