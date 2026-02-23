(function () {
  const logEl = document.getElementById("log");
  const pending = new Map();
  let nextId = 1;

  function log(...args) {
    const s = args
      .map((a) => (typeof a === "string" ? a : JSON.stringify(a, null, 2)))
      .join(" ");
    logEl.textContent += s + "\n";
    logEl.scrollTop = logEl.scrollHeight;
  }

  function invoke(cmd, args = {}) {
    const id = nextId++;
    const payload = { id, cmd, args };

    return new Promise((resolve, reject) => {
      pending.set(id, { resolve, reject });

      if (!window.ipc || typeof window.ipc.postMessage !== "function") {
        pending.delete(id);
        reject(
          new Error(
            "window.ipc.postMessage is not available (are you running inside wry?)",
          ),
        );
        return;
      }

      window.ipc.postMessage(JSON.stringify(payload));
    });
  }

  // Rust calls: window.__demo_ipc._handleResponse({ id, ok, data?, error? })
  function _handleResponse(resp) {
    const slot = pending.get(resp.id);
    if (!slot) return;

    pending.delete(resp.id);

    if (resp.ok) slot.resolve(resp.data);
    else slot.reject(new Error(resp.error || "invoke failed"));
  }

  window.__demo_ipc = { invoke, _handleResponse };

  // UI wiring:
  document.getElementById("btn-greet").addEventListener("click", async () => {
    const name = document.getElementById("name").value || "world";
    log("-> invoke greet", { name });
    try {
      const out = await invoke("greet", { name });
      log("<- greet ok:", out);
    } catch (e) {
      log("<- greet err:", String(e));
    }
  });

  document.getElementById("btn-add").addEventListener("click", async () => {
    const a = Number(document.getElementById("a").value);
    const b = Number(document.getElementById("b").value);
    log("-> invoke add", { a, b });
    try {
      const out = await invoke("add", { a, b });
      log("<- add ok:", out);
    } catch (e) {
      log("<- add err:", String(e));
    }
  });

  document.getElementById("btn-time").addEventListener("click", async () => {
    log("-> invoke time_ms");
    try {
      const out = await invoke("time_ms");
      log("<- time_ms ok:", out);
    } catch (e) {
      log("<- time_ms err:", String(e));
    }
  });

  document.getElementById("btn-spam").addEventListener("click", async () => {
    log("-> spamming 10 invokes...");
    const jobs = [];
    for (let i = 0; i < 10; i++) jobs.push(invoke("add", { a: i, b: 100 }));
    const results = await Promise.all(jobs);
    log("<- spam results:", results);
  });

  log("ready. try the buttons!");
})();
