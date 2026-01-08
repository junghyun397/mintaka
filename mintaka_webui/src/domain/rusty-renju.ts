import { Color, Pos } from "../wasm/pkg/mintaka_wasm"

export const NUMS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15] as const

export const LETTERS = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o'] as const

export const INDEX_TO_POS: Pos[] =
    NUMS.flatMap((num) =>
        LETTERS.map((letter) =>
            `${letter}${num}` as const,
        ),
    )

export function flip(color: Color): Color {
    return color === "Black" ? "White" : "Black"
}
