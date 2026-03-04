/**
 * 应用运行时配置结构。
 */
export interface AppConfig {
  appEnv: string;
  sqlite: {
    dbName: string;
    user: string;
    password: string;
    host: string;
    port: number;
  };
}

/**
 * 读取必填环境变量值。
 *
 * @param key - 环境变量键名（必须为 VITE_ 前缀）。
 * @returns 返回对应环境变量字符串值。
 * @throws 当环境变量不存在或为空时抛出错误。
 */
function getRequiredEnv(key: keyof ImportMetaEnv): string {
  const value = import.meta.env[key];

  if (!value || value.trim() === "") {
    throw new Error(`[config] Missing required env: ${String(key)}`);
  }

  return value;
}

/**
 * 统一导出的应用配置对象。
 *
 * @returns 返回当前环境下可用的配置集合。
 */
export const appConfig: AppConfig = {
  appEnv: getRequiredEnv("VITE_APP_ENV"),
  sqlite: {
    dbName: getRequiredEnv("VITE_SQLITE_DB_NAME"),
    user: getRequiredEnv("VITE_SQLITE_DB_USER"),
    password: getRequiredEnv("VITE_SQLITE_DB_PASSWORD"),
    host: getRequiredEnv("VITE_SQLITE_DB_HOST"),
    port: Number(getRequiredEnv("VITE_SQLITE_DB_PORT")),
  },
};
