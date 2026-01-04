import { ForwardMethod } from "../domain/HistoryTree"
import { Pos } from "../wasm/pkg/mintaka_wasm"
import { AppGameState } from "../stores/app.state"

interface GameController {
    play: (pos: Pos) => void,
    forward: (method: ForwardMethod) => void,
    bulkForward: (method: ForwardMethod) => void,
    backward: () => void,
    bulkBackward: () => void,
}

export function createGameController(
    gameState: () => AppGameState,
    setGameState: (gameState: AppGameState) => void,
): GameController {
    return {
        play: (pos: Pos) => {
            if (pos ? !gameState().boardWorker.isLegalMove(pos) : true) return

            const boardWorker = gameState().boardWorker.set(pos)
            const hashKey = boardWorker.hashKey()

            const historyTree = gameState().historyTree.push({ hashKey, pos })

            setGameState({ boardWorker, historyTree })
        },
        forward: (method: ForwardMethod) => {
            const result = gameState().historyTree.forward(method)
            if (!result) return
            const [historyTree, entry] = result

            const boardWorker = gameState().boardWorker.set(entry.pos!)

            setGameState({ boardWorker, historyTree })
        },
        bulkForward: (method: ForwardMethod) => {
            const result = gameState().historyTree.bulkForward(method)
            if (!result) return
            const [historyTree, entries] = result

            let boardWorker = gameState().boardWorker
            for (const entry of entries)
                boardWorker = boardWorker.set(entry.pos)

            setGameState({ boardWorker, historyTree })
        },
        backward: () => {
            const result = gameState().historyTree.backward()
            if (!result) return
            const [historyTree, entry] = result

            const boardWorker = gameState().boardWorker.unset(entry.pos!)

            setGameState({ boardWorker, historyTree })
        },
        bulkBackward: () => {
            const result = gameState().historyTree.bulkBackward()
            if (!result) return
            const [historyTree, entries] = result

            let boardWorker = gameState().boardWorker
            for (const entry of entries.reverse())
                boardWorker = boardWorker.unset(entry.pos)

            setGameState({ boardWorker, historyTree })
        },
    }
}
