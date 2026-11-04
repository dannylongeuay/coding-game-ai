// Fall Challenge 2026 (TryAngle Catch) bot.
// Single file: CodinGame takes one source file, std only.

use std::io::{self, BufRead, Write};
use std::time::Instant;

const ME: usize = 0;
const OP: usize = 1;
const NONE: i8 = -1;

const TURN_BUDGET_MS: u128 = 30;
const FIRST_TURN_BUDGET_MS: u128 = 600;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

#[derive(Clone)]
struct Tri {
    c: [usize; 3],
    owner: i8,
    can: [bool; 2],
}

#[derive(Clone)]
struct Game {
    n: usize,
    turn: usize,
    score: [i32; 2],
    units: [Vec<i32>; 2],
    adj: Vec<Vec<usize>>,
    tris: Vec<Tri>,
    dist: Vec<Vec<u8>>,
    /// Triangles next to each house, i.e. the ones that can attack it.
    near: Vec<Vec<usize>>,
}

struct Input<'a> {
    lines: std::io::Lines<std::io::StdinLock<'a>>,
}

impl Input<'_> {
    fn nums(&mut self) -> Vec<i32> {
        let line = self.lines.next().expect("eof").expect("read");
        line.split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect()
    }
}

impl Game {
    fn compute_dist(&mut self) {
        let n = self.n;
        self.dist = vec![vec![u8::MAX; n]; n];
        let mut queue = Vec::with_capacity(n);
        for s in 0..n {
            let d = &mut self.dist[s];
            d[s] = 0;
            queue.clear();
            queue.push(s);
            let mut head = 0;
            while head < queue.len() {
                let v = queue[head];
                head += 1;
                for &w in &self.adj[v] {
                    if d[w] == u8::MAX {
                        d[w] = d[v] + 1;
                        queue.push(w);
                    }
                }
            }
        }
    }

    fn compute_near(&mut self) {
        self.near = (0..self.n)
            .map(|v| {
                (0..self.tris.len())
                    .filter(|&i| self.tris[i].c.iter().any(|&c| self.adj[c].contains(&v)))
                    .collect()
            })
            .collect();
    }

    fn total(&self, p: usize) -> i32 {
        self.units[p].iter().sum()
    }
}

/// Result of resolving the end of a turn: units after surround kills, triangle owners and capture flags.
struct Outcome {
    units: [Vec<i32>; 2],
    node_owner: Vec<i8>,
    tri_owner: Vec<i8>,
}

fn node_owners(adj: &[Vec<usize>], units: &[Vec<i32>; 2]) -> Vec<i8> {
    let n = adj.len();
    let mut owner = vec![NONE; n];
    for v in 0..n {
        let nb = &adj[v];
        if !nb.is_empty() && nb.iter().all(|&w| units[0][w] > units[1][w]) {
            owner[v] = 0;
        } else if !nb.is_empty() && nb.iter().all(|&w| units[1][w] > units[0][w]) {
            owner[v] = 1;
        } else if units[0][v] > units[1][v] {
            owner[v] = 0;
        } else if units[1][v] > units[0][v] {
            owner[v] = 1;
        }
    }
    owner
}

fn update_can(tris: &[Tri], owner: &[i8], can: &mut [[bool; 2]], units: &[Vec<i32>; 2]) {
    for (i, t) in tris.iter().enumerate() {
        for p in 0..2 {
            if owner[i] != NONE {
                can[i][p] = owner[i] as usize != p;
            } else {
                can[i][p] |= t.c.iter().all(|&c| units[p][c] == 0);
            }
        }
    }
}

