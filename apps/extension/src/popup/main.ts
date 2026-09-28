import "./popup.css";
import { calendarEventsFromResponse } from "../calendar/events";
import { deserializeCalendarResponse } from "../calendar/deserialize";
import { serializeCalendar } from "../calendar/ical";
import type { CalendarApiResponse, CalendarEvent } from "../calendar/types";

declare const chrome: {
  storage: {
    local: {
      get(keys: string[]): Promise<Record<string, unknown>>;
      set(items: Record<string, unknown>): Promise<void>;
    };
    onChanged: {
      addListener(listener: (changes: Record<string, { newValue?: unknown }>, areaName: string) => void): void;
    };
  };
};

const status = document.querySelector<HTMLElement>("#status");
const exportButton = document.querySelector<HTMLButtonElement>("#export");
const themeSelect = document.querySelector<HTMLSelectElement>("#theme");
const languageSelect = document.querySelector<HTMLSelectElement>("#language");
const title = document.querySelector<HTMLElement>("#title");
const description = document.querySelector<HTMLElement>("#description");
const calendarSummary = document.querySelector<HTMLDetailsElement>("#calendar-summary");
const summaryTitle = document.querySelector<HTMLElement>("#summary-title");
const summaryList = document.querySelector<HTMLUListElement>("#summary-list");
const responseKey = "calendarApiResponse";
const capturedAtKey = "calendarCapturedAt";
const themeKey = "popupTheme";
const languageKey = "popupLanguage";

type StatusKind = "info" | "success" | "warning" | "error";
type Theme = "light" | "dark";
type Language = "tr" | "en";
type StatusMessage =
  | { key: "checking" | "noSchedule" | "noMeetings" | "storageError" }
  | { key: "ready"; courseCount: number; meetingCount: number; capturedAt?: number };

const translations: Record<Language, {
  title: string;
  description: string;
  export: string;
  themeLabel: string;
  themeLight: string;
  themeDark: string;
  summaryTitle(count: number): string;
  summaryRepeatUntil(date: string): string;
  status: Record<"checking" | "noSchedule" | "noMeetings" | "storageError", string> & {
    ready(courseCount: number, meetingCount: number, capturedAt?: number): string;
  };
}> = {
  tr: {
    title: "İTÜ Takvim",
    description: "Programınızı almak için İTÜ OBS Ders Takvimi sayfasını açın ya da yenileyin ve yüklenmesini bekleyin.",
    export: "iCalendar dosyasını indir (.ics)",
    themeLabel: "Renk teması",
    themeLight: "Açık",
    themeDark: "Koyu",
    summaryTitle: (count) => `Takvime eklenecekler (${count})`,
    summaryRepeatUntil: (date) => `Haftalık · ${date} tarihine kadar`,
    status: {
      checking: "Takvim verisi aranıyor…",
      noSchedule: "Henüz program alınmadı. İTÜ OBS Ders Takvimi sayfasını açın ve yüklenmesini bekleyin.",
      noMeetings: "Program bulundu ancak dışa aktarılabilecek haftalık ders bulunmuyor.",
      storageError: "Kaydedilen program okunamadı. Eklentiyi yeniden yüklemeyi deneyin.",
      ready: (courseCount, meetingCount, capturedAt) => {
        const timestamp = capturedAt === undefined ? "" : ` (alınma zamanı: ${new Date(capturedAt).toLocaleString("tr-TR")})`;
        return `${courseCount} ders ve ${meetingCount} haftalık ders programı dışa aktarılmaya hazır${timestamp}.`;
      },
    },
  },
  en: {
    title: "ITU Calendar",
    description: "Open or refresh the ITU OBS Ders Takvimi page and wait for its schedule to load.",
    export: "Download iCalendar (.ics)",
    themeLabel: "Color theme",
    themeLight: "Light",
    themeDark: "Dark",
    summaryTitle: (count) => `Calendar preview (${count})`,
    summaryRepeatUntil: (date) => `Weekly · through ${date}`,
    status: {
      checking: "Checking for a captured schedule…",
      noSchedule: "No schedule captured yet. Open the OBS Ders Takvimi page and wait for it to load.",
      noMeetings: "Schedule detected, but it contains no exportable weekly meetings.",
      storageError: "Could not read the saved schedule. Try reloading the extension.",
      ready: (courseCount, meetingCount, capturedAt) => {
        const timestamp = capturedAt === undefined ? "" : ` (captured ${new Date(capturedAt).toLocaleString("en-US")})`;
        return `${courseCount} courses and ${meetingCount} weekly meetings ready to export${timestamp}.`;
      },
    },
  },
};

