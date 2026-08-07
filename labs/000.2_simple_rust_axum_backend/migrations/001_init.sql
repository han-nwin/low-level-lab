CREATE TABLE users (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE teams (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE team_members (
    team_id TEXT NOT NULL,
    user_id TEXT NOT NULL,

    PRIMARY KEY(team_id,user_id),

    FOREIGN KEY(team_id)
        REFERENCES teams(id),

    FOREIGN KEY(user_id)
        REFERENCES users(id)
);


CREATE TABLE tasks (
    id TEXT PRIMARY KEY,

    team_id TEXT NOT NULL,

    title TEXT NOT NULL,
    description TEXT,

    status TEXT NOT NULL DEFAULT 'todo',

    owner_id TEXT,

    due_date DATETIME,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,


    FOREIGN KEY(team_id)
        REFERENCES teams(id),

    FOREIGN KEY(owner_id)
        REFERENCES users(id)
);


CREATE TABLE comments (
    id TEXT PRIMARY KEY,

    task_id TEXT NOT NULL,

    user_id TEXT NOT NULL,

    body TEXT NOT NULL,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,


    FOREIGN KEY(task_id)
        REFERENCES tasks(id),

    FOREIGN KEY(user_id)
        REFERENCES users(id)
);
