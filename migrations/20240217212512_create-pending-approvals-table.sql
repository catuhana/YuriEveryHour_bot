ALTER TABLE submissions
    DROP COLUMN pending_approval;

CREATE TABLE IF NOT EXISTS pending_approvals (
    submission_id INTEGER,
    message_id BIGINT,

    PRIMARY KEY (submission_id, message_id),
    FOREIGN KEY (submission_id) REFERENCES submissions(submission_id)
);
