CREATE TRIGGER validate_task_status_on_insert
BEFORE INSERT ON tasks
WHEN NEW.status NOT IN ('todo', 'in_progress', 'done')
BEGIN
    SELECT RAISE(ABORT, 'invalid task status');
END;

CREATE TRIGGER validate_task_status_on_update
BEFORE UPDATE OF status ON tasks
WHEN NEW.status NOT IN ('todo', 'in_progress', 'done')
BEGIN
    SELECT RAISE(ABORT, 'invalid task status');
END;

CREATE INDEX idx_tasks_team_id ON tasks(team_id);
CREATE INDEX idx_tasks_owner_id ON tasks(owner_id);
CREATE INDEX idx_comments_task_id ON comments(task_id);
