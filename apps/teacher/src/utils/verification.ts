
export const isNull = (params: any): boolean => {
    if (params === null || params === undefined) return true;  // 更明确的检查 null 和 undefined
    if (typeof params === 'string' && params.trim().length === 0) return true;
    if (Array.isArray(params) && params.length === 0) return true;
    if (typeof params === 'object' && Object.keys(params).length === 0) return true; // 处理空对象
    return false;
}

export const isEmail = (email: string): boolean => {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
}