/// Mirrors Board.finalizeTurn: allowed captures, surround kills, then captures.
fn resolve(g: &Game, units: [Vec<i32>; 2], tri_owner: Vec<i8>, tri_can: Vec<[bool; 2]>) -> Outcome {
    let mut units = units;
    let mut tri_owner = tri_owner;
    let mut tri_can = tri_can;
    let n = g.n;

    update_can(&g.tris, &tri_owner, &mut tri_can, &units);

    let mut kill = [vec![false; n], vec![false; n]];
    for p in 0..2 {
        let o = 1 - p;
        for v in 0..n {
            let nb = &g.adj[v];
            if units[p][v] > 0 && !nb.is_empty() && nb.iter().all(|&w| units[o][w] > units[p][w]) {
                kill[p][v] = true;
            }
        }
    }
    let mut killed = false;
    for p in 0..2 {
        for v in 0..n {
            if kill[p][v] {
                units[p][v] = 0;
                killed = true;
            }
        }
    }
    if killed {
        update_can(&g.tris, &tri_owner, &mut tri_can, &units);
    }

    let node_owner = node_owners(&g.adj, &units);
    for (i, t) in g.tris.iter().enumerate() {
        for p in 0..2 {
            if tri_can[i][p] && t.c.iter().all(|&c| node_owner[c] == p as i8) {
                tri_owner[i] = p as i8;
            }
        }
    }
    update_can(&g.tris, &tri_owner, &mut tri_can, &units);

    Outcome {
        units,
        node_owner,
        tri_owner,
    }
}

struct Plan {
    /// Distance from each node to the nearest corner worth taking, measured before moving.
    target_dist: Vec<f64>,
    unit_value: f64,
}

/// Search budget: wall-clock on CodinGame, a fixed iteration count when CG_ITERS is set (reproducible local matches).
struct Budget {
    start: Instant,
    ms: u128,
    iters: Option<u64>,
    used: u64,
}

impl Budget {
    /// A sub-budget holding `percent` of what remains, for a planning phase that runs before the main one.
    fn share(&self, percent: u64) -> Budget {
        let left_ms = self.ms.saturating_sub(self.start.elapsed().as_millis());
        Budget {
            start: Instant::now(),
            ms: left_ms * percent as u128 / 100,
            iters: self
                .iters
                .map(|limit| limit.saturating_sub(self.used) * percent / 100),
            used: 0,
        }
    }

    fn spent(&mut self) -> bool {
        self.used += 1;
        match self.iters {
            Some(limit) => self.used > limit,
            None => self.start.elapsed().as_millis() >= self.ms,
        }
    }
}

/// Picks capturable triangles that free units can cover fastest and gives each chosen unit a corner to reach.
/// A corner needs one more unit than the opponent has there.
fn assign_targets(
    g: &Game,
    origin: &[usize],
    owner: &[i8],
    can: &[[bool; 2]],
) -> Vec<Option<usize>> {
    let mut assigned: Vec<Option<usize>> = vec![None; origin.len()];
    let mut chosen = vec![false; g.tris.len()];
    loop {
        let mut best: Option<(u32, usize, Vec<(usize, usize)>)> = None;
        for (i, t) in g.tris.iter().enumerate() {
            if chosen[i] || owner[i] == ME as i8 || !can[i][ME] {
                continue;
            }
            let mut taken: Vec<(usize, usize)> = Vec::new();
            let mut cost = 0u32;
            let mut ok = true;
            for &c in &t.c {
                for _ in 0..(g.units[OP][c] + 1) {
                    let pick = (0..origin.len())
                        .filter(|&u| assigned[u].is_none() && !taken.iter().any(|&(x, _)| x == u))
                        .min_by_key(|&u| g.dist[origin[u]][c]);
                    match pick {
                        Some(u) => {
                            cost = cost.max(g.dist[origin[u]][c] as u32);
                            taken.push((u, c));
                        }
                        None => ok = false,
                    }
                }
            }
            if ok && best.as_ref().map_or(true, |(bc, _, _)| cost < *bc) {
                best = Some((cost, i, taken));
            }
        }
        match best {
            Some((cost, i, taken)) if cost < 20 => {
                chosen[i] = true;
                for (u, c) in taken {
                    assigned[u] = Some(c);
                }
            }
            _ => break,
        }
    }
    assigned
}

