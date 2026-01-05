import { ForwardMethod, HistoryTree } from "../domain/HistoryTree"
import { BestMove, BoardWorker, Pos } from "../wasm/pkg/mintaka_wasm"
import { AppGameState } from "../stores/app.state"

interface GameController {
    play: (pos: Pos) => "ok" | "illegal",
    forward: (method: ForwardMethod) => "ok" | "illegal",
    bulkForward: (method: ForwardMethod) => "ok" | "illegal",
    backward: () => "ok" | "illegal",
    bulkBackward: () => "ok" | "illegal",
    applyBestMove: (bestMove: BestMove, historySnapshot: HistoryTree) => void,
}

export function createGameController(
    gameState: () => AppGameState,
    setGameState: (gameState: AppGameState) => void,
): GameController {
    return {
        play: (pos: Pos) => {
            if (pos ? !gameState().boardWorker.isLegalMove(pos) : true)
                return "illegal"

            const boardWorker = gameState().boardWorker.set(pos)
            const hashKey = boardWorker.hashKey()

            const historyTree = gameState().historyTree.push({ hashKey, pos })

            setGameState({ boardWorker, historyTree })

            return "ok"
        },
        forward: (method: ForwardMethod) => {
            const result = gameState().historyTree.forward(method)
            if (!result) return "illegal"
            const [historyTree, entry] = result

            const boardWorker = gameState().boardWorker.set(entry.pos!)

            setGameState({ boardWorker, historyTree })

            return "ok"
        },
        bulkForward: (method: ForwardMethod) => {
            const result = gameState().historyTree.bulkForward(method)
            if (!result) return "illegal"
            const [historyTree, entries] = result

            let boardWorker = gameState().boardWorker
            for (const entry of entries)
                boardWorker = boardWorker.set(entry.pos)

            setGameState({ boardWorker, historyTree })

            return "ok"
        },
        backward: () => {
            const result = gameState().historyTree.backward()
            if (!result) return "illegal"
            const [historyTree, entry] = result

            const boardWorker = gameState().boardWorker.unset(entry.pos!)

            setGameState({ boardWorker, historyTree })

            return "ok"
        },
        bulkBackward: () => {
            const result = gameState().historyTree.bulkBackward()
            if (!result) return "illegal"
            const [historyTree, entries] = result

            let boardWorker = gameState().boardWorker
            for (const entry of entries.reverse())
                boardWorker = boardWorker.unset(entry.pos)

            setGameState({ boardWorker, historyTree })

            return "ok"
        },
        applyBestMove: (bestMove: BestMove, historySnapshot: HistoryTree) => {
            let { boardWorker, historyTree } = gameState()

            if (bestMove.position_hash !== boardWorker.hashKey()) {
                historyTree = historySnapshot
                boardWorker = BoardWorker.fromHistory(historyTree.toHistory())
            }

            boardWorker = boardWorker.set(bestMove.best_move)
            historyTree = historyTree.push({ hashKey: boardWorker.hashKey(), pos: bestMove.best_move })

            setGameState({ boardWorker, historyTree })
        },
    }
}
