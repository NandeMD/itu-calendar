#[cfg(feature = "server")]
use std::cell::RefCell;

#[cfg(feature = "server")]
use crate::obs::Lesson;

#[cfg(feature = "server")]
use rusqlite::OptionalExtension;

#[cfg(feature = "server")]
thread_local! {
    pub static DB: RefCell<rusqlite::Connection> = RefCell::new({
        let conn = rusqlite::Connection::open("lessons.db").expect("Cannot open/create database!!!");

        // Create our lesson db
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS lessons (
                crn             INTEGER PRIMARY KEY CHECK (crn BETWEEN 0 AND 4294967295),
                course_code     TEXT NOT NULL,
                teaching_method TEXT NOT NULL,
                instructor      TEXT NOT NULL,
                buildings       TEXT NOT NULL,
                time            TEXT NOT NULL,
                day             TEXT NOT NULL,
                room            TEXT NOT NULL,
                capacity        INTEGER NOT NULL CHECK (capacity BETWEEN 0 AND 4294967295),
                enrolled        INTEGER NOT NULL CHECK (enrolled BETWEEN 0 AND 4294967295),
                majors          TEXT NOT NULL DEFAULT '[]'
            );
            "
        ).unwrap();

        conn
    });
}

#[cfg(feature = "server")]
pub fn upsert_lessons(lessons: &[crate::obs::Lesson]) -> rusqlite::Result<()> {
    DB.with(|conn| {
        let mut conn = conn.borrow_mut();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO lessons (
                    crn, course_code, teaching_method, instructor, buildings,
                    time, day, room, capacity, enrolled, majors
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ON CONFLICT(crn) DO UPDATE SET
                    course_code = excluded.course_code,
                    teaching_method = excluded.teaching_method,
                    instructor = excluded.instructor,
                    buildings = excluded.buildings,
                    time = excluded.time,
                    day = excluded.day,
                    room = excluded.room,
                    capacity = excluded.capacity,
                    enrolled = excluded.enrolled,
                    majors = excluded.majors",
            )?;

            for lesson in lessons {
                stmt.execute(rusqlite::params![
                    lesson.crn,
                    lesson.course_code,
                    lesson.teaching_method,
                    lesson.instructor,
                    lesson.buildings,
                    lesson.time,
                    lesson.day,
                    lesson.room,
                    lesson.capacity,
                    lesson.enrolled,
                    lesson.majors,
                ])?;
            }
        }
        tx.commit()
    })
}

#[cfg(feature = "server")]
pub fn find_lessons_by_crn(crns: Vec<u32>) -> rusqlite::Result<Vec<Lesson>> {
    let mut lessons: Vec<Lesson> = vec![];

    DB.with(|conn| {
        let conn = conn.borrow();
        let mut stmt = conn.prepare(
            "SELECT crn, course_code, teaching_method, instructor, buildings,
                    day, time, room, capacity, enrolled, majors
             FROM lessons
             WHERE crn = ?1",
        )?;

        for crn in crns {
            if let Some(lesson) = stmt
                .query_row([crn], |row| {
                    Ok(Lesson {
                        crn: row.get(0)?,
                        course_code: row.get(1)?,
                        teaching_method: row.get(2)?,
                        instructor: row.get(3)?,
                        buildings: row.get(4)?,
                        day: row.get(5)?,
                        time: row.get(6)?,
                        room: row.get(7)?,
                        capacity: row.get(8)?,
                        enrolled: row.get(9)?,
                        majors: row.get(10)?,
                    })
                })
                .optional()?
            {
                lessons.push(lesson);
            }
        }

        Ok(lessons)
    })
}
