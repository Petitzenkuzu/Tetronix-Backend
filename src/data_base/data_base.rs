use sqlx::{Pool,Postgres} ;

use crate::models::{User,Session, Game, GameJson, GameStats};
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


/**
 *  Return sqlx::Error::WorkerCrashed si c'est une erreur de sérialisation
 *  Return sqlx::Error::RowNotFound si la game n'a pas été créée ou mise à jour
 *  Return sqlx::Error si c'est une erreur de base de données
 *  Return () si le game a été créé ou mis à jour
 */
pub async fn upsert_game(pool: &Pool<Postgres>, game: &Game) -> Result<(), sqlx::Error> {
    let actions = serde_json::to_value(&game.game_actions).map_err(|_| sqlx::Error::WorkerCrashed)?;
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

    let game = sqlx::query_as::<_, GameJson>("SELECT * FROM games WHERE game_owner = $1 LIMIT 1 ;")
        .bind(owner)
        .fetch_one(pool)
        .await?;

    let game_actions: Vec<crate::models::Action> = serde_json::from_value(game.game_actions)
        .map_err(|_| sqlx::Error::WorkerCrashed)?;

    let game = Game {
        game_owner: game.game_owner,
        game_score: game.game_score,
        game_level: game.game_level,
        game_lines: game.game_lines,
        game_actions,
    };

    Ok(game)
}

/**
 *  Return un GameStats
 *  Return sqlx::Error::RowNotFound si la game n'existe pas
 *  Return sqlx::Error si c'est une erreur de base de données
 */
