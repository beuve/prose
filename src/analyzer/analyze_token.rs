use crate::{
    analyzer::token_stats::TokenStats, engine::tokens::Token, parser::yaml_parser::LogingConfig,
};
use ndarray::s;

pub fn analyze_single_token(
    token: &Token,
    loging_config: &LogingConfig,
    max_time: usize,
) -> TokenStats {
    let process_count = loging_config.actors_indices.len();
    let mut stats = TokenStats::zeros(process_count, max_time);
    let mut timeline = token.timeline.iter();
    let (time, code) = timeline.next().expect("msg");
    let mut time = *time as usize;
    let mut code = code;
    for (next_time, next_code) in timeline {
        let actor_idx = loging_config.actors_indices.get_by_left(code).expect("msg");
        stats.reentrances[[*actor_idx, time]] += 1;

        let end_time = (*next_time as usize).min(max_time - 1);

        stats.lifetimes[*actor_idx] += (end_time - time) as f64;

        let mut occupancy_slice = stats.occupancies.slice_mut(s![*actor_idx, time..end_time]);
        occupancy_slice += 1;
        time = *next_time as usize;
        code = next_code;
    }

    stats.lifetimes_sq = stats.lifetimes.map(|x| x.powi(2));
    stats
}
