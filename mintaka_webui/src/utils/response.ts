export function assertOk(result: unknown): asserts result is "ok" {
    if (result !== "ok") throw result
}
