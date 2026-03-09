import * as XLSX from 'xlsx';

export type XlsxCellValue = string | number | boolean | Date | null | undefined;
export type XlsxRow = Record<string, XlsxCellValue>;

export interface XlsxReadOptions {
  sheet?: string | number;
  raw?: boolean;
  defval?: XlsxCellValue;
}

const DEFAULT_SHEET_INDEX = 0;

/**
 * 从二进制内容读取并解析为 XLSX 工作簿对象。
 *
 * @param buffer - 包含 Excel 文件内容的 ArrayBuffer
 * @returns 已解析的 XLSX.WorkBook 对象，可用于后续读取工作表或转换数据
 */
export function readWorkbookFromArrayBuffer(buffer: ArrayBuffer): XLSX.WorkBook {
  return XLSX.read(buffer, { type: 'array' });
}

/**
 * 接受浏览器层面的 File 对象并读取其内容，然后解析为 XLSX.WorkBook。
 *
 * @param file - 用户通过 `<input type="file" />` 等方式获得的 Excel 文件
 * @returns 解析后的 XLSX.WorkBook，可能抛出读取或解析错误
 */
export async function readWorkbookFromFile(file: File): Promise<XLSX.WorkBook> {
  const buffer = await file.arrayBuffer();
  return readWorkbookFromArrayBuffer(buffer);
}

/**
 * 从 WorkBook 中根据名称或索引选择对应的工作表。
 *
 * @param workbook - 已解析的 XLSX 工作簿对象
 * @param sheet - 可选的工作表标识，字符串表示名字，数字表示索引，默认取第一个工作表
 * @returns 对应的 XLSX.WorkSheet 对象
 * @throws 当工作表不存在或索引超出范围时抛出错误
 */
export function getWorksheet(workbook: XLSX.WorkBook, sheet?: string | number): XLSX.WorkSheet {
  if (workbook.SheetNames.length === 0) {
    throw new Error('Excel 文件中没有可读取的工作表');
  }

  if (typeof sheet === 'string') {
    const namedSheet = workbook.Sheets[sheet];
    if (!namedSheet) {
      throw new Error(`未找到工作表: ${sheet}`);
    }
    return namedSheet;
  }

  const sheetIndex = sheet ?? DEFAULT_SHEET_INDEX;
  const sheetName = workbook.SheetNames[sheetIndex];
  if (!sheetName) {
    throw new Error(`工作表索引超出范围: ${sheetIndex}`);
  }

  return workbook.Sheets[sheetName];
}

/**
 * 把工作表转换为 JSON 行数组。
 */
export function readSheetAsRows<T extends Record<string, unknown> = XlsxRow>(
  workbook: XLSX.WorkBook,
  options: XlsxReadOptions = {},
): T[] {
  const worksheet = getWorksheet(workbook, options.sheet);

  return XLSX.utils.sheet_to_json<T>(worksheet, {
    raw: options.raw ?? false,
    defval: options.defval ?? '',
  });
}

/**
 * 一步完成：从 File 读取并转为 JSON 行数组。
 */
export async function parseXlsxFile<T extends Record<string, unknown> = XlsxRow>(
  file: File,
  options: XlsxReadOptions = {},
): Promise<T[]> {
  const workbook = await readWorkbookFromFile(file);
  return readSheetAsRows<T>(workbook, options);
}