let currentResponse: CalendarApiResponse | undefined;
let currentEvents: CalendarEvent[] = [];
let language: Language = "tr";
let currentStatus: { kind: StatusKind; message: StatusMessage } = {
  kind: "info",
  message: { key: "checking" },
};

function isTheme(value: unknown): value is Theme {
  return value === "light" || value === "dark";
}

function applyTheme(value: unknown): void {
  const theme: Theme = isTheme(value) ? value : "light";
  document.documentElement.dataset.theme = theme;
  if (themeSelect) themeSelect.value = theme;
}

function isLanguage(value: unknown): value is Language {
  return value === "tr" || value === "en";
}

function renderStatus(): void {
  if (!status) return;
  const messages = translations[language].status;
  const { message, kind } = currentStatus;
  status.className = `alert alert-${kind} text-sm`;
  status.textContent = message.key === "ready"
    ? messages.ready(message.courseCount, message.meetingCount, message.capturedAt)
    : messages[message.key];
}

function renderSummary(): void {
  if (!calendarSummary || !summaryTitle || !summaryList) return;
  const copy = translations[language];
  calendarSummary.hidden = currentEvents.length === 0;
  summaryTitle.textContent = copy.summaryTitle(currentEvents.length);
  summaryList.replaceChildren();

  const locale = language === "tr" ? "tr-TR" : "en-US";
  for (const event of currentEvents) {
    const item = document.createElement("li");
    item.className = "rounded-box border border-base-300 bg-base-100 p-3";

    const eventTitle = document.createElement("h3");
    eventTitle.className = "font-medium";
    eventTitle.textContent = event.summary;
    item.append(eventTitle);

    const time = document.createElement("p");
    time.className = "text-sm text-base-content/70";
    const weekday = new Intl.DateTimeFormat(locale, {
      weekday: "long",
      timeZone: "Europe/Istanbul",
    }).format(event.start);
    const firstDate = new Intl.DateTimeFormat(locale, {
      dateStyle: "medium",
      timeZone: "Europe/Istanbul",
    }).format(event.start);
    const startTime = new Intl.DateTimeFormat(locale, {
      hour: "2-digit",
      minute: "2-digit",
      timeZone: "Europe/Istanbul",
    }).format(event.start);
    const endTime = new Intl.DateTimeFormat(locale, {
      hour: "2-digit",
      minute: "2-digit",
      timeZone: "Europe/Istanbul",
    }).format(event.end);
    time.textContent = `${weekday}, ${firstDate} · ${startTime}–${endTime}`;
    item.append(time);

    if (event.recurrence) {
      const recurrenceEnd = new Intl.DateTimeFormat(locale, {
        dateStyle: "medium",
        timeZone: "Europe/Istanbul",
      }).format(event.recurrence.until);
      const recurrence = document.createElement("p");
      recurrence.className = "text-xs text-base-content/60";
      recurrence.textContent = copy.summaryRepeatUntil(recurrenceEnd);
      item.append(recurrence);
    }

    for (const extra of [event.location, event.description]) {
      if (!extra) continue;
      const detail = document.createElement("p");
      detail.className = "text-xs text-base-content/60";
      detail.textContent = extra;
      item.append(detail);
    }

    summaryList.append(item);
  }
}

function setStatus(message: StatusMessage, kind: StatusKind): void {
  currentStatus = { message, kind };
  renderStatus();
}

