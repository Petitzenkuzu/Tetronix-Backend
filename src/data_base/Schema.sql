CREATE TABLE Users (
    name VARCHAR(255) Primary Key,
    number_of_games INT DEFAULT 0,
    best_score INT DEFAULT 0,
    highest_level INT DEFAULT 0
);

CREATE TABLE Sessions (
    name VARCHAR(255) NOT NULL,
    session_id VARCHAR(255) Primary Key
);

