import { execSync } from "node:child_process";

const PORTS = [1420, 1421];

function killWindowsPort(port) {
  try {
    const output = execSync(`netstat -ano | findstr ":${port}"`, {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    });

    const pids = new Set();
    for (const line of output.split(/\r?\n/)) {
      if (!line.includes("LISTENING")) continue;
      const pid = line.trim().split(/\s+/).at(-1);
      if (pid && pid !== "0") pids.add(pid);
    }

    for (const pid of pids) {
      try {
        execSync(`taskkill /PID ${pid} /F`, { stdio: "ignore" });
        console.log(`[kill-dev-ports] port ${port}: stopped PID ${pid}`);
      } catch {
        // Process may have already exited.
      }
    }
  } catch {
    // No listeners on this port.
  }
}

function killUnixPort(port) {
  try {
    const output = execSync(`lsof -ti :${port}`, {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    });

    for (const pid of output.split(/\r?\n/).filter(Boolean)) {
      try {
        process.kill(Number(pid), "SIGTERM");
        console.log(`[kill-dev-ports] port ${port}: stopped PID ${pid}`);
      } catch {
        // Process may have already exited.
      }
    }
  } catch {
    // No listeners on this port.
  }
}

for (const port of PORTS) {
  if (process.platform === "win32") {
    killWindowsPort(port);
  } else {
    killUnixPort(port);
  }
}
