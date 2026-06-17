export function flatmap<I, O>(value: I | undefined, map: (valid: I) => O): O | undefined {
    return value === undefined ? undefined : map(value)
}

export function filter<T, S extends T>(value: T | undefined, predict: (valid: T) => valid is S): S | undefined {
    return (value !== undefined && predict(value)) ? value : undefined
}
