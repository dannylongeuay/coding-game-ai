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

struct Game {
    n: usize,
    turn: usize,
    score: [i32; 2],
    units: [Vec<i32>; 2],
    adj: Vec<Vec<usize>>,
    tris: Vec<Tri>,
    dist: Vec<Vec<u8>>,
}

struct Input<'a> {
    lines: std::io::Lines<std::io::StdinLock<'a>>,
}

impl Input<'_> {
    fn nums(&mut self) -> Vec<i32> {
        let line = self.lines.next().expect("eof").expect("read");
        line.split_whitespace().map(|t| t.parse().unwrap()).collect()
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

    Outcome { units, node_owner, tri_owner }
}

struct Plan {
    /// Distance from each node to the nearest corner worth taking, measured before moving.
    target_dist: Vec<f64>,
    unit_value: f64,
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
            0 => val += 1.0,
            1 => val -= 1.0,
            _ => {}
        }
        if out.units[ME][v] > 0 {
            val -= 0.4 * plan.target_dist[v] * out.units[ME][v] as f64;
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
            let reach: i32 = out.units[OP][c] + g.adj[c].iter().map(|&w| out.units[OP][w]).sum::<i32>();
            reach > out.units[ME][c]
        });
        if threatened {
            val -= 4.0;
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
    (g.tris.iter().map(|t| t.owner).collect(), g.tris.iter().map(|t| t.can).collect())
}

fn play_turn(g: &mut Game, rng: &mut Rng, start: Instant, budget: u128) -> String {
    let mut cmds: Vec<String> = Vec::new();
    let (mut owner, mut can) = tri_state(g);

    // Attacks: kill large enemy stacks next to a triangle we own with a unit on every corner.
    let mut used = vec![false; g.tris.len()];
    for i in 0..g.tris.len() {
        let t = g.tris[i].clone();
        if owner[i] != ME as i8 || !t.c.iter().all(|&c| g.units[ME][c] >= 1) {
            continue;
        }
        let mut best: Option<(i32, usize)> = None;
        for &c in &t.c {
            for &w in g.adj[c].iter() {
                let k = g.units[OP][w];
                if k >= 3 && best.map_or(true, |(bk, _)| k > bk) {
                    best = Some((k, w));
                }
            }
        }
        if let Some((_, target)) = best {
            for &c in &t.c {
                g.units[ME][c] -= 1;
            }
            g.units[OP][target] = 0;
            owner[i] = NONE;
            can[i][ME] = false;
            used[i] = true;
            cmds.push(format!("ATTACK {} {} {} {}", t.c[0], t.c[1], t.c[2], target));
        }
    }

    // Moves: one entry per unit, each choosing to stay or step to a neighbor.
    let plan = Plan { target_dist: target_distances(g), unit_value: 4.0 };
    let mut origin: Vec<usize> = Vec::new();
    for v in 0..g.n {
        for _ in 0..g.units[ME][v] {
            origin.push(v);
        }
    }
    let mut dest = origin.clone();
    let score_of = |dest: &[usize]| -> f64 {
        let mut mine = vec![0i32; g.n];
        for &d in dest {
            mine[d] += 1;
        }
        let out = resolve(g, [mine, g.units[OP].clone()], owner.clone(), can.clone());
        evaluate(g, &plan, &out)
    };
    let mut best = score_of(&dest);
    if !origin.is_empty() {
        // Coordinate descent, then random pair moves for coordinated captures.
        let mut improved = true;
        while improved && start.elapsed().as_millis() < budget {
            improved = false;
            for u in 0..origin.len() {
                let keep = dest[u];
                let options: Vec<usize> = std::iter::once(origin[u]).chain(g.adj[origin[u]].iter().copied()).collect();
                for o in options {
                    if o == keep {
                        continue;
                    }
                    dest[u] = o;
                    let s = score_of(&dest);
                    if s > best + 1e-9 {
                        best = s;
                        improved = true;
                    } else {
                        dest[u] = keep;
                    }
                    if dest[u] != keep {
                        break;
                    }
                }
            }
        }
        while start.elapsed().as_millis() < budget {
            let a = rng.below(origin.len());
            let b = rng.below(origin.len());
            let (ka, kb) = (dest[a], dest[b]);
            let pick = |rng: &mut Rng, u: usize| {
                let k = rng.below(g.adj[origin[u]].len() + 1);
                if k == 0 { origin[u] } else { g.adj[origin[u]][k - 1] }
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
    }

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
    let cap = (op_total + 2).clamp(8, 40);
    let mut want = if g.turn < 160 { (cap - my_total).max(0) } else { 0 };
    let spawn_plan = Plan { target_dist: plan.target_dist.clone(), unit_value: 12.0 };
    while want > 0 {
        let base = {
            let out = resolve(g, [mine.clone(), g.units[OP].clone()], owner.clone(), can.clone());
            evaluate(g, &spawn_plan, &out)
        };
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
                let out = resolve(g, [m, g.units[OP].clone()], o, cn);
                let s = evaluate(g, &spawn_plan, &out) - base;
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
    let mut input = Input { lines: stdin.lock().lines() };
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
    };
    let mut rng = Rng(0x9E3779B97F4A7C15);
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
        let budget = if g.turn == 0 { FIRST_TURN_BUDGET_MS } else { TURN_BUDGET_MS };
        let cmd = play_turn(&mut g, &mut rng, start, budget);
        eprintln!("turn {} score {}-{} tris {} time {}ms", g.turn, g.score[0], g.score[1], g.tris.len(), start.elapsed().as_millis());
        let mut lock = out.lock();
        writeln!(lock, "{}", cmd).unwrap();
        lock.flush().unwrap();
        g.turn += 1;
    }
}
