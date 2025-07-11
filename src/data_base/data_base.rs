use sqlx::{Pool,Postgres} ;

use crate::models::{User,Session, Game, GameJson, Action};
/*
    Return () si le user a été créé
    Return sqlx::Error::RowNotFound si le user n'a pas été créé
    Return sqlx::Error si c'est une erreur de base de données
*/
pub async fn create_user(pool: &Pool<Postgres>, name: &str) -> Result<(),sqlx::Error> {
    let result = sqlx::query("INSERT INTO users (name) VALUES ($1)")
        .bind(&name)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

/*
    Return sqlx::Error::RowNotFound si le user n'a pas été supprimé
    Return sqlx::Error si c'est une erreur de base de données
    Return () si le user a été supprimé
*/
pub async fn delete_user(pool: &Pool<Postgres>, name: &str) -> Result<(), sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE name = $1")
        .bind(name)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

/*
    Return sqlx::Error::RowNotFound si la session n'a pas été créée
    Return sqlx::Error si c'est une erreur de base de données
    Return () si la session a été créée
*/
pub async fn create_session(pool: &Pool<Postgres>, name: &str , session_id: &str) -> Result<(), sqlx::Error> {
    let result = sqlx::query("INSERT INTO sessions (name, session_id) VALUES ($1, $2)")
        .bind(name)
        .bind(session_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

/**
 *  Return sqlx::Error::RowNotFound si la session n'a pas été supprimée
 *  Return sqlx::Error si c'est une erreur de base de données
 *  Return () si la session a été supprimée 
 */
pub async fn delete_session(pool: &Pool<Postgres>, session_id: &str) -> Result<(), sqlx::Error> {
    let result = sqlx::query("DELETE FROM sessions WHERE session_id = $1")
        .bind(session_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

/*
    Return un User
    Return sqlx::Error::RowNotFound si le user n'existe pas
    Return sqlx::Error si c'est une erreur de base de données
*/
pub async fn get_user_from_name(pool: &Pool<Postgres>, name: &str) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE name = $1")
        .bind(name)
        .fetch_one(pool)
        .await?;

    Ok(user)
}

/*
    Return un User 
    Return sqlx::Error::RowNotFound si la session n'existe pas
    Return sqlx::Error::RowNotFound si le user n'existe pas
    Return sqlx::Error si c'est une erreur de base de données
*/
pub async fn get_user_from_session(pool: &Pool<Postgres>, session: &str) -> Result<User, sqlx::Error> {
    
    let user = sqlx::query_as::<_, User>("SELECT u.* FROM Sessions s NATURAL JOIN users u WHERE s.session_id = $1;")
        .bind(session)
        .fetch_one(pool)
        .await?;

    Ok(user)
}

/*
    Return un Session
    Return sqlx::Error::RowNotFound si la session n'existe pas
    Return sqlx::Error si c'est une erreur de base de données
*/
pub async fn get_session_from_name(pool: &Pool<Postgres>, name: &str) -> Result<Session, sqlx::Error> {
    let session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE name = $1;")
        .bind(name)
        .fetch_one(pool)
        .await?;
    Ok(session)

}

/*
    Return une Session
    Return sqlx::Error::RowNotFound si la session n'existe pas
    Return sqlx::Error si c'est une erreur de base de données
*/
pub async fn get_session_from_id(pool: &Pool<Postgres>, session_id: &str) -> Result<Session, sqlx::Error> {
    let session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(pool)
        .await?;

    Ok(session)
}
/*
    Return un Vec<User>
    Return sqlx::Error::RowNotFound si la table users est vide
    Return sqlx::Error si c'est une erreur de base de données
*/
pub async fn get_leaderboard(pool: &Pool<Postgres>) -> Result<Vec<User>, sqlx::Error> {
    let leaderboard = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY best_score DESC LIMIT 3")
        .fetch_all(pool)
        .await?;
    Ok(leaderboard)
}

/*
    Return sqlx::Error::RowNotFound si le user n'a pas été mis à jour
    Return sqlx::Error si c'est une erreur de base de données
    Return () si le user a été mis à jour
*/
pub async fn update_user(pool: &Pool<Postgres>, user: &User) -> Result<(), sqlx::Error> {
    let result = sqlx::query("UPDATE users SET best_score = $1, highest_level = $2, number_of_games = $3 WHERE name = $4")
        .bind(user.best_score)
        .bind(user.highest_level)
        .bind(user.number_of_games)
        .bind(&user.name)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

/**
 *  Return sqlx::Error::WorkerCrashed si c'est une erreur de sérialisation
 *  Return sqlx::Error::RowNotFound si la game n'a pas été créée ou mise à jour
 *  Return sqlx::Error si c'est une erreur de base de données
 *  Return () si le game a été créé ou mis à jour
 */
pub async fn upsert_game(pool: &Pool<Postgres>, game: &Game) -> Result<(), sqlx::Error> {
    let actions = serde_json::to_string(&game.game_actions)
        .map_err(|_| sqlx::Error::WorkerCrashed)?;

    let result = sqlx::query("INSERT INTO games (game_owner, game_score, game_level, game_lines, game_actions) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (game_owner) DO UPDATE SET game_score = EXCLUDED.game_score, game_level = EXCLUDED.game_level, game_lines = EXCLUDED.game_lines, game_actions = EXCLUDED.game_actions")
        .bind(&game.game_owner)
        .bind(game.game_score)
        .bind(game.game_level)
        .bind(game.game_lines)
        .bind(actions)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

/**
 *  Return une Game
 *  Return sqlx::Error::RowNotFound si la game n'existe pas
 *  Return sqlx::Error::WorkerCrashed si c'est une erreur de désérialisation
 *  Return sqlx::Error si c'est une erreur de base de données
 */
pub async fn get_game_from_owner(pool: &Pool<Postgres>, owner: &str) -> Result<Game, sqlx::Error> {

    let game_json = sqlx::query_as::<_, GameJson>("SELECT * FROM games WHERE game_owner = $1 ;")
        .bind(owner)
        .fetch_one(pool)
        .await?;
    let game_actions : Vec<Action> = serde_json::from_str(&game_json.game_actions).map_err(|_| sqlx::Error::WorkerCrashed)?;
    let game = Game {
        game_owner: game_json.game_owner,
        game_score: game_json.game_score,
        game_level: game_json.game_level,
        game_lines: game_json.game_lines,
        game_actions: game_actions,
    };
    Ok(game)
}

/**
 *  Return sqlx::Error::RowNotFound si la game n'a pas été supprimée
 *  Return sqlx::Error si c'est une erreur de base de données
 *  Return () si la game a été supprimée
 */
pub async fn delete_game(pool: &Pool<Postgres>, owner: &str) -> Result<(), sqlx::Error> {
    let result = sqlx::query("DELETE FROM games WHERE game_owner = $1")
        .bind(owner)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}


