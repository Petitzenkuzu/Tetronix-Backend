CREATE TABLE Users (
    name VARCHAR(255) Primary Key CHECK (length(name) > 1),
    number_of_games INT DEFAULT 0 CHECK (number_of_games >= 0),
    best_score INT DEFAULT 0 CHECK (best_score >= 0),
    highest_level INT DEFAULT 0 CHECK (highest_level >= 0)
);

CREATE TABLE Sessions (
    name VARCHAR(255) NOT NULL,
    session_id VARCHAR(255) Primary Key,
    FOREIGN KEY (name) REFERENCES Users(name) ON DELETE CASCADE
);

CREATE TABLE Games (
    game_owner VARCHAR(255) Primary Key,
    game_score INT NOT NULL DEFAULT 0 CHECK (game_score >= 0),
    game_level INT NOT NULL DEFAULT 0 CHECK (game_level >= 0),
    game_lines INT NOT NULL DEFAULT 0 CHECK (game_lines >= 0),
    game_actions JSONB NOT NULL,
    FOREIGN KEY (game_owner) REFERENCES Users(name) ON DELETE CASCADE
);

-- Indexes pour un max de vitesse ( ça fait une sorte de table fantôme qui stock les meilleurs scores des user donc get leaderboard est plus rapide)
CREATE INDEX idx_users_best_score ON Users(best_score DESC);