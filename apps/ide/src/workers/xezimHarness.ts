import { runXezimWorker } from "./xezimClient";

async function main(): Promise<void> {
  const output = document.querySelector<HTMLPreElement>("#result");
  if (!output) throw new Error("Missing #result acceptance output element.");
  try {
    const fixture = await fetch("/fixtures/counter.sv");
    if (!fixture.ok) throw new Error(`Could not load counter fixture (${fixture.status}).`);
    const counterInput = {
      source: await fixture.text(),
      topModule: "counter",
      maxTimeNs: 50,
    };
    const response = await runXezimWorker(counterInput);
    const invalidSource = await runXezimWorker({
      source: "module invalid syntax; endmodule",
      topModule: "invalid",
      maxTimeNs: 50,
    });
    const recovery = await runXezimWorker(counterInput);
    const passed = response.ok
      && response.result.status === "completed"
      && response.result.finished
      && response.result.stdout.includes("COUNT=4")
      && invalidSource.ok
      && invalidSource.result.status === "failed"
      && invalidSource.result.diagnostics.some((diagnostic) => diagnostic.severity === "error")
      && recovery.ok
      && recovery.result.status === "completed"
      && recovery.result.finished
      && recovery.result.stdout.includes("COUNT=4");
    output.dataset.status = passed ? "passed" : "failed";
    output.textContent = JSON.stringify({ counter: response, invalidSource, recovery }, null, 2);
  } catch (error) {
    output.dataset.status = "failed";
    output.textContent = error instanceof Error ? error.stack ?? error.message : String(error);
  }
}

void main();