fn evaluate(g: &Game, plan: &Plan, out: &Outcome) -> f64 {
    let n = g.n;
    let mut val = 0.0;
    let mut tri_count = [0i32; 2];
    for &o in &out.tri_owner {
        if o != NONE {
            tri_count[o as usize] += 1;
        }
    }
    val += 10.0 * (tri_count[ME] - tri_count[OP]) as f64;

    let total_me: i32 = out.units[ME].iter().sum();
    let total_op: i32 = out.units[OP].iter().sum();
    val += plan.unit_value * (total_me - total_op) as f64;

    for v in 0..n {
        match out.node_owner[v] {
            0 => val += weights().node,
            1 => val -= weights().node,
            _ => {}
        }
    }

    // Triangles we own whose every corner the opponent could take next turn.
    for (i, t) in g.tris.iter().enumerate() {
        if out.tri_owner[i] != ME as i8 {
            continue;
        }
        let threatened = t.c.iter().all(|&c| {
            if out.node_owner[c] == OP as i8 {
                return true;
            }
            let reach: i32 =
                out.units[OP][c] + g.adj[c].iter().map(|&w| out.units[OP][w]).sum::<i32>();
            reach > out.units[ME][c]
        });
        if threatened {
            val -= 4.0;
        }
    }

    // Best attack each side could make next turn: an owned triangle with a unit on every corner
    // wipes an adjacent stack for 3 units and the triangle.
    let w = weights();
    if w.expose > 0.0 {
        for p in [ME, OP] {
            let q = 1 - p;
            let mut best = 0.0f64;
            for v in 0..n {
                let k = out.units[q][v];
                if k <= 3 {
                    continue;
                }
                let gain = plan.unit_value * (k - 3) as f64 - w.expose_tri;
                if gain <= best {
                    continue;
                }
                let armed = g.near[v].iter().any(|&i| {
                    out.tri_owner[i] == p as i8 && g.tris[i].c.iter().all(|&c| out.units[p][c] > 0)
                });
                if armed {
                    best = gain;
                }
            }
            val += if p == ME { 1.0 } else { -1.0 } * w.expose * best;
        }
    }
    val
}

fn target_distances(g: &Game) -> Vec<f64> {
    let n = g.n;
    let owner = node_owners(&g.adj, &g.units);
    let mut targets = vec![false; n];
    for t in &g.tris {
        if t.owner != ME as i8 && t.can[ME] {
            for &c in &t.c {
                if owner[c] != ME as i8 {
                    targets[c] = true;
                }
            }
        }
    }
    (0..n)
        .map(|v| {
            (0..n)
                .filter(|&w| targets[w])
                .map(|w| g.dist[v][w] as f64)
                .fold(f64::INFINITY, f64::min)
        })
        .map(|d| if d.is_finite() { d } else { 0.0 })
        .collect()
}

fn tri_state(g: &Game) -> (Vec<i8>, Vec<[bool; 2]>) {
    (
        g.tris.iter().map(|t| t.owner).collect(),
        g.tris.iter().map(|t| t.can).collect(),
    )
}

/// The same position seen from the opponent's side, so our planner can predict its moves.
fn swapped(g: &Game, owner: &[i8], can: &[[bool; 2]]) -> (Game, Vec<i8>, Vec<[bool; 2]>) {
    let flip = |o: i8| if o == NONE { NONE } else { 1 - o };
    let mut sg = g.clone();
    sg.units.swap(0, 1);
    sg.score = [g.score[1], g.score[0]];
    for t in sg.tris.iter_mut() {
        t.owner = flip(t.owner);
        t.can = [t.can[1], t.can[0]];
    }
    let s_owner = owner.iter().map(|&o| flip(o)).collect();
    let s_can = can.iter().map(|c| [c[1], c[0]]).collect();
    (sg, s_owner, s_can)
}

/// Score of the position after our units reach `mine`, averaged over weighted opponent placements.
fn score_vs(
    g: &Game,
    plan: &Plan,
    owner: &[i8],
    can: &[[bool; 2]],
    mine: &[i32],
    scenarios: &[(f64, Vec<i32>)],
) -> f64 {
    scenarios
        .iter()
        .filter(|(w, _)| *w > 0.0)
        .map(|(w, op)| {
            let out = resolve(g, [mine.to_vec(), op.clone()], owner.to_vec(), can.to_vec());
            w * evaluate(g, plan, &out)
        })
        .sum()
}

