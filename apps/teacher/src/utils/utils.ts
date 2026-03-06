export const omitString = (str: string, auto_size: number = 20): string => {
    return str.length >= auto_size ? `${str.slice(0, auto_size + 1)}...` : str
}

/**
 * 深度克隆（工程级）
 * @param target 需要克隆的对象
 */
export function deepClone<T>(target: T, weakMap = new WeakMap()): T {
  // 基础类型 & null
  if (target === null || typeof target !== 'object') {
    return target;
  }

  // 处理循环引用
  if (weakMap.has(target as object)) {
    return weakMap.get(target as object);
  }

  // Date
  if (target instanceof Date) {
    return new Date(target.getTime()) as T;
  }

  // RegExp
  if (target instanceof RegExp) {
    return new RegExp(target.source, target.flags) as T;
  }

  // Map
  if (target instanceof Map) {
    const result = new Map();
    weakMap.set(target, result);
    target.forEach((value, key) => {
      result.set(deepClone(key, weakMap), deepClone(value, weakMap));
    });
    return result as T;
  }

  // Set
  if (target instanceof Set) {
    const result = new Set();
    weakMap.set(target, result);
    target.forEach(value => {
      result.add(deepClone(value, weakMap));
    });
    return result as T;
  }

  // Array
  if (Array.isArray(target)) {
    const result: any[] = [];
    weakMap.set(target, result);
    target.forEach((item, index) => {
      result[index] = deepClone(item, weakMap);
    });
    return result as T;
  }

  // Object
  const result = {} as T;
  weakMap.set(target as object, result);

  Object.keys(target as object).forEach(key => {
    (result as any)[key] = deepClone(
      (target as any)[key],
      weakMap
    );
  });

  return result;
}
