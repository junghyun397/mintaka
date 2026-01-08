export type Keys<S, T> = {
    [K in keyof T]: T[K] extends S ? K : never
}[keyof T]