/// Chooses a destination (stay or a neighbor) for each of our units. Returns (origin, destination) per unit and the plan's score.
fn plan_moves(
    g: &Game,
    plan: &Plan,
    owner: &[i8],
    can: &[[bool; 2]],
    scenarios: &[(f64, Vec<i32>)],
    rng: &mut Rng,
    budget: &mut Budget,
) -> (Vec<usize>, Vec<usize>, f64) {
    let mut origin: Vec<usize> = Vec::new();
    for v in 0..g.n {
        for _ in 0..g.units[ME][v] {
            origin.push(v);
        }
    }
    let mut dest = origin.clone();
    let assigned = assign_targets(g, &origin, owner, can);
    if std::env::var("CG_DEBUG").is_ok() {
        eprintln!(
            "origin {:?} assigned {:?} my_tris {} capturable {}",
            origin,
            assigned,
            owner.iter().filter(|&&o| o == ME as i8).count(),
            (0..g.tris.len())
                .filter(|&i| owner[i] != ME as i8 && can[i][ME])
                .count()
        );
    }
    let score_of = |dest: &[usize]| -> f64 {
        let mut mine = vec![0i32; g.n];
        let mut shaping = 0.0;
        for (u, &d) in dest.iter().enumerate() {
            mine[d] += 1;
            shaping += match assigned[u] {
                Some(c) => weights().assign * g.dist[d][c] as f64,
                None => 0.4 * plan.target_dist[d],
            };
        }
        score_vs(g, plan, owner, can, &mine, scenarios) - shaping
    };
    let mut best = score_of(&dest);
    if origin.is_empty() {
        return (origin, dest, best);
    }
    // Coordinate descent, then random pair moves for coordinated captures.
    let mut improved = true;
    while improved && !budget.spent() {
        improved = false;
        for u in 0..origin.len() {
            let keep = dest[u];
            let options: Vec<usize> = std::iter::once(origin[u])
                .chain(g.adj[origin[u]].iter().copied())
                .collect();
            for o in options {
                if o == keep {
                    continue;
                }
                dest[u] = o;
                let s = score_of(&dest);
                if s > best + 1e-9 {
                    best = s;
                    improved = true;
                    break;
                }
                dest[u] = keep;
            }
        }
    }
    while !budget.spent() {
        let a = rng.below(origin.len());
        let b = rng.below(origin.len());
        let (ka, kb) = (dest[a], dest[b]);
        let pick = |rng: &mut Rng, u: usize| {
            let k = rng.below(g.adj[origin[u]].len() + 1);
            if k == 0 {
                origin[u]
            } else {
                g.adj[origin[u]][k - 1]
            }
        };
        dest[a] = pick(rng, a);
        dest[b] = pick(rng, b);
        let s = score_of(&dest);
        if s >= best {
            best = s;
        } else {
            dest[a] = ka;
            dest[b] = kb;
        }
    }
    (origin, dest, best)
}

/// Opponent placements to plan against: its units where they stand, and where our planner predicts they go.
fn scenarios_for(
    g: &Game,
    pred_w: f64,
    prediction: &Option<Vec<usize>>,
) -> Vec<(f64, Vec<i32>)> {
    let mut scenarios = vec![(1.0 - pred_w, g.units[OP].clone())];
    if let Some(s_dest) = prediction {
        let mut predicted = vec![0i32; g.n];
        for &d in s_dest {
            predicted[d] += 1;
        }
        scenarios.push((pred_w, predicted));
    }
    scenarios
}

/// The position after we attack `target` from triangle `tri`: one unit spent per corner, the target's opponent units killed.
fn after_attack(
    g: &Game,
    owner: &[i8],
    can: &[[bool; 2]],
    tri: usize,
    target: usize,
) -> (Game, Vec<i8>, Vec<[bool; 2]>) {
    let mut ng = g.clone();
    for &c in &g.tris[tri].c {
        ng.units[ME][c] -= 1;
    }
    ng.units[OP][target] = 0;
    let mut o = owner.to_vec();
    let mut cn = can.to_vec();
    o[tri] = NONE;
    cn[tri][ME] = false;
    ng.tris[tri].owner = NONE;
    ng.tris[tri].can[ME] = false;
    (ng, o, cn)
}

