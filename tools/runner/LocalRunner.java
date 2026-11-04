import com.codingame.gameengine.runner.MultiplayerGameRunner;
import com.codingame.gameengine.runner.simulate.GameResult;

// Usage: LocalRunner <seed> <league> <cmd0> <cmd1>
// Prints one line: "seed score0 score1 last-summary".
public class LocalRunner {
    public static void main(String[] args) {
        long seed = Long.parseLong(args[0]);
        MultiplayerGameRunner runner = new MultiplayerGameRunner();
        runner.setSeed(seed);
        runner.setLeagueLevel(Integer.parseInt(args[1]));
        runner.addAgent(args[2], "p0");
        runner.addAgent(args[3], "p1");
        GameResult result = runner.simulate();
        String last = "";
        for (String s : result.summaries) if (s != null && !s.trim().isEmpty()) last = s.trim().replace("\n", " | ");
        if (System.getenv("RUNNER_DEBUG") != null) {
            System.err.println("frames " + result.summaries.size());
            result.outputs.forEach((k, v) -> v.stream().filter(e -> e != null).limit(12).forEach(e -> System.err.println("out " + k + ": " + e.trim())));
            result.errors.forEach((k, v) -> v.stream().filter(e -> e != null).limit(5).forEach(e -> System.err.println("error " + k + ": " + e)));
        }
        System.out.println(seed + " " + result.scores.get(0) + " " + result.scores.get(1) + " " + last);
        System.exit(0);
    }
}
