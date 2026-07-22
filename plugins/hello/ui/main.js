const pending = new Map();
let seq = 0;

window.addEventListener("message", (event) => {
  const message = event.data;
  if (!message) return;
  if (message.kind === "res" && pending.has(message.id)) {
    const { resolve, reject } = pending.get(message.id);
    pending.delete(message.id);
    if (message.ok) resolve(message.result);
    else reject(new Error(message.error?.message || "call failed"));
  } else if (message.kind === "evt") {
    document.getElementById("out").textContent = `event: ${JSON.stringify(message.payload)}`;
  }
});

function invokeCommand(command, args) {
  const id = String(++seq);
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    parent.postMessage({ kind: "req", id, method: "invokeCommand", params: { command, args } }, "*");
  });
}

document.getElementById("say").addEventListener("click", async () => {
  try {
    const result = await invokeCommand("hello.say");
    document.getElementById("out").textContent = JSON.stringify(result);
  } catch (err) {
    document.getElementById("out").textContent = String(err);
  }
});
