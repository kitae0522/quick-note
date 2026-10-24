"""Shared isolated note data for release resource comparisons."""
import sqlite3


def seed(directory, count):
    total = 0
    with sqlite3.connect(directory / "notes.sqlite3") as database:
        database.executescript("""
            PRAGMA encoding='UTF-8'; PRAGMA user_version=2;
            CREATE TABLE notes(id TEXT PRIMARY KEY NOT NULL,body TEXT NOT NULL,
                created_at REAL NOT NULL,updated_at REAL NOT NULL,
                selection_location INTEGER NOT NULL DEFAULT 0,
                selection_length INTEGER NOT NULL DEFAULT 0,title TEXT NOT NULL);
            CREATE TABLE app_state(key TEXT PRIMARY KEY NOT NULL,value TEXT NOT NULL);
        """)
        for index in range(count):
            title = f"Idea {index + 1:03}"
            body = f"# {title}\n\n아이디어 💡 café\n\n" + "A small idea, saved immediately.\n" * 32
            body += f"\nmarker-{index:03}"
            total += len(body.encode("utf-8"))
            cursor = len(body.encode("utf-16-le")) // 2
            database.execute("INSERT INTO notes VALUES(?,?,?,?,?,?,?)",
                             (str(index), body, index + 1, index + 1, cursor, 0, title))
        database.execute("INSERT INTO app_state VALUES('activeNote','0')")
    return total
