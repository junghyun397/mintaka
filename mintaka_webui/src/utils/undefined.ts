export function flatmap<T, R>(value: T | undefined, map: (t: T) => R | undefined): R | undefined {
    return value === undefined ? undefined : map(value);
}

export function mapOrDefault<T, R>(value: T | undefined, map: (t: T) => R, defaultValue: ()=> R): R {
    return value === undefined ? defaultValue() : map(value)
}

export function mapOrDefaultValue<T, R>(value: T | undefined, map: (t: T) => R, defaultValue: R): R {
    return value === undefined ? defaultValue : map(value)
}
