export function assertNever(value: never): never {
    throw new Error(`not never: ${value}`)
}
