const { test } = require("node:test");
const assert = require("node:assert/strict");
const { LESSONS } = require("../tutorial.js");

test("each lesson can be started, and has steps", () => {
  assert.equal(new Set(LESSONS.map((l) => l.id)).size, LESSONS.length);
  for (const l of LESSONS) {
    assert.ok(l.title && l.about && typeof l.start === "function", l.id);
    assert.ok(l.steps.length > 0, l.id);
  }
});

test("each step that waits for the reader can also be shown", () => {
  for (const l of LESSONS) {
    for (const s of l.steps) {
      assert.ok(s.say && s.mark, l.id);
      if (!s.free) {
        assert.equal(typeof s.done, "function", `${l.id}: ${s.say}`);
        assert.equal(typeof s.show, "function", `${l.id}: ${s.say}`);
      }
    }
  }
});