function applyLanguage(value: unknown): void {
  language = isLanguage(value) ? value : "tr";
  const copy = translations[language];
  document.documentElement.lang = language;
  document.title = copy.title;
  if (languageSelect) languageSelect.value = language;
  if (title) title.textContent = copy.title;
  if (description) description.textContent = copy.description;
  if (exportButton) exportButton.textContent = copy.export;
  if (themeSelect) {
    themeSelect.setAttribute("aria-label", copy.themeLabel);
    const lightOption = themeSelect.querySelector<HTMLOptionElement>('option[value="light"]');
    const darkOption = themeSelect.querySelector<HTMLOptionElement>('option[value="dark"]');
    if (lightOption) lightOption.textContent = copy.themeLight;
    if (darkOption) darkOption.textContent = copy.themeDark;
  }
  renderStatus();
  renderSummary();
}

function render(rawResponse: unknown, capturedAt?: unknown): void {
  currentResponse = deserializeCalendarResponse(rawResponse);
  currentEvents = currentResponse ? calendarEventsFromResponse(currentResponse) : [];
  renderSummary();

  if (exportButton) exportButton.disabled = currentEvents.length === 0;
  if (!currentResponse) {
    setStatus({ key: "noSchedule" }, "info");
    return;
  }
  if (currentEvents.length === 0) {
    setStatus({ key: "noMeetings" }, "warning");
    return;
  }

  const courseCount = currentResponse.kayitSinifResultList.length;
  setStatus({
    key: "ready",
    courseCount,
    meetingCount: currentEvents.length,
    capturedAt: typeof capturedAt === "number" ? capturedAt : undefined,
  }, "success");
}

function downloadCalendar(): void {
  if (!currentResponse || currentEvents.length === 0) return;

  const term = currentResponse.kayitSinifResultList[0]?.donem ?? "schedule";
  const safeTerm = term.replace(/[^a-zA-Z0-9_-]/g, "_");
  const calendarName = `ITU OBS Calendar ${term}`;
  const blob = new Blob([serializeCalendar(currentEvents, calendarName)], { type: "text/calendar;charset=utf-8" });
  const objectUrl = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = objectUrl;
  anchor.download = `itu-calendar-${safeTerm}.ics`;
  anchor.click();
  window.setTimeout(() => URL.revokeObjectURL(objectUrl), 1000);
}

exportButton?.addEventListener("click", downloadCalendar);

themeSelect?.addEventListener("change", () => {
  const theme: Theme = themeSelect.value === "dark" ? "dark" : "light";
  applyTheme(theme);
  void chrome.storage.local.set({ [themeKey]: theme }).catch((error: unknown) => {
    console.error("ITU Calendar could not save the selected theme.", error);
  });
});

languageSelect?.addEventListener("change", () => {
  const selectedLanguage: Language = languageSelect.value === "en" ? "en" : "tr";
  applyLanguage(selectedLanguage);
  void chrome.storage.local.set({ [languageKey]: selectedLanguage }).catch((error: unknown) => {
    console.error("ITU Calendar could not save the selected language.", error);
  });
});

void chrome.storage.local.get([responseKey, capturedAtKey, themeKey, languageKey]).then((stored) => {
  applyTheme(stored[themeKey]);
  applyLanguage(stored[languageKey]);
  render(stored[responseKey], stored[capturedAtKey]);
}).catch((error: unknown) => {
  applyTheme("light");
  applyLanguage("tr");
  setStatus({ key: "storageError" }, "error");
  console.error("ITU Calendar could not read extension storage.", error);
});

chrome.storage.onChanged.addListener((changes, areaName) => {
  if (areaName !== "local") return;
  if (changes[themeKey]) applyTheme(changes[themeKey].newValue);
  if (changes[languageKey]) applyLanguage(changes[languageKey].newValue);
  if (changes[responseKey]) render(changes[responseKey].newValue, changes[capturedAtKey]?.newValue);
});