pub async fn get_game_stats_from_owner(pool: &Pool<Postgres>, owner: &str) -> Result<GameStats, sqlx::Error> {

    let game = sqlx::query_as::<_, GameStats>("SELECT game_score, game_level, game_lines FROM games WHERE game_owner = $1 LIMIT 1 ;")
        .bind(owner)
        .fetch_one(pool)
        .await?;

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


#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;
    use uuid::Uuid;
    use sqlx::{Pool, Postgres,postgres::PgPoolOptions};
    use std::sync::OnceLock;
    use crate::models::Action;

    // pool commune à tous les tests partagés entre les threads
    static POOL: OnceLock<Pool<Postgres>> = OnceLock::new();

    // fonction pour get ou init la pool
    async fn get_pool() -> &'static Pool<Postgres> {
        if POOL.get().is_none() {
            dotenv().ok();
            let database_url = env::var("TEST_DB_URL").expect("TEST_DB_URL must be set");
            let pool = PgPoolOptions::new()
                   .max_connections(1)
                   .connect(&database_url)
                   .await
                   .expect("Failed to connect to DB");
            return POOL.get_or_init(|| pool);
        }
        POOL.get().unwrap()
    }

    // fonction pour générer un string aléatoire
    fn get_random_string() -> String {
        Uuid::new_v4().to_string()
    }

    #[tokio::test]
    async fn test_create_get_delete_user() {
        //Test de la création, de la récupération et de la suppression d'un user
        let pool = get_pool().await;
        let name = get_random_string();
        
        // test d'un user valide
        assert!(create_user(pool, &name).await.is_ok());
        let user = get_user_from_name(pool, &name).await.unwrap();
        assert_eq!(user.name, name);
        assert_eq!(user.best_score, 0);
        assert_eq!(user.highest_level, 0);
        assert_eq!(user.number_of_games, 0);
        assert!(delete_user(pool, &name).await.is_ok());
        assert!(delete_user(pool, &name).await.is_err());

        // test d'un user invalide
        let un_authorized_name = "a";
        assert!(create_user(pool, un_authorized_name).await.is_err());
        assert!(get_user_from_name(pool, un_authorized_name).await.is_err());
        assert!(delete_user(pool, un_authorized_name).await.is_err());
    }

    #[tokio::test]
    async fn test_create_get_delete_session() {
        //Test de la création, de la récupération et de la suppression d'une session

        let pool = get_pool().await;
        let name = get_random_string();
        let session_id = get_random_string();

        // test d'une session valide
        assert!(create_user(pool, &name).await.is_ok());
        assert!(create_session(pool, &name, &session_id).await.is_ok());

        // test insertion d'un session_id déjà existant
        assert!(create_session(pool, &name, &session_id).await.is_err());

        let session = get_session_from_name(pool, &name).await.unwrap();

        assert_eq!(session.name, name);
        assert_eq!(session.session_id, session_id);

        let session = get_session_from_id(pool, &session_id).await.unwrap();

        assert_eq!(session.name, name);
        assert_eq!(session.session_id, session_id);

        assert!(delete_session(pool, &session_id).await.is_ok());
        assert!(delete_session(pool, &session_id).await.is_err());
        assert!(delete_user(pool, &name).await.is_ok());
        assert!(delete_user(pool, &name).await.is_err());

        // test des get des sessions avec un nom et un invalide
        assert!(get_session_from_name(pool, &name).await.is_err());
        assert!(get_session_from_id(pool, &session_id).await.is_err());
    }

    #[tokio::test]
    async fn test_get_user_from_session() {
        //Test de la récupération d'un user à partir d'une session
        let pool = get_pool().await;
        let name = get_random_string();
        let session_id = get_random_string();

        create_user(pool, &name).await.unwrap();
        create_session(pool, &name, &session_id).await.unwrap();

        let user = get_user_from_session(pool, &session_id).await.unwrap();

        assert_eq!(user.name, name);
        assert_eq!(user.best_score, 0);
        assert_eq!(user.highest_level, 0);
        assert_eq!(user.number_of_games, 0);

        // delete cascade alors session supprimée en même temps que le user
        assert!(delete_user(pool, &name).await.is_ok());
        assert!(delete_user(pool, &name).await.is_err());
        assert!(delete_session(pool, &session_id).await.is_err());

        // test d'une session invalide
        assert!(get_user_from_session(pool, &session_id).await.is_err());
    }

    #[tokio::test]
    async fn test_get_session_from_name() {
        //Test de la récupération d'une session à partir d'un nom
        let pool = get_pool().await;
        let name = get_random_string();
        let session_id = get_random_string();

        create_user(pool, &name).await.unwrap();
        create_session(pool, &name, &session_id).await.unwrap();

        let session = get_session_from_name(pool, &name).await.unwrap();

        assert_eq!(session.name, name);
        assert_eq!(session.session_id, session_id);

        // delete cascade alors session supprimée en même temps que le user
        assert!(delete_user(pool, &name).await.is_ok());
        assert!(delete_session(pool, &session_id).await.is_err());
    }
    #[tokio::test]
    async fn test_delete_user() {
        //Test de la suppression d'un user
        let pool = get_pool().await;
        let name = get_random_string();

        create_user(pool, &name).await.unwrap();

        assert!(delete_user(pool, &name).await.is_ok());
        assert!(delete_user(pool, &name).await.is_err());
    }
    #[tokio::test]
    async fn test_delete_session() {
        //Test de la suppression d'une session
        let pool = get_pool().await;
        let name = get_random_string();
        let session_id = get_random_string();

        create_user(pool, &name).await.unwrap();
        create_session(pool, &name, &session_id).await.unwrap();

        // delete cascade alors user supprimé en même temps que la session
        assert!(delete_user(pool, &name).await.is_ok());
        assert!(delete_user(pool, &name).await.is_err());
        assert!(delete_session(pool, &session_id).await.is_err());
    }

    #[tokio::test]
    async fn test_update_user() {
        //Test de la mise à jour d'un user
        let pool = get_pool().await;
        let name = get_random_string();
        create_user(pool, &name).await.unwrap();
        let user = get_user_from_name(pool, &name).await.unwrap();

        assert_eq!(user.best_score, 0);
        assert_eq!(user.highest_level, 0);
        assert_eq!(user.number_of_games, 0);

        let user = User {
            name: name.clone(),
            best_score: 100,
            highest_level: 1,
            number_of_games: 1,
        };
        update_user(pool, &user).await.unwrap();

        let user = get_user_from_name(pool, &name).await.unwrap();

        assert_eq!(user.best_score, 100);
        assert_eq!(user.highest_level, 1);
        assert_eq!(user.number_of_games, 1);

        assert!(delete_user(pool, &name).await.is_ok());
        assert!(delete_user(pool, &name).await.is_err());
    }

    #[tokio::test]
    async fn test_get_leaderboard() {
        //Test de la récupération du leaderboard
        let pool = get_pool().await;

        // mettre ce test uniquement si on lance les tests avec cargo test -- --test-threads=1
        //assert_eq!(get_leaderboard(pool).await.unwrap().len(), 0);

        let name = get_random_string();
        let name2 = get_random_string();
        let name3 = get_random_string();
        let name4 = get_random_string();
        let name5 = get_random_string();

        create_user(pool, &name).await.unwrap();
        create_user(pool, &name2).await.unwrap();
        create_user(pool, &name3).await.unwrap();
        create_user(pool, &name4).await.unwrap();
        create_user(pool, &name5).await.unwrap();

        let user = User {
            name: name.clone(),
            best_score: 150,
            highest_level: 1,
            number_of_games: 1,
        };
        update_user(pool, &user).await.unwrap();
        let user2 = User {
            name: name2.clone(),
            best_score: 120,
            highest_level: 1,
            number_of_games: 1,
        };
        update_user(pool, &user2).await.unwrap();
        let user3 = User {
            name: name3.clone(),
            best_score: 100,
            highest_level: 1,
            number_of_games: 1,
        };
        update_user(pool, &user3).await.unwrap();
        let leaderboard = get_leaderboard(pool).await.unwrap();

        // les users sont correctement triés par score décroissant
        assert_eq!(leaderboard.len(), 3);
        assert_eq!(leaderboard[0].name, name);
        assert_eq!(leaderboard[1].name, name2);
        assert_eq!(leaderboard[2].name, name3);

        assert!(delete_user(pool, &name).await.is_ok());
        assert!(delete_user(pool, &name2).await.is_ok());
        assert!(delete_user(pool, &name3).await.is_ok());
        assert!(delete_user(pool, &name4).await.is_ok());
        assert!(delete_user(pool, &name5).await.is_ok());
    }

    #[tokio::test]
    async fn test_upsert_game() {
        //Test de l'insertion et de la mise à jour d'une game
        let pool = get_pool().await;
        let owner = get_random_string();
        create_user(pool, &owner).await.unwrap();

        let game = Game {
            game_owner: owner.clone(),
            game_score: 100,
            game_level: 1,
            game_lines: 1,
            game_actions: vec![],
        };

        upsert_game(pool, &game).await.unwrap();
        let game = get_game_from_owner(pool, &owner).await.unwrap();

        // la game est correctement insérée
        assert_eq!(game.game_owner, owner);
        assert_eq!(game.game_score, 100);
        assert_eq!(game.game_level, 1);
        assert_eq!(game.game_lines, 1);
        assert_eq!(game.game_actions.len(), 0);

        let game = Game {
            game_owner: owner.clone(),
            game_score: 200,
            game_level: 2,
            game_lines: 2,
            game_actions: vec![Action{action_type: "move".to_string(), piece: None, timestamp: 100}],
        };

        upsert_game(pool, &game).await.unwrap();
        let game = get_game_from_owner(pool, &owner).await.unwrap();

        // la game est correctement mise à jour
        assert_eq!(game.game_owner, owner);
        assert_eq!(game.game_score, 200);
        assert_eq!(game.game_level, 2);
        assert_eq!(game.game_lines, 2);
        assert_eq!(game.game_actions.len(), 1);

        assert!(delete_game(pool, &owner).await.is_ok());
        assert!(delete_game(pool, &owner).await.is_err());
        assert!(delete_user(pool, &owner).await.is_ok());
        assert!(delete_user(pool, &owner).await.is_err());
    }
    #[tokio::test]
    async fn test_get_game_stats_from_owner() {
        //Test de la récupération des stats d'une game
        let pool = get_pool().await;
        let owner = get_random_string();
        create_user(pool, &owner).await.unwrap();

        // la game n'existe pas
        assert!(get_game_stats_from_owner(pool, &owner).await.is_err());

        let game = Game {
            game_owner: owner.clone(),
            game_score: 100,
            game_level: 1,
            game_lines: 1,
            game_actions: vec![],
        };

        upsert_game(pool, &game).await.unwrap();
        let game_stats = get_game_stats_from_owner(pool, &owner).await.unwrap();

        // les stats sont correctement récupérées
        assert_eq!(game_stats.game_score, 100);
        assert_eq!(game_stats.game_level, 1);
        assert_eq!(game_stats.game_lines, 1);

        assert!(delete_game(pool, &owner).await.is_ok());
        assert!(delete_game(pool, &owner).await.is_err());
        assert!(delete_user(pool, &owner).await.is_ok());
        assert!(delete_user(pool, &owner).await.is_err());
    }

    #[tokio::test]
    async fn test_get_game_from_owner() {
        //Test de la récupération d'une game à partir d'un owner
        let pool = get_pool().await;
        let owner = get_random_string();
        create_user(pool, &owner).await.unwrap();

        // la game n'existe pas
        assert!(get_game_from_owner(pool, &owner).await.is_err());

        let game = Game {
            game_owner: owner.clone(),
            game_score: 100,
            game_level: 1,
            game_lines: 1,
            game_actions: vec![Action{action_type: "move".to_string(), piece: None, timestamp: 100}],
        };

        upsert_game(pool, &game).await.unwrap();
        let game = get_game_from_owner(pool, &owner).await.unwrap();

        // la game est correctement récupérée
        assert_eq!(game.game_owner, owner);
        assert_eq!(game.game_score, 100);
        assert_eq!(game.game_level, 1);
        assert_eq!(game.game_lines, 1);
        assert_eq!(game.game_actions.len(), 1);
        assert_eq!(game.game_actions[0].action_type, "move");
        assert!(game.game_actions[0].piece.is_none());
        assert_eq!(game.game_actions[0].timestamp, 100);

        assert!(delete_game(pool, &owner).await.is_ok());
        assert!(delete_game(pool, &owner).await.is_err());
        assert!(delete_user(pool, &owner).await.is_ok());
        assert!(delete_user(pool, &owner).await.is_err());
    }
}
