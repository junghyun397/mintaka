export function chunk<T>(array: T[], size: number): T[][] {
    return Array.from({ length: Math.ceil(array.length / size) }, (_, i) =>
        array.slice(i * size, i * size + size)
    );
}

export function range(start: number, end: number): number[] {
    return Array.from({ length: end - start }, (_, i) => start + i)
}
