// Worker frame clock for Chrome on Android.
//
// On 120 Hz screens, Chrome on Android limits a page's own frames (requestAnimationFrame and
// canvas commits) to 60 per second; the limit only lifts while a finger moves. A worker that
// owns a canvas gets its own frame clock, which is not limited. So, as flutter_shim and Uno
// Platform do for their canvas renderers:
//
// - The app's WebGL context comes from a hidden OffscreenCanvas instead of the page canvas.
// - Each finished frame is taken as an ImageBitmap and sent to a worker, which shows it on a
//   second canvas (placed under the page canvas) on its own frame clock.
// - requestAnimationFrame on the page is driven by that worker's clock.
//
// The page canvas stays in place, transparent, and still receives all input and sizing.
// On by default in Chrome on Android only; `?clock=worker` or `?clock=page` forces a mode.
// `window.__bvsClock` tells which one is active.
(() => {
  window.__bvsClock = "page";
  // Frames sent to and shown by the worker (diagnostics).
  const stats = (window.__bvsFrameStats = { sent: 0, shown: 0, requests: 0, ticks: 0 });
  const forced = new URLSearchParams(location.search).get("clock");
  const ua = navigator.userAgent;
  const androidChrome = /Android/.test(ua) && /Chrome\//.test(ua);
  const wanted = forced ? forced === "worker" : androidChrome;
  const canvas = document.getElementById("bevy");
  if (
    !wanted ||
    !canvas ||
    typeof OffscreenCanvas === "undefined" ||
    typeof Worker === "undefined" ||
    !HTMLCanvasElement.prototype.transferControlToOffscreen ||
    !OffscreenCanvas.prototype.transferToImageBitmap
  ) {
    return;
  }

  // Shows frames, oldest first, one per refresh; ticks the page while it wants frames;
  // stops a few refreshes after going idle.
  const workerSource = `
    let view = null, ctx = null, running = false, wanted = false, idle = 0, shown = 0;
    // The worker clock's own rate (refreshes per second while running): the ceiling for
    // this page, whatever the app manages.
    let since = 0, ticks = 0;
    const queue = [];
    onmessage = (event) => {
      const m = event.data;
      if (m.view) { view = m.view; ctx = view.getContext("bitmaprenderer"); }
      if (m.frame) {
        queue.push(m.frame);
        while (queue.length > 2) queue.shift().close();
      }
      if (m.want) wanted = true;
      if (!running) { running = true; idle = 0; since = 0; requestAnimationFrame(tick); }
    };
    function tick(time) {
      if (!since) since = time;
      ticks++;
      if (time - since >= 500) {
        postMessage({ hz: (ticks * 1000) / (time - since) });
        since = time;
        ticks = 0;
      }
      const frame = queue.shift();
      if (frame && ctx) {
        if (view.width !== frame.width) view.width = frame.width;
        if (view.height !== frame.height) view.height = frame.height;
        ctx.transferFromImageBitmap(frame);
        shown++;
      }
      if (wanted) {
        wanted = false;
        idle = 0;
        postMessage({ tick: true, shown });
      } else if (!frame && ++idle > 6) {
        running = false;
        return;
      }
      requestAnimationFrame(tick);
    }
  `;

  let worker;
  let view;
  try {
    view = document.createElement("canvas");
    view.id = "bevy-view";
    view.setAttribute("aria-hidden", "true");
    canvas.parentNode.insertBefore(view, canvas);
    const offscreenView = view.transferControlToOffscreen();
    worker = new Worker(URL.createObjectURL(new Blob([workerSource], { type: "text/javascript" })));
    worker.postMessage({ view: offscreenView }, [offscreenView]);
  } catch (error) {
    console.warn("worker frame clock unavailable:", error);
    view?.remove();
    worker?.terminate();
    return;
  }

  // requestAnimationFrame, paced by the worker.
  const pageRaf = window.requestAnimationFrame.bind(window);
  const pageCancel = window.cancelAnimationFrame.bind(window);
  let callbacks = new Map();
  let nextId = 1;
  let requested = false;
  const raf = (callback) => {
    const id = nextId++;
    stats.requests++;
    callbacks.set(id, callback);
    if (!requested) {
      requested = true;
      worker.postMessage({ want: true });
    }
    return id;
  };
  const cancel = (id) => callbacks.delete(id);
  worker.onmessage = (event) => {
    if (event.data.hz) {
      // Shown as "display … Hz" in the FPS readout.
      window.__bvsHz = event.data.hz;
      return;
    }
    stats.shown = event.data.shown;
    stats.ticks++;
    requested = false;
    const due = callbacks;
    callbacks = new Map();
    const now = performance.now();
    for (const callback of due.values()) {
      try {
        callback(now);
      } catch (error) {
        setTimeout(() => {
          throw error;
        });
      }
    }
  };

  // The app's WebGL context lives on an OffscreenCanvas that mirrors the page canvas size.
  const offscreen = new OffscreenCanvas(Math.max(canvas.width, 1), Math.max(canvas.height, 1));
  for (const property of ["width", "height"]) {
    const native = Object.getOwnPropertyDescriptor(HTMLCanvasElement.prototype, property);
    Object.defineProperty(canvas, property, {
      configurable: true,
      get() {
        return native.get.call(canvas);
      },
      set(value) {
        native.set.call(canvas, value);
        offscreen[property] = Math.max(value, 1);
      },
    });
  }

  let gl = null;
  const pageGetContext = canvas.getContext.bind(canvas);
  canvas.getContext = (type, attributes) => {
    if (type !== "webgl2") return pageGetContext(type, attributes);
    if (gl) return gl;
    gl = offscreen.getContext("webgl2", attributes);
    if (!gl) {
      // Fall back to the page canvas and clock.
      view.remove();
      worker.terminate();
      window.requestAnimationFrame = pageRaf;
      window.cancelAnimationFrame = pageCancel;
      canvas.getContext = pageGetContext;
      return pageGetContext(type, attributes);
    }
    trackPresents(gl);
    window.__bvsClock = "worker";
    return gl;
  };
  window.requestAnimationFrame = raf;
  window.cancelAnimationFrame = cancel;

  // Chrome delivers moves (touchmove, pointermove) just before the page's own frames. With the
  // app off the page clock, keep a no-op page frame loop going while a pointer is active, so
  // moves arrive promptly instead of piling up (a delayed move reads as a long-press).
  let lastInput = 0;
  let heartbeat = false;
  const beat = (time) => {
    if (time - lastInput > 250) {
      heartbeat = false;
      return;
    }
    pageRaf(beat);
  };
  const onInput = () => {
    lastInput = performance.now();
    if (!heartbeat) {
      heartbeat = true;
      pageRaf(beat);
    }
  };
  for (const type of ["pointerdown", "pointermove", "pointerup", "wheel"]) {
    window.addEventListener(type, onInput, { capture: true, passive: true });
  }

  // A frame is finished once something is drawn to the default framebuffer (wgpu's present);
  // it is sent after the current task, when the app's update has returned.
  function trackPresents(context) {
    let drawingToScreen = true;
    let pending = false;
    const send = () => {
      pending = false;
      const frame = offscreen.transferToImageBitmap();
      stats.sent++;
      worker.postMessage({ frame }, [frame]);
    };
    const bind = context.bindFramebuffer;
    context.bindFramebuffer = function (target, framebuffer) {
      // wasm-bindgen passes "no framebuffer" (the default one) as undefined.
      if (target !== context.READ_FRAMEBUFFER) drawingToScreen = framebuffer == null;
      return bind.call(this, target, framebuffer);
    };
    for (const name of ["drawArrays", "drawElements", "drawArraysInstanced", "drawElementsInstanced", "blitFramebuffer"]) {
      const draw = context[name];
      context[name] = function (...args) {
        if (drawingToScreen && !pending) {
          pending = true;
          queueMicrotask(send);
        }
        return draw.apply(this, args);
      };
    }
  }
})();
