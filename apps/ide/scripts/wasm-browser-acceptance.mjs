const devtoolsBase = process.env.XEZIM_CHROME_DEVTOOLS ?? "http://127.0.0.1:9222";
const harnessUrl = process.env.XEZIM_BROWSER_TEST_URL ?? "http://127.0.0.1:4173/worker-harness.html";
const devtools = devtoolsBase.replace(/\/$/, "");

async function openTarget() {
  const response = await fetch(`${devtools}/json/new?${encodeURIComponent("about:blank")}`, { method: "PUT" });
  if (!response.ok) throw new Error(`Could not create a Chromium page (${response.status}).`);
  return response.json();
}

const target = await openTarget();
const socket = new WebSocket(target.webSocketDebuggerUrl);
let nextId = 1;
const pending = new Map();

socket.addEventListener("message", (event) => {
  const message = JSON.parse(String(event.data));
  if (typeof message.id !== "number") return;
  const request = pending.get(message.id);
  if (!request) return;
  pending.delete(message.id);
  clearTimeout(request.timer);
  if (message.error) request.reject(new Error(message.error.message));
  else request.resolve(message);
});

function command(method, params = {}) {
  const id = nextId++;
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(new Error(`Chromium DevTools command timed out: ${method}`));
    }, 10_000);
    pending.set(id, { resolve, reject, timer });
    socket.send(JSON.stringify({ id, method, params }));
  });
}

try {
  await new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", () => reject(new Error("Could not connect to Chromium DevTools.")), { once: true });
  });
  await command("Page.enable");
  await command("Runtime.enable");
  await command("Network.enable");
  await command("Network.setCacheDisabled", { cacheDisabled: true });
  await command("Page.navigate", { url: harnessUrl });

  const deadline = Date.now() + 180_000;
  while (Date.now() < deadline) {
    await new Promise((resolve) => setTimeout(resolve, 500));
    const response = await command("Runtime.evaluate", {
      expression: "JSON.stringify((()=>{const e=document.querySelector('#result');return {status:e?.dataset.status,text:e?.textContent}})())",
      returnByValue: true,
    });
    const state = JSON.parse(response.result.result.value ?? "{}");
    if (state.status === "starting") continue;
    console.log(state.text);
    if (state.status === "passed") process.exitCode = 0;
    else process.exitCode = 1;
    break;
  }
  if (process.exitCode === undefined) {
    console.error("Browser acceptance did not finish within 180 seconds.");
    process.exitCode = 2;
  }
} finally {
  socket.close();
  await fetch(`${devtools}/json/close/${target.id}`).catch(() => {});
}
