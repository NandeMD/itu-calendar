declare const chrome: {
  storage: { local: { set(items: Record<string, unknown>): Promise<void> } };
};

const observerMessageSource = "itu-calendar-api-observer";
const observerMessageType = "calendar-response";

window.addEventListener("message", (event: MessageEvent<unknown>) => {
  if (event.source !== window || typeof event.data !== "object" || event.data === null) return;

  const message = event.data as { source?: unknown; type?: unknown; payload?: unknown };
  if (message.source !== observerMessageSource || message.type !== observerMessageType) return;

  const response = message.payload;
  if (typeof response !== "object" || response === null) return;
  const apiResponse = response as Record<string, unknown>;
  if (apiResponse.statusCode !== 0 || apiResponse.resultCode !== "SCCDersTakvimi" || !Array.isArray(apiResponse.kayitSinifResultList)) return;

  window.dispatchEvent(new CustomEvent("itu-calendar:response", { detail: response }));
  void chrome.storage.local.set({ calendarApiResponse: response, calendarCapturedAt: Date.now() }).catch((error: unknown) => {
    console.error("ITU Calendar could not save the detected schedule.", error);
  });
  console.info("ITU Calendar detected an OBS calendar response.");
});

// The observer keeps the latest response in case it arrived before this bridge started.
window.postMessage({ source: observerMessageSource, type: "request-current" }, window.location.origin);

export {};