fn play_turn(g: &mut Game, rng: &mut Rng, budget: &mut Budget) -> String {
    let mut cmds: Vec<String> = Vec::new();
    let (mut owner, mut can) = tri_state(g);
    let mut used = vec![false; g.tris.len()];

    // Attacks: kill large enemy stacks next to a triangle we own with a unit on every corner.
    for i in 0..g.tris.len() {
        let t = g.tris[i].clone();
        if owner[i] != ME as i8 || t.c.iter().any(|&c| g.units[ME][c] < 1) {
            continue;
        }
        let mut best: Option<(i32, usize)> = None;
        for &c in &t.c {
            for &w in &g.adj[c] {
                let k = g.units[OP][w];
                if k >= param("CG_ATTACK_MIN", 3) && best.map_or(true, |(bk, _)| k > bk) {
                    best = Some((k, w));
                }
            }
        }
        if let Some((_, target)) = best {
            let (ng, o, cn) = after_attack(g, &owner, &can, i, target);
            *g = ng;
            owner = o;
            can = cn;
            used[i] = true;
            cmds.push(format!("ATTACK {} {} {} {}", t.c[0], t.c[1], t.c[2], target));
        }
    }

    // Predict the opponent with our own planner from its side.
    let pred_w = param("CG_PRED_W", 100) as f64 / 100.0;
    let mut prediction = None;
    if pred_w > 0.0 {
        let (sg, s_owner, s_can) = swapped(g, &owner, &can);
        let s_plan = Plan {
            target_dist: target_distances(&sg),
            unit_value: 4.0,
        };
        let mut s_budget = budget.share(param("CG_PRED_SHARE", 33) as u64);
        let static_me = vec![(1.0, sg.units[OP].clone())];
        let (_, s_dest, _) = plan_moves(
            &sg,
            &s_plan,
            &s_owner,
            &s_can,
            &static_me,
            rng,
            &mut s_budget,
        );
        prediction = Some(s_dest);
    }

    // Moves: plan against where the opponent's units are now and where we expect them to go.
    let plan = Plan {
        target_dist: target_distances(g),
        unit_value: 4.0,
    };
    let scenarios = scenarios_for(g, pred_w, &prediction);
    let (origin, dest, _) = plan_moves(g, &plan, &owner, &can, &scenarios, rng, budget);

    let mut moves: Vec<((usize, usize), i32)> = Vec::new();
    for (u, &d) in dest.iter().enumerate() {
        if d == origin[u] {
            continue;
        }
        match moves.iter_mut().find(|(k, _)| *k == (origin[u], d)) {
            Some((_, cnt)) => *cnt += 1,
            None => moves.push(((origin[u], d), 1)),
        }
    }
    for ((from, to), cnt) in &moves {
        cmds.push(format!("MOVE {} {} {}", from, to, cnt));
    }
    let mut mine = vec![0i32; g.n];
    for &d in &dest {
        mine[d] += 1;
    }

    // Spawns: grow the army while it is small, picking the triangles and corners the evaluation likes best.
    let my_total = origin.len() as i32;
    let op_total = g.total(OP);
    let cap = (op_total + param("CG_SPAWN_LEAD", 2))
        .clamp(param("CG_SPAWN_MIN", 8), param("CG_SPAWN_MAX", 40));
    let mut want = if g.turn < param("CG_SPAWN_UNTIL", 160) as usize {
        (cap - my_total).max(0)
    } else {
        0
    };
    let spawn_plan = Plan {
        target_dist: plan.target_dist.clone(),
        unit_value: param("CG_SPAWN_VALUE", 12) as f64,
    };
    while want > 0 {
        let base = score_vs(g, &spawn_plan, &owner, &can, &mine, &scenarios);
        let mut best_spawn: Option<(f64, usize, usize)> = None;
        for i in 0..g.tris.len() {
            if used[i] || g.tris[i].owner != ME as i8 {
                continue;
            }
            for &c in &g.tris[i].c {
                let mut m = mine.clone();
                m[c] += 1;
                let mut o = owner.clone();
                let mut cn = can.clone();
                o[i] = NONE;
                cn[i][ME] = false;
                let s = score_vs(g, &spawn_plan, &o, &cn, &m, &scenarios)
                    - 0.4 * plan.target_dist[c]
                    - base;
                if best_spawn.map_or(true, |(bs, _, _)| s > bs) {
                    best_spawn = Some((s, i, c));
                }
            }
        }
        match best_spawn {
            Some((s, i, c)) if s > 0.0 => {
                let t = &g.tris[i];
                let others: Vec<usize> = t.c.iter().copied().filter(|&x| x != c).collect();
                cmds.push(format!("SPAWN {} {} {}", c, others[0], others[1]));
                mine[c] += 1;
                owner[i] = NONE;
                can[i][ME] = false;
                used[i] = true;
                want -= 1;
            }
            _ => break,
        }
    }

    if cmds.is_empty() {
        "WAIT".to_string()
    } else {
        cmds.join(";")
    }
}

