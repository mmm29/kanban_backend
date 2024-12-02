CREATE TABLE users (
    user_id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL
);

CREATE TABLE sessions (
    user_id INT NOT NULL,
    token VARCHAR(64) PRIMARY KEY,
    FOREIGN KEY (user_id) REFERENCES users (user_id)
);

CREATE TABLE task_categories (
    category_id VARCHAR(64) PRIMARY KEY,
    user_id INT NOT NULL,
    label VARCHAR(64) NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (user_id)
);

CREATE TABLE tasks (
    task_id VARCHAR(64) PRIMARY KEY,
    user_id INT NOT NULL,
    category_id VARCHAR(64) NOT NULL,
    label TEXT NOT NULL,
    description TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (user_id),
    FOREIGN KEY (category_id) REFERENCES task_categories (category_id)
);