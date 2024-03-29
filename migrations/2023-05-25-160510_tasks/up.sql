-- Your SQL goes here

CREATE TABLE IF NOT EXISTS tasks (
  id SERIAL PRIMARY KEY,
  task_name VARCHAR NOT NULL,
  task_description VARCHAR DEFAULT NULL,
  task_score INTEGER DEFAULT 1 NOT NULL,
  task_priority INTEGER DEFAULT 1 NOT NULL,
  hashtag VARCHAR DEFAULT '' NOT NULL,
  tweet_content VARCHAR DEFAULT '' NOT NULL,
  retweet_id VARCHAR DEFAULT '' NOT NULL,
  like_tweet_id VARCHAR DEFAULT '' NOT NULL,
  admin_id SERIAL REFERENCES users(id),
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

SELECT diesel_manage_updated_at('tasks');