const messageSource = "itu-calendar-api-observer";
const messageType = "calendar-response";

type CalendarMessage = {
  source: typeof messageSource;
  type: typeof messageType | "request-current";
  payload?: unknown;
};

let latestResponse: unknown;

function reportIfCalendarResponse(value: unknown): void {
  if (typeof value !== "object" || value === null || !("kayitSinifResultList" in value)) return;
  latestResponse = value;
  window.postMessage({ source: messageSource, type: messageType, payload: value } satisfies CalendarMessage, window.location.origin);
}

window.addEventListener("message", (event: MessageEvent<CalendarMessage>) => {
  if (event.source !== window || event.data?.source !== messageSource || event.data.type !== "request-current") return;
  if (latestResponse !== undefined) reportIfCalendarResponse(latestResponse);
});

const originalFetch = window.fetch;
window.fetch = function (...args: Parameters<typeof fetch>): Promise<Response> {
  return originalFetch.apply(this, args).then((response) => {
    if (response.ok) {
      void response.clone().json().then(reportIfCalendarResponse).catch(() => undefined);
    }
    return response;
  });
};

const originalXhrOpen = XMLHttpRequest.prototype.open;
XMLHttpRequest.prototype.open = function (
  method: string,
  url: string | URL,
  async?: boolean,
  username?: string | null,
  password?: string | null,
): void {
  this.addEventListener("load", function () {
    if (this.status < 200 || this.status >= 300) return;
    try {
      const body: unknown = this.responseType === "json" ? this.response : JSON.parse(this.responseText);
      reportIfCalendarResponse(body);
    } catch {
      // This XHR was not a JSON calendar response.
    }
  }, { once: true });
  originalXhrOpen.call(this, method, url, async ?? true, username ?? null, password ?? null);
};

export {};
