use crate::models::GameStats;
use crate::errors::ServicesError;
use crate::models::Game;
use crate::repository::GameRepositoryTrait;
use crate::services::traits::GameServiceTrait;
#[derive(Clone)]
pub struct GameService<T: GameRepositoryTrait> {
    game_repository: T,
}

impl<T: GameRepositoryTrait> GameService<T> {
    pub fn new(game_repository: T) -> Self {
        Self { game_repository }
    }
}

impl<T: GameRepositoryTrait> GameServiceTrait for GameService<T> {
    /// Get the stats of a game by its owner
    /// 
    /// # Arguments
    /// 
    /// * `game_owner` - The name of the user to get the stats for
    /// 
    /// # Returns
    /// 
    /// * `Ok(stats)` - If the stats have been found
    /// * `Err(ServicesError::NotFound{what})` - If the game does not exist
    /// * `Err(ServicesError::InternalServerError(e))` - If there is an internal server error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let game_service = GameService::new(GameRepository::new(db_pool));
    /// match game_service.get_stats("john_doe").await {
    ///     Ok(stats) => println!("Stats found: {:?}", stats),
    ///     Err(e) => eprintln!("Error getting stats: {}", e),
    /// }
    /// ```
    async fn get_stats(&self, game_owner: &str) -> Result<GameStats, ServicesError> {
        let stats = self.game_repository.get_game_stats_by_owner(game_owner).await?;
        Ok(stats)
    }

    /// Get the game by its owner
    /// 
    /// # Arguments
    /// 
    /// * `game_owner` - The name of the user to get the game for
    /// 
    /// # Returns
    /// 
    /// * `Ok(game)` - If the game has been found
    /// * `Err(ServicesError::NotFound{what})` - If the game does not exist
    /// * `Err(ServicesError::DeserializationError)` - If the game actions cannot be deserialized
    /// * `Err(ServicesError::InternalServerError(e))` - If there is an internal server error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let game_service = GameService::new(GameRepository::new(db_pool));
    /// match game_service.get_by_owner("john_doe").await {
    ///     Ok(game) => println!("Game found: {:?}", game),
    ///     Err(e) => eprintln!("Error getting game: {}", e),
    /// }
    /// ```
    async fn get_by_owner(&self, game_owner: &str) -> Result<Game, ServicesError> {
        let game = self.game_repository.get_game_by_owner(game_owner).await?;
        Ok(game)
    }

    /// Upsert a game
    /// 
    /// # Arguments
    /// 
    /// * `game` - The game to upsert
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the game has been upserted
    /// * `Err(ServicesError::InternalServerError(e))` - If there is an internal server error
    /// 
    /// # Examples
    /// 
    /// ```
    /// let game_service = GameService::new(GameRepository::new(db_pool));
    /// let game = Game::new("john_doe".to_string(), vec![]);
    /// match game_service.upsert(&game).await {
    ///     Ok(()) => println!("Game upserted"),
    ///     Err(e) => eprintln!("Error upserting game: {}", e),
    /// }
    /// ```
    async fn upsert(&self, game: &Game) -> Result<(), ServicesError> {
        let res = self.game_repository.upsert_game(game).await?;
        Ok(res)
    }
}