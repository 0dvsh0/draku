//! "Let go" — the judgment layer. Every file gets a row of plain signals:
//! age, category, date passed, duplicate distance, blur, size. The judge
//! turns them into a dead score in [0, 1].

pub struct Signals {
    pub age_days: f32,
    pub is_screenshot: bool,
    pub is_otp: bool,
    pub is_ticket: bool,
    pub date_passed: bool,
    pub is_dup: bool,
    pub size: u64,
}

pub struct Verdict {
    pub dead: f32,
    pub reason: Option<String>,
}

/// Weights tuned so the product's story holds exactly:
/// a fresh untouched file stays alive, but an expired deadline, a duplicate
/// or a one-time-code screenshot is already "ready to let go" today.
pub fn judge(s: &Signals) -> Verdict {
    let mut score = 0.10;
    score += (s.age_days / 180.0).min(0.30);
    if s.is_screenshot {
        score += 0.08;
    }
    if s.is_otp {
        score += 0.44;
    }
    if s.is_ticket {
        score += 0.16;
    }
    if s.date_passed {
        score += 0.50;
    }
    if s.is_dup {
        score += 0.50;
    }
    if (s.size as f32) < 60_000.0 {
        score += 0.02;
    }

    let mut reason = None;
    if s.date_passed {
        reason = Some("Its date has passed".into());
    } else if s.is_dup {
        reason = Some("A near-identical file exists".into());
    } else if s.is_otp {
        reason = Some("One-time codes only live once".into());
    } else if s.age_days > 365.0 {
        reason = Some("Untouched for a year".into());
    }

    Verdict {
        dead: score.clamp(0.0, 1.0),
        reason,
    }
}

/// Threshold above which a file shows up on the "ready to let go" card.
pub const DEAD_CUTOFF: f32 = 0.60;

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(over: impl Fn(&mut Signals)) -> Verdict {
        let mut s = Signals {
            age_days: 0.0,
            is_screenshot: false,
            is_otp: false,
            is_ticket: false,
            date_passed: false,
            is_dup: false,
            size: 500_000,
        };
        over(&mut s);
        judge(&s)
    }

    #[test]
    fn fresh_small_file_is_alive() {
        assert!(sig(|_| ()).dead < DEAD_CUTOFF);
    }

    #[test]
    fn an_otp_screenshot_is_dead_the_day_it_is_taken() {
        let v = sig(|s| {
            s.is_otp = true;
            s.is_screenshot = true;
        });
        assert!(v.dead >= DEAD_CUTOFF, "got {}", v.dead);
    }

    #[test]
    fn a_passed_date_marks_death_with_a_reason() {
        let v = sig(|s| s.date_passed = true);
        assert!(v.dead >= DEAD_CUTOFF);
        assert_eq!(v.reason.as_deref(), Some("Its date has passed"));
    }

    #[test]
    fn duplicate_beats_age() {
        let dup = sig(|s| {
            s.is_dup = true;
        });
        let old = sig(|s| {
            s.age_days = 400.0;
        });
        assert!(dup.dead >= DEAD_CUTOFF);
        assert!(dup.dead > old.dead, "dup {} age {}", dup.dead, old.dead);
    }

    #[test]
    fn never_exceeds_one() {
        let v = sig(|s| {
            s.is_otp = true;
            s.is_dup = true;
            s.date_passed = true;
            s.age_days = 900.0;
            s.size = 0;
        });
        assert!(v.dead <= 1.0);
    }
}
