-- Your SQL goes here
CREATE TABLE users_tasks (
  user_id INTEGER REFERENCES users(id),
  task_id INTEGER REFERENCES tasks(id),
  done_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  PRIMARY KEY(user_id, task_id)
);