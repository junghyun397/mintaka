import {MaybePos} from "../wasm/pkg";

export type HistoryEntry = {
    pos: MaybePos,
}

export type HistoryBranchMarker = {
    readonly branched: boolean
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

    get linear(): HistoryEntry[] {
        const acc = [this.history.slice(0, this.top)]

        let current = this.root
        while (current) {
            acc.push(current.history.slice(0, current.top))
            current = current.root
        }

        return acc.reverse().flat()
    }

    get length(): number {
        return (this.root?.length ?? 0) + this.top
    }

    get inBranchHead(): boolean {
        return this.top === 0
    }

    flatten(): HistoryTree {
        return new HistoryTree(undefined, this.linear)
    }

    push (entry: HistoryEntry): HistoryTree {
        if (this.top < this.history.length) {
            return new HistoryTree(this, [entry])
        } else {
            return new HistoryTree(this.root, this.history.concat(entry))
        }
    }

    backward (): [HistoryTree, HistoryEntry] | undefined {
        if (this.top === 0) {
            return this.root?.backward()
        } else if (this.top === 1 && this.root) {
            return [this.root, this.history[0]]
        } else {
            return [new HistoryTree(this.root, this.history, this.top - 1), this.history[this.top - 1]]
        }
    }

    forward (method: ForwardMethod): [HistoryTree, HistoryEntry] | undefined {
        if (this.top < this.history.length) {
            if (this.top === 0 && method === "return") {
                return this.root?.forward("continue")
            }

            return [new HistoryTree(this.root, this.history, this.top + 1), this.history[this.top]]
        }

        return undefined
    }

}
