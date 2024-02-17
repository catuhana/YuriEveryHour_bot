CREATE TABLE IF NOT EXISTS submissions (
    submission_id SERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,

    artist TEXT NOT NULL,
    art_link TEXT NOT NULL,
    additional_information TEXT,

    sample_image_url TEXT,

    approved BOOLEAN NOT NULL DEFAULT FALSE,

    submission_date TIMESTAMP NOT NULL DEFAULT NOW(),
    submission_accept_date TIMESTAMP
);

CREATE TABLE IF NOT EXISTS votes (
    vote_id SERIAL PRIMARY KEY,
    submission_id INTEGER NOT NULL,
    user_id BIGINT NOT NULL,

    vote BOOLEAN NOT NULL,
    vote_date TIMESTAMP NOT NULL DEFAULT NOW(),

    FOREIGN KEY (submission_id) REFERENCES submissions(submission_id)
);

CREATE TABLE IF NOT EXISTS images (
    image_id SERIAL PRIMARY KEY,
    submission_id INTEGER NOT NULL,
    vote_id INTEGER NOT NULL,

    image_path TEXT NOT NULL,

    FOREIGN KEY (submission_id) REFERENCES submissions(submission_id),
    FOREIGN KEY (vote_id) REFERENCES votes(vote_id)
);
