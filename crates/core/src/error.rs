use thiserror::Error;

pub type Result<T> = std::result::Result<T, GameError>;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("游戏已满，无法加入")]
    GameFull,
    
    #[error("玩家不在游戏中")]
    PlayerNotFound,
    
    #[error("无效的操作")]
    InvalidAction,
    
    #[error("不是当前玩家的回合")]
    NotYourTurn,
    
    #[error("手牌中没有这张牌")]
    CardNotInHand,
    
    #[error("无效的牌型")]
    InvalidMeld,
    
    #[error("游戏尚未开始")]
    GameNotStarted,
    
    #[error("游戏已经结束")]
    GameAlreadyEnded,
    
    #[error("技能使用失败: {0}")]
    SkillError(String),
    
    #[error("网络错误: {0}")]
    NetworkError(String),
    
    #[error("内部错误: {0}")]
    InternalError(String),
}
