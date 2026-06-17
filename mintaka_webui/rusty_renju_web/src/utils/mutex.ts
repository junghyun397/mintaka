export class Mutex {
    private tail: Promise<void> = Promise.resolve()

    async run<T>(fn: () => Promise<T> | T): Promise<T> {
        const prev = this.tail

        let release!: () => void
        this.tail = new Promise<void>((res) => (release = res))

        await prev
        try {
            return await fn()
        } finally {
            release()
        }
    }
}
