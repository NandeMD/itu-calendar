import type { CalendarApiResponse, ClassMeeting, RegisteredCourse } from "./types";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function hasStringFields(value: Record<string, unknown>, fields: string[]): boolean {
  return fields.every((field) => typeof value[field] === "string");
}

function deserializeMeeting(value: unknown): ClassMeeting | undefined {
  if (!isRecord(value)) return undefined;
  if (typeof value.id !== "number" || typeof value.baslangicSaati !== "number" || typeof value.bitisSaati !== "number") {
    return undefined;
  }
  if (!hasStringFields(value, [
    "gunAdiEN", "gunAdiTR", "gunKodu", "kampusAdi", "kampusKodu", "binaAdi", "binaKodu",
    "mekanAdi", "mekanKapiTabelasi", "mekanTipiAdi",
  ])) return undefined;

  return value as unknown as ClassMeeting;
}

function deserializeCourse(value: unknown): RegisteredCourse | undefined {
  if (!isRecord(value) || typeof value.id !== "number") return undefined;
  if (!hasStringFields(value, [
    "donem", "crn", "bransKodu", "dersKodu", "dersAdiTR", "dersAdiEN",
    "donemBaslangicTarihi", "donemBitisTarihi", "dersDilKodu",
  ])) return undefined;
  if (!Array.isArray(value.sinifYerZaman)) return undefined;

  const meetings = value.sinifYerZaman.map(deserializeMeeting);
  if (meetings.some((meeting) => meeting === undefined)) return undefined;

  return { ...value, sinifYerZaman: meetings as ClassMeeting[] } as RegisteredCourse;
}

/** Converts an unknown JSON value into the typed OBS calendar response shape. */
export function deserializeCalendarResponse(value: unknown): CalendarApiResponse | undefined {
  if (!isRecord(value) || value.statusCode !== 0) return undefined;
  if (value.resultCode !== "SCCDersTakvimi" || typeof value.resultMessage !== "string") return undefined;
  if (!Array.isArray(value.kayitSinifResultList)) return undefined;

  const courses = value.kayitSinifResultList.map(deserializeCourse);
  if (courses.some((course) => course === undefined)) return undefined;

  return { ...value, kayitSinifResultList: courses as RegisteredCourse[] } as CalendarApiResponse;
}
