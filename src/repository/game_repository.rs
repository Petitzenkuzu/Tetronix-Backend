use sqlx::{Pool, Postgres};
use crate::errors::RepositoryError;
use crate::models::Game;
use crate::models::GameStats;
use crate::models::GameJson;

pub struct GameRepository {
    pub db: Pool<Postgres>,
}

impl GameRepository {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    /// Upsert a game
    /// 
    /// # Arguments
    /// 
    /// * `game` - The game to upsert
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the game has been upserted successfully
    /// * `Err(RepositoryError::InternalServerError)` - If there is a database error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let repo = GameRepository::new(db_pool);
    /// match repo.upsert_game(&game).await {
    ///     Ok(()) => println!("Game upserted successfully"),
    ///     Err(e) => eprintln!("Error upserting game: {}", e),
    /// }
    /// ```
    pub async fn upsert_game(&self, game: &Game) -> Result<(), RepositoryError> {
        let actions = serde_json::to_value(&game.game_actions).map_err(|_| RepositoryError::SerializationError("Failed to serialize game actions".into()))?;
        let result = sqlx::query("INSERT INTO games (game_owner, game_score, game_level, game_lines, game_actions) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (game_owner) DO UPDATE SET game_score = EXCLUDED.game_score, game_level = EXCLUDED.game_level, game_lines = EXCLUDED.game_lines, game_actions = EXCLUDED.game_actions")
            .bind(&game.game_owner)
            .bind(game.game_score)
            .bind(game.game_level)
            .bind(game.game_lines)
            .bind(actions)
            .execute(&self.db)
            .await;
    
        match result {
            Ok(result) => {
                if result.rows_affected() == 0 {
                    Err(RepositoryError::InternalServerError("Failed to upsert game".into()))
                } else {
                    Ok(())
                }   
            },
            Err(e) => Err(RepositoryError::InternalServerError(e.to_string())),
        }
    }

    /// Get a game by its owner
    /// 
    /// # Arguments
    /// 
    /// * `owner` - The name of the user to get the game for
    /// 
    /// # Returns
    /// 
    /// * `Ok(game)` - If the game has been found
    /// * `Err(RepositoryError::NotFound)` - If the game does not exist
    /// * `Err(RepositoryError::InternalServerError)` - If there is a database error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let repo = GameRepository::new(db_pool);
    /// match repo.get_game_by_owner("john_doe").await {
    ///     Ok(game) => println!("Game found: {:?}", game),
    ///     Err(e) => eprintln!("Error getting game: {}", e),
    /// }
    /// ```
    pub async fn get_game_by_owner(&self, owner: &str) -> Result<Game, RepositoryError> {

        let game = sqlx::query_as::<_, GameJson>("SELECT * FROM games WHERE game_owner = $1 LIMIT 1 ;")
            .bind(owner)
            .fetch_optional(&self.db)
            .await;

        match game {
            Ok(Some(game)) => {
                let game_actions: Vec<crate::models::Action> = serde_json::from_value(game.game_actions)
                    .map_err(|_| RepositoryError::DeserializationError("Failed to deserialize game actions".into()))?;

                let game = Game {
                    game_owner: game.game_owner,
                    game_score: game.game_score,
                    game_level: game.game_level,
                    game_lines: game.game_lines,
                    game_actions,
                };

                Ok(game)
            }
            Ok(None) => Err(RepositoryError::NotFound{what: "Game".into(), identifier: owner.into()}),
            Err(e) => Err(RepositoryError::InternalServerError(e.to_string())),
        }
    }

    /// Get the stats of a game by its owner
    /// 
    /// # Arguments
    /// 
    /// * `owner` - The name of the user to get the stats for
    /// 
    /// # Returns
    /// 
    /// * `Ok(stats)` - If the stats have been found
    /// * `Err(RepositoryError::NotFound)` - If the game does not exist
    /// * `Err(RepositoryError::InternalServerError)` - If there is a database error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let repo = GameRepository::new(db_pool);
    /// match repo.get_game_stats_by_owner("john_doe").await {
    ///     Ok(stats) => println!("Stats found: {:?}", stats),
    ///     Err(e) => eprintln!("Error getting stats: {}", e),
    /// }
    /// ```
    pub async fn get_game_stats_by_owner(&self, owner: &str) -> Result<GameStats, RepositoryError> {

        let game = sqlx::query_as::<_, GameStats>("SELECT game_score, game_level, game_lines FROM games WHERE game_owner = $1 LIMIT 1 ;")
            .bind(owner)
            .fetch_optional(&self.db)
            .await;
    
        match game {
            Ok(Some(game)) => Ok(game),
            Ok(None) => Err(RepositoryError::NotFound{what: "Game".into(), identifier: owner.into()}),
            Err(e) => Err(RepositoryError::InternalServerError(e.to_string())),
        }
    }

    /// Delete a game
    /// 
    /// # Arguments
    /// 
    /// * `owner` - The name of the user to delete the game for
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the game has been deleted successfully
    /// * `Err(RepositoryError::NotFound)` - If the game does not exist
    /// * `Err(RepositoryError::InternalServerError)` - If there is a database error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let repo = GameRepository::new(db_pool);
    /// match repo.delete_game("john_doe").await {
    ///     Ok(()) => println!("Game deleted successfully"),
    ///     Err(e) => eprintln!("Error deleting game: {}", e),
    /// }
    /// ```
    pub async fn delete_game(&self, owner: &str) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM games WHERE game_owner = $1")
            .bind(owner)
            .execute(&self.db)
            .await;
    
        match result {
            Ok(result) => {
                if result.rows_affected() == 0 {
                    Err(RepositoryError::NotFound{what: "Game".into(), identifier: owner.into()})
                } else {
                    Ok(())
                }
            },
            Err(e) => Err(RepositoryError::InternalServerError(e.to_string())),
        }
    }

}