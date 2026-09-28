/** One scheduled meeting returned by the OBS Ders Takvimi endpoint. */
export interface ClassMeeting {
  id: number;
  baslangicSaati: number;
  bitisSaati: number;
  gunAdiEN: string;
  gunAdiTR: string;
  gunKodu: string;
  kampusAdi: string;
  kampusKodu: string;
  binaAdi: string;
  binaKodu: string;
  mekanAdi: string;
  mekanKapiTabelasi: string;
  mekanTipiAdi: string;
}

/** A registered course section and its scheduled meetings. */
export interface RegisteredCourse {
  id: number;
  donem: string;
  crn: string;
  bransKodu: string;
  dersKodu: string;
  dersAdiTR: string;
  dersAdiEN: string;
  sinifYerZaman: ClassMeeting[];
  donemBaslangicTarihi: string;
  donemBitisTarihi: string;
  dersDilKodu: string;
}

/** Raw response shape returned by SCCDersTakvimi. */
export interface CalendarApiResponse {
  kayitSinifResultList: RegisteredCourse[];
  statusCode: number;
  resultCode: string;
  resultMessage: string;
}

export interface CalendarEvent {
  uid: string;
  summary: string;
  start: Date;
  end: Date;
  location?: string;
  description?: string;
  recurrence?: {
    frequency: "weekly";
    byDay: string;
    until: Date;
  };
}
