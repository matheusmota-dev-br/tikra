// A real VoiceOver walks a figure made with `--parts`, and what it says is written down: the check a
// screen reader's user would make, with Guidepup driving it. Run by .github/workflows/voiceover.yml.
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { voiceOver } from "@guidepup/guidepup";

const [page, out] = process.argv.slice(2);
// each part on its own stop, in these words: the figure's description, read first, says them too
const expected = ["“Idea”", "“Draft”", "“Review”", "from Idea to Draft", "from Draft to Review", "from Review to Draft"];
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

execFileSync("open", ["-a", "Safari", page]);
await sleep(4000);
await voiceOver.start();
await sleep(2000);
// Safari's toolbar can have the cursor first: into the page, once it is on it
let inside = false;
for (let k = 0; k < 28; k++) {
  await voiceOver.next();
  if (!inside && (await voiceOver.lastSpokenPhrase()).includes("web content")) {
    await voiceOver.interact();
    inside = true;
  }
}
const log = await voiceOver.spokenPhraseLog();
await voiceOver.stop();
writeFileSync(out, log.join("\n") + "\n");
console.log(log.join("\n"));

const stops = log.filter((said) => !said.startsWith("Diagram") && said.trim() !== "");
const missing = expected.filter((phrase) => !stops.some((said) => said.includes(phrase)));
const empty = stops.filter((said) => said.trim() === "image").length;
if (missing.length || empty) {
  console.error(`VoiceOver did not say: ${missing.join("; ") || "(nothing missing)"}. Empty stops: ${empty}`);
  process.exit(1);
}
console.log(`VoiceOver stopped once at each of the ${expected.length} parts`);
