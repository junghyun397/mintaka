export function flatmap<I, O>(value: I | undefined, map: (valid: I) => O | undefined): O | undefined {
    if (value === undefined) return undefined
    else return map(value)
}

export function filter<T, S extends T>(value: T | undefined, predict: (valid: T) => valid is S): S | undefined {
    if (value !== undefined && predict(value)) return value
    else return undefined
}

export function isSomeAnd<T>(value: T | undefined, predict: (valid: T) => boolean): boolean {
    return value !== undefined && predict(value)
}
