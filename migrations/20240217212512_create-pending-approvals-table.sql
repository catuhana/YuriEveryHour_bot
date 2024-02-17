ALTER TABLE submissions
  DROP COLUMN pending_approval;

CREATE TABLE IF NOT EXISTS pending_approvals (
   submission_id INTEGER PRIMARY KEY,
   message_id BIGINT NOT NULL,
   FOREIGN KEY (submission_id) REFERENCES submissions(submission_id)
);
