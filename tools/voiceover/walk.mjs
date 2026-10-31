// A real VoiceOver walks a figure made with `--parts`, and what it says is written down: the check a
// screen reader's user would make, with Guidepup driving it. Run by .github/workflows/voiceover.yml.
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { voiceOver } from "@guidepup/guidepup";

const [page, out] = process.argv.slice(2);
const expected = ["Idea", "Draft", "Review", "from Idea to Draft", "from Draft to Review", "from Review to Draft"];
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

execFileSync("open", ["-a", "Safari", page]);
await sleep(4000);
await voiceOver.start();
await sleep(2000);
for (let k = 0; k < 24; k++) {
  await voiceOver.next();
}
const log = await voiceOver.spokenPhraseLog();
await voiceOver.stop();
writeFileSync(out, log.join("\n") + "\n");
console.log(log.join("\n"));

const missing = expected.filter((phrase) => !log.some((said) => said.includes(phrase)));
if (missing.length) {
  console.error(`VoiceOver did not say: ${missing.join("; ")}`);
  process.exit(1);
}
console.log(`VoiceOver said all ${expected.length} parts`);
