//! Pure scoring / lives logic and tests.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreResult {
    pub score: u32,
    pub lives: u32,
    pub high_score: u32,
    pub game_over: bool,
}

/// Process one wave result.
///
/// `covered` - true if the dam covered every river column.
pub fn evaluate_wave(
    score: u32,
    lives: u32,
    high_score: u32,
    covered: bool,
    points: u32,
) -> ScoreResult {
    if covered {
        let new_score = score.saturating_add(points);
        let new_high = high_score.max(new_score);
        ScoreResult {
            score: new_score,
            lives,
            high_score: new_high,
            game_over: false,
        }
    } else {
        let new_lives = lives.saturating_sub(1);
        ScoreResult {
            score,
            lives: new_lives,
            high_score,
            game_over: new_lives == 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covered_wave_adds_points_and_updates_high_score() {
        let r = evaluate_wave(10, 3, 10, true, 5);
        assert_eq!(
            r,
            ScoreResult {
                score: 15,
                lives: 3,
                high_score: 15,
                game_over: false,
            }
        );
    }

    #[test]
    fn uncovered_wave_removes_life() {
        let r = evaluate_wave(10, 3, 20, false, 5);
        assert_eq!(
            r,
            ScoreResult {
                score: 10,
                lives: 2,
                high_score: 20,
                game_over: false,
            }
        );
    }

    #[test]
    fn last_leak_is_game_over() {
        let r = evaluate_wave(10, 1, 20, false, 5);
        assert_eq!(
            r,
            ScoreResult {
                score: 10,
                lives: 0,
                high_score: 20,
                game_over: true,
            }
        );
    }

    #[test]
    fn high_score_does_not_decrease() {
        let r = evaluate_wave(5, 3, 20, true, 5);
        assert_eq!(r.high_score, 20);
    }
}
