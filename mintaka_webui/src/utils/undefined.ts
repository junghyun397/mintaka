export function filter<T, S extends T>(value: T | undefined, predicate: (value: T) => value is S): S | undefined {
    if (value === undefined) return undefined
    return predicate(value) ? value : undefined
}

export function flatmap<T>(value: T | undefined, map: (value: T) => T | undefined): T | undefined {
    if (value === undefined) return undefined

    return map(value)
}
