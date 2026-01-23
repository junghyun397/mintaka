import { Color, HashKey, History, MaybePos } from "../wasm/pkg/rusty_renju_wasm"

export type HistoryEntry = {
    hashKey: HashKey,
    pos: MaybePos,
}

export type ForwardMethod = "continue" | "return"

export class HistoryTree {
    private readonly root?: HistoryTree
    private readonly history: readonly HistoryEntry[]
    private readonly top: number

    constructor(root: HistoryTree | undefined, history: readonly HistoryEntry[], top?: number) {
        this.root = root
        this.history = history
        this.top = top ?? history.length
    }

    get length(): number {
        return (this.root?.length ?? 0) + this.top
    }

    get backwardable(): boolean {
        return this.top > 0 || !!this.root?.backwardable
    }

    get forwardable(): boolean {
        return this.top < this.history.length || (this.top === 0 && !!this.root?.forwardable)
    }

    get inBranchHead(): boolean {
        return this.root != undefined && this.top === 0
    }

    get playerColor(): Color {
        return this.length % 2 === 0 ? "Black" : "White"
    }

    linear(): HistoryEntry[] {
        const acc = [this.history.slice(0, this.top)]

        let current = this.root
        while (current) {
            acc.push(current.history.slice(0, current.top))
            current = current.root
        }

        return acc.reverse().flat()
    }

    toHistory(): History {
        return this.linear().map(entry => entry.pos)
    }

    push(entry: HistoryEntry): HistoryTree {
        if (this.top < this.history.length)
            return new HistoryTree(this, [entry])
        else
            return new HistoryTree(this.root, this.history.concat(entry))
    }

    backward(): [HistoryTree, HistoryEntry] | undefined {
        if (this.top === 0)
            return this.root?.backward()
        else if (this.top === 1 && this.root)
            return [new HistoryTree(this.root, this.history, 0), this.history[0]]
        else
            return [new HistoryTree(this.root, this.history, this.top - 1), this.history[this.top - 1]]
    }

    bulkBackward(): [HistoryTree, HistoryEntry[]] | undefined {
        if (this.top === 0)
            return this.root?.bulkBackward()
        else
            return [new HistoryTree(this.root, this.history, 0), this.history.slice(0, this.top)]
    }

    forward(method: ForwardMethod): [HistoryTree, HistoryEntry] | undefined {
        if (this.top === 0 && method === "return")
            return this.root?.forward("continue")

        if (this.top >= this.history.length) return undefined

        return [new HistoryTree(this.root, this.history, this.top + 1), this.history[this.top]]
    }

    bulkForward(method: ForwardMethod): [HistoryTree, HistoryEntry[]] | undefined {
        if (this.top === 0 && method === "return")
            return this.root?.bulkForward("continue")

        if (this.top >= this.history.length) return undefined

        return [new HistoryTree(this.root, this.history), this.history.slice(this.top)]
    }
}

export const EmptyHistoryTree = new HistoryTree(undefined, [])
