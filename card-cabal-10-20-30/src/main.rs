use plotters::prelude::*;

use rand;
use rand::seq::SliceRandom;
use std::collections::VecDeque;

#[derive(Debug)]
struct CardStack {
    cards: VecDeque<i32>,
}

impl CardStack {
    pub fn empty() -> Self {
        CardStack {
            cards: VecDeque::new(),
        }
    }

    pub fn full_deck() -> Self {
        let mut cards = vec![];
        for _ in 0..4 {
            cards.extend(1..=10);
            cards.extend_from_slice(&[10, 10, 10]);
        }

        assert_eq!(cards.len(), 52);

        CardStack {
            cards: cards.into(),
        }
    }

    pub fn push_front(&mut self, card: i32) {
        self.cards.push_front(card);
    }

    pub fn push_back(&mut self, card: i32) {
        self.cards.push_back(card);
    }

    pub fn pop_front(&mut self) -> i32 {
        self.cards.pop_front().expect("pop_front: card deck empty")
    }

    pub fn pop_back(&mut self) -> i32 {
        self.cards.pop_back().expect("pop_back: card deck empty")
    }

    pub fn shuffle(self) -> CardStack {
        let mut v: Vec<i32> = self.cards.into();
        v.shuffle(&mut rand::thread_rng());
        return CardStack { cards: v.into() };
    }
}

fn turn(hand: &mut CardStack, board: &mut CardStack) -> bool {
    let len = board.cards.len();

    if len >= 3 {
        let mut sum: i32 = board.cards.iter().take(3).sum();

        if sum == 10 || sum == 20 || sum == 30 {
            // First 3 cards
            for _ in 1..=3 {
                hand.push_back(board.pop_front());
            }
            return true;
        }

        for (idx, (old, new)) in board
            .cards
            .iter()
            .take(3)
            .rev()
            .zip(board.cards.iter().rev().take(3))
            .enumerate()
        {
            sum = sum - *old + *new;
            if sum == 10 || sum == 20 || sum == 30 {
                let from_back = board.cards.iter().rev().take(idx + 1).rev();
                let from_back_len = from_back.len();
                for c in from_back {
                    hand.push_back(*c);
                }
                for _ in 1..=from_back_len {
                    board.pop_back();
                }
                for _ in 1..=3 - from_back_len {
                    hand.push_back(board.pop_front());
                }
                return true;
            }
        }
    }

    false
}

fn game() -> (bool, usize) {
    let mut rounds = 0;

    let mut hand = CardStack::full_deck().shuffle();
    assert_eq!(hand.cards.len(), 52);

    let mut board: Vec<_> = (1..=6).into_iter().map(|_| CardStack::empty()).collect();
    assert_eq!(6, board.len());

    // Setup
    for stack in &mut board {
        stack.push_front(hand.pop_front());
    }

    // Play
    for _ in 1..=10000 {
        for mut stack in &mut board {
            rounds += 1;

            if !stack.cards.is_empty() {
                stack.push_back(hand.pop_front());
                while turn(&mut hand, &mut stack) {}
            }
            if hand.cards.len() == 52 {
                return (true, rounds);
            }
            if hand.cards.len() == 0 {
                return (false, rounds);
            }
        }
    }

    (false, rounds)
}

fn main() {
    let games: u64 = 100000;
    let mut rounds: u64 = 0;
    let mut wins = 0;

    let mut win_max_rounds = 0;
    let mut win_min_rounds = 100000;

    let mut los_max_rounds = 0;
    let mut los_min_rounds = 100000;

    let mut dist = vec![0; 10000];

    for n in 1..=games {
        let (won, r) = game();
        wins += won as usize;

        if r != 60000 {
            rounds += r as u64;
            dist[r] += 1;
            if won {
                win_min_rounds = std::cmp::min(win_min_rounds, r);
                win_max_rounds = std::cmp::max(win_max_rounds, r);
            } else {
                los_min_rounds = std::cmp::min(los_min_rounds, r);
                los_max_rounds = std::cmp::max(los_max_rounds, r);
            }
        }
        println!(
            "{}: avg {} rounds, total {} wins",
            n,
            rounds / n,
            wins
        );
    }

    println!(
        "win rounds in {} .. {} .. {}",
        win_min_rounds,
        rounds / games,
        win_max_rounds
    );
    println!(
        "los rounds in {} .. {} .. {}",
        los_min_rounds,
        (rounds as f64) / (games as f64),
        los_max_rounds
    );

    let root = BitMapBackend::new("dist.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    dist = dist.into_iter().rev().skip_while(|x| *x == 0).collect();
    dist.reverse();

    let mut chart = ChartBuilder::on(&root)
        .caption("Game count by #rounds", ("sans-serif", 40).into_font())
        .x_label_area_size(20)
        .y_label_area_size(40)
        .build_cartesian_2d(0..dist.len() as i32, 0..*dist.iter().max().unwrap())
        .unwrap();
    chart.configure_mesh().draw().unwrap();

    chart.draw_series(LineSeries::new(dist.iter().enumerate().map(|(n, v)| (n as i32, *v)), &RED)).unwrap();
}

#[cfg(test)]
mod tests {
    use crate::{turn, CardStack};

    #[test]
    fn test_turn_result() {
        let cases = vec![
            (vec![5, 4, 1], vec![5, 4, 1]),
            (vec![10, 10, 10], vec![10, 10, 10]),
            (vec![10, 5, 5], vec![10, 5, 5]),
            (vec![6, 10, 10], vec![]),
            (vec![1, 2, 3, 5, 6, 7], vec![7, 1, 2]),
            (vec![1, 9, 3, 5, 2, 7], vec![2, 7, 1]),
            (vec![1, 9, 3, 10, 3, 7], vec![10, 3, 7]),
            (vec![10, 10, 1, 2, 10], vec![10, 10, 10]),
            (vec![1, 2, 3, 5, 5, 10], vec![5, 5, 10]),
        ];

        for (board_vec, expected_hand) in cases {
            let mut hand = CardStack::empty();
            let mut board = CardStack {
                cards: board_vec.into(),
            };

            assert_eq!(!expected_hand.is_empty(), turn(&mut hand, &mut board));
            assert_eq!(expected_hand, hand.cards.into_iter().collect::<Vec<i32>>());
        }
    }

    #[test]
    fn test_turn_random() {
        for i in 0..3600 {
            let mut board = CardStack::full_deck().shuffle();
            let mut hand = CardStack::empty();
            if turn(&mut hand, &mut board) {
                let sum: i32 = hand.cards.iter().sum();
                assert!(sum == 10 || sum == 20 || sum == 30);
            } else {
                assert_eq!(0, hand.cards.len());
            }
        }
    }
}
