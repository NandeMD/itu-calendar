import type { CalendarEvent } from "./types";

const timezone = "Europe/Istanbul";
const encoder = new TextEncoder();

function escapeText(value: string): string {
  return value
    .replace(/\\/g, "\\\\")
    .replace(/\r\n|\r|\n/g, "\\n")
    .replace(/,/g, "\\,")
    .replace(/;/g, "\\;");
}

function foldLine(line: string): string {
  let folded = "";
  let segment = "";
  let segmentBytes = 0;

  for (const character of line) {
    const characterBytes = encoder.encode(character).length;
    if (segmentBytes + characterBytes > 75) {
      folded += `${segment}\r\n `;
      segment = "";
      segmentBytes = 1;
    }
    segment += character;
    segmentBytes += characterBytes;
  }

  return folded + segment;
}

function formatUtc(date: Date): string {
  return date.toISOString().replace(/[-:]/g, "").replace(/\.\d{3}/, "");
}

function formatIstanbul(date: Date): string {
  const wallTime = new Date(date.getTime() + 3 * 60 * 60 * 1000);
  const year = String(wallTime.getUTCFullYear()).padStart(4, "0");
  const month = String(wallTime.getUTCMonth() + 1).padStart(2, "0");
  const day = String(wallTime.getUTCDate()).padStart(2, "0");
  const hour = String(wallTime.getUTCHours()).padStart(2, "0");
  const minute = String(wallTime.getUTCMinutes()).padStart(2, "0");
  const second = String(wallTime.getUTCSeconds()).padStart(2, "0");
  return `${year}${month}${day}T${hour}${minute}${second}`;
}

/** Serializes recurring and one-off events as RFC 5545 iCalendar text. */
export function serializeCalendar(events: CalendarEvent[], calendarName = "ITU OBS Calendar"): string {
  const lines = [
    "BEGIN:VCALENDAR",
    "VERSION:2.0",
    "PRODID:-//ITU Calendar//OBS Schedule//EN",
    "CALSCALE:GREGORIAN",
    `X-WR-CALNAME:${escapeText(calendarName)}`,
    "BEGIN:VTIMEZONE",
    `TZID:${timezone}`,
    `X-LIC-LOCATION:${timezone}`,
    "BEGIN:STANDARD",
    "DTSTART:19700101T000000",
    "TZOFFSETFROM:+0300",
    "TZOFFSETTO:+0300",
    "TZNAME:TRT",
    "END:STANDARD",
    "END:VTIMEZONE",
  ];

  const timestamp = formatUtc(new Date());
  for (const event of events) {
    lines.push(
      "BEGIN:VEVENT",
      `UID:${escapeText(event.uid)}`,
      `DTSTAMP:${timestamp}`,
      `DTSTART;TZID=${timezone}:${formatIstanbul(event.start)}`,
      `DTEND;TZID=${timezone}:${formatIstanbul(event.end)}`,
    );
    if (event.recurrence) {
      lines.push(`RRULE:FREQ=WEEKLY;BYDAY=${event.recurrence.byDay};UNTIL=${formatUtc(event.recurrence.until)}`);
    }
    lines.push(`SUMMARY:${escapeText(event.summary)}`);
    if (event.description) lines.push(`DESCRIPTION:${escapeText(event.description)}`);
    if (event.location) lines.push(`LOCATION:${escapeText(event.location)}`);
    lines.push("STATUS:CONFIRMED", "TRANSP:OPAQUE", "END:VEVENT");
  }

  lines.push("END:VCALENDAR");
  return `${lines.map(foldLine).join("\r\n")}\r\n`;
}
