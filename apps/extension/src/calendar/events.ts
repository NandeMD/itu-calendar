import type { CalendarApiResponse, CalendarEvent, ClassMeeting, RegisteredCourse } from "./types";

const ISTANBUL_OFFSET_MINUTES = 180;
const dayByName: Record<string, string> = {
  monday: "MO",
  tuesday: "TU",
  wednesday: "WE",
  thursday: "TH",
  friday: "FR",
  saturday: "SA",
  sunday: "SU",
};
const dayByCode: Record<string, string> = {
  M: "MO",
  T: "TU",
  W: "WE",
  R: "TH",
  F: "FR",
  S: "SA",
  U: "SU",
};

function parseDate(value: string): Date | undefined {
  const match = /^(\d{4})-(\d{2})-(\d{2})/.exec(value);
  if (!match) return undefined;
  const [, yearText, monthText, dayText] = match;
  const date = new Date(Date.UTC(Number(yearText), Number(monthText) - 1, Number(dayText)));
  if (date.toISOString().slice(0, 10) !== `${yearText}-${monthText}-${dayText}`) return undefined;
  return date;
}

function dayCode(meeting: ClassMeeting): string | undefined {
  return dayByName[meeting.gunAdiEN.toLowerCase()] ?? dayByCode[meeting.gunKodu.toUpperCase()];
}

function firstOccurrence(termStart: Date, byDay: string): Date {
  const weekday = ["SU", "MO", "TU", "WE", "TH", "FR", "SA"].indexOf(byDay);
  const daysUntil = (weekday - termStart.getUTCDay() + 7) % 7;
  return new Date(termStart.getTime() + daysUntil * 24 * 60 * 60 * 1000);
}

function atIstanbulTime(date: Date, minuteOfDay: number): Date {
  const utcMillis = date.getTime() + minuteOfDay * 60_000 - ISTANBUL_OFFSET_MINUTES * 60_000;
  return new Date(utcMillis);
}

function courseTitle(course: RegisteredCourse): string {
  const english = course.dersDilKodu.toLowerCase().startsWith("en");
  const preferred = english ? course.dersAdiEN : course.dersAdiTR;
  return preferred.trim() || (english ? course.dersAdiTR : course.dersAdiEN).trim() || course.dersKodu;
}

function meetingLocation(meeting: ClassMeeting): string | undefined {
  const parts = [meeting.mekanAdi, meeting.binaAdi, meeting.kampusAdi]
    .map((part) => part.trim())
    .filter(Boolean);
  const uniqueParts = [...new Set(parts)];
  return uniqueParts.length ? uniqueParts.join(", ") : undefined;
}

function courseEvents(course: RegisteredCourse): CalendarEvent[] {
  const termStart = parseDate(course.donemBaslangicTarihi);
  const termEnd = parseDate(course.donemBitisTarihi);
  if (!termStart || !termEnd || termStart > termEnd) return [];
  const endOfTerm = new Date(atIstanbulTime(termEnd, 23 * 60 + 59).getTime() + 59_000);

  return course.sinifYerZaman.flatMap((meeting) => {
    const byDay = dayCode(meeting);
    if (!byDay || !Number.isInteger(meeting.baslangicSaati) || !Number.isInteger(meeting.bitisSaati)) return [];
    if (meeting.baslangicSaati < 0 || meeting.baslangicSaati >= 1440 || meeting.bitisSaati < meeting.baslangicSaati || meeting.bitisSaati >= 1440) return [];

    const occurrenceDate = firstOccurrence(termStart, byDay);
    if (occurrenceDate > termEnd) return [];
    const start = atIstanbulTime(occurrenceDate, meeting.baslangicSaati);
    // OBS reports the last included minute (for example 16:29), so add one minute for DTEND.
    const end = atIstanbulTime(occurrenceDate, meeting.bitisSaati + 1);
    const title = courseTitle(course);

    return [{
      uid: `obs-${course.id}-${meeting.id}@itu-calendar`,
      summary: `${course.dersKodu} ${title}`,
      description: `CRN ${course.crn}`,
      start,
      end,
      location: meetingLocation(meeting),
      recurrence: { frequency: "weekly", byDay, until: endOfTerm },
    }];
  });
}

/** Converts all weekly OBS meetings in a response into term-bounded events. */
export function calendarEventsFromResponse(response: CalendarApiResponse): CalendarEvent[] {
  return response.kayitSinifResultList.flatMap(courseEvents);
}
