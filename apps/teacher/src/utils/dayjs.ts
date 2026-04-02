import dayjs, { Dayjs } from 'dayjs';
export type { Dayjs } from 'dayjs';
import utc from 'dayjs/plugin/utc';
import timezone from 'dayjs/plugin/timezone';
import customParseFormat from 'dayjs/plugin/customParseFormat';

// Plugins
dayjs.extend(utc);
dayjs.extend(timezone);
dayjs.extend(customParseFormat);

/**
 * Dayjs 工具封装
 */

/**
 * 格式化日期/时间
 *
 * @param d - 可接受字符串/数字/Date/Dayjs对象，undefined 或 null 返回空字符串
 * @param fmt - 输出格式，默认为 `YYYY-MM-DD HH:mm:ss`
 * @returns 格式化后的字符串
 */
export const format = (d?: string | number | Date | Dayjs, fmt = 'YYYY-MM-DD HH:mm:ss') => {
  if (d === undefined || d === null) return '';
  return dayjs(d).format(fmt);
};

/**
 * 转换为 Unix 毫秒时间戳
 *
 * @param d - 可接受字符串/数字/Date/Dayjs对象
 * @returns 时间戳或 null
 */
export const toTimestamp = (d?: string | number | Date | Dayjs) => {
  if (d === undefined || d === null) return null;
  return dayjs(d).valueOf();
};

/**
 * 从 Unix 毫秒时间戳创建 Dayjs 对象
 *
 * @param ts - 时间戳
 * @returns Dayjs 对象或 null
 */
export const fromTimestamp = (ts?: number | null) => {
  if (!ts) return null;
  return dayjs(ts);
};

/**
 * 使用指定格式解析字符串为 Dayjs 对象
 *
 * @param str - 时间字符串
 * @param fmt - 输入格式
 * @returns Dayjs 对象
 */
export const parse = (str: string, fmt = 'YYYY-MM-DD HH:mm:ss') => dayjs(str, fmt);

/**
 * 将 Unix 毫秒时间戳格式化为显示字符串，空或非法返回 "-"
 *
 * @param ts - 毫秒时间戳
 * @returns 格式化后的字符串，格式 `YYYY-MM-DD HH:mm:ss`
 */
export const formatTimestamp = (ts?: number | null) => {
  if (ts === undefined || ts === null) return '-';
  if (!Number.isFinite(ts)) return '-';
  return dayjs(ts).format('YYYY-MM-DD HH:mm:ss');
};

export default dayjs;
