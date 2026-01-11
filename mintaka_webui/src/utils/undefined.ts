export function flatmap<T>(value: T | undefined, map: (valid: T) => T | undefined): T | undefined {
    if (value === undefined) return undefined
    else return map(value)
}

export function filter<T, S extends T>(value: T | undefined, predict: (valid: T) => valid is S): S | undefined {
    if (value !== undefined && predict(value)) return value
    else return undefined
}
