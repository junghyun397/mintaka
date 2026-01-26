import { Accessor, createEffect, createSignal } from "solid-js"

export function useStableSignal<T>(initial: NonNullable<T>, source: () => NonNullable<T> | undefined): Accessor<NonNullable<T>> {
    const [value, setValue] = createSignal<NonNullable<T>>(initial)

    createEffect(() => {
        const v = source()
        if (v !== undefined) setValue(() => v)
    })

    return value
}
