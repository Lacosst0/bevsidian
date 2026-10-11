// Shared start-up logic: shows progress, hides the splash once Bevy draws, reports errors.
export async function boot(start) {
  const loading = document.getElementById("loading");
  const status = document.getElementById("status");
  const canvas = document.getElementById("bevy");
  status.textContent = "Starting Bevy…";
  try {
    await start();
  } catch (error) {
    // winit uses an exception to hand control to the browser's event loop.
    const message = String(error && error.message ? error.message : error);
    if (!message.includes("Using exceptions for control flow")) {
      loading.classList.add("error");
      status.textContent = "Failed to start: " + message;
      console.error(error);
      return;
    }
  }
  canvas.focus();
  // Give the first frames time to render before revealing the canvas.
  setTimeout(() => loading.classList.add("done"), 400);
}
