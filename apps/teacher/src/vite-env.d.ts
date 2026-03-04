/// <reference types="vite/client" />

interface ImportMetaEnv {
	readonly VITE_APP_ENV: string;
	readonly VITE_SQLITE_DB_NAME: string;
	readonly VITE_SQLITE_DB_USER: string;
	readonly VITE_SQLITE_DB_PASSWORD: string;
	readonly VITE_SQLITE_DB_HOST: string;
	readonly VITE_SQLITE_DB_PORT: string;
}

interface ImportMeta {
	readonly env: ImportMetaEnv;
}
