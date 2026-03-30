import Database from "@tauri-apps/plugin-sql";

// Instancia global de la base de datos (singleton)
let db: Database | null = null;

/**
 * Inicializa la conexión a SQLite.
 * Las migraciones se ejecutan automáticamente la primera vez.
 * "sqlite:pane.db" le dice al plugin que cree/abra "pane.db"
 * en el directorio de datos de la app.
 */
export async function initDatabase(): Promise<Database> {
  if (db) return db;
  db = await Database.load("sqlite:pane.db");
  console.log("Database initialized");
  return db;
}

/**
 * Devuelve la instancia de la DB.
 * Lanza error si no se llamó initDatabase() primero.
 */
export function getDatabase(): Database {
  if (!db) throw new Error("Database not initialized. Call initDatabase() first.");
  return db;
}
