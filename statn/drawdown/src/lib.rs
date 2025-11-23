pub mod random;
pub mod drawdown;

pub use random::{set_seed, unifrand, normal};
pub use drawdown::{
    get_trades, mean_return, drawdown as calc_drawdown,
    drawdown_quantiles, find_quantile,
};