fn main() {
    let stdin = io::stdin();
    let mut input = Input {
        lines: stdin.lock().lines(),
    };
    let n = input.nums()[0] as usize;
    for _ in 0..n {
        input.nums();
    }
    let mut g = Game {
        n,
        turn: 0,
        score: [0, 0],
        units: [vec![0; n], vec![0; n]],
        adj: vec![Vec::new(); n],
        tris: Vec::new(),
        dist: Vec::new(),
        near: Vec::new(),
    };
    let mut rng = Rng(0x9E3779B97F4A7C15);
    let iters: Option<u64> = std::env::var("CG_ITERS").ok().and_then(|s| s.parse().ok());
    let out = io::stdout();
    loop {
        let s = input.nums();
        let start = Instant::now();
        g.score = [s[0], s[1]];
        for _ in 0..n {
            let h = input.nums();
            let v = h[0] as usize;
            g.units[ME][v] = h[1];
            g.units[OP][v] = h[2];
        }
        for a in g.adj.iter_mut() {
            a.clear();
        }
        let paths = input.nums()[0];
        for _ in 0..paths {
            let p = input.nums();
            let (a, b) = (p[0] as usize, p[1] as usize);
            g.adj[a].push(b);
            g.adj[b].push(a);
        }
        let tc = input.nums()[0];
        g.tris.clear();
        for _ in 0..tc {
            let t = input.nums();
            g.tris.push(Tri {
                c: [t[0] as usize, t[1] as usize, t[2] as usize],
                owner: t[3] as i8,
                can: [t[4] != 0, t[5] != 0],
            });
        }
        let lc = input.nums()[0];
        for _ in 0..lc {
            input.nums();
        }
        g.compute_dist();
        g.compute_near();
        let ms = if g.turn == 0 {
            FIRST_TURN_BUDGET_MS
        } else {
            TURN_BUDGET_MS
        };
        let mut budget = Budget {
            start,
            ms,
            iters,
            used: 0,
        };
        let status = format!(
            "units {}-{} owned {}-{}",
            g.total(ME),
            g.total(OP),
            g.tris.iter().filter(|t| t.owner == ME as i8).count(),
            g.tris.iter().filter(|t| t.owner == OP as i8).count()
        );
        let cmd = play_turn(&mut g, &mut rng, &mut budget);
        eprintln!(
            "turn {} score {}-{} tris {} {} time {}ms",
            g.turn,
            g.score[0],
            g.score[1],
            g.tris.len(),
            status,
            start.elapsed().as_millis()
        );
        let mut lock = out.lock();
        writeln!(lock, "{}", cmd).unwrap();
        lock.flush().unwrap();
        g.turn += 1;
    }
}

/// Tuning knob read from the environment for local experiments; CodinGame always gets the default.
fn param(name: &str, default: i32) -> i32 {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// Evaluation weights used in hot loops, read once (tunable through CG_* in tenths).
struct Weights {
    node: f64,
    assign: f64,
    expose: f64,
    expose_tri: f64,
}

fn weights() -> &'static Weights {
    static W: std::sync::OnceLock<Weights> = std::sync::OnceLock::new();
    W.get_or_init(|| Weights {
        node: param("CG_NODE_W", 10) as f64 / 10.0,
        assign: param("CG_ASSIGN_W", 20) as f64 / 10.0,
        expose: param("CG_EXPOSE_W", 0) as f64 / 10.0,
        expose_tri: param("CG_EXPOSE_TRI", 50) as f64 / 10.0,
    })
}
