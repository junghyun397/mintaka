import { BestMove, ComputingResource, Response } from "../wasm/pkg/mintaka_wasm";
import { MintakaProviderType } from "../domain/mintaka.provider";
import { createStore, SetStoreFunction } from "solid-js/store";
import { createEffect } from "solid-js";

type NormEval = { normEval: number }

export type WorkerState =
    | { type: "began", content: ComputingResource }
    | ({ type: "in-computing", content: Extract<Response, { type: "Status" }>["content"] } & NormEval)
    | ({ type: "finished", content: BestMove } & NormEval)

export type WorkerStore = {
    loadedProviderType?: MintakaProviderType,
    state?: WorkerState,
    autoLaunch: boolean,
    inComputing: boolean,
}

export function createWorkerStore(): [WorkerStore, SetStoreFunction<WorkerStore>] {
    const [workerStore, setWorkerStore] = createStore<WorkerStore>({
        loadedProviderType: undefined,
        state: undefined,
        autoLaunch: false,
        inComputing: false,
    })

    createEffect(() => {
        setWorkerStore("inComputing",
            workerStore.state?.type === "began" || workerStore.state?.type === "in-computing",
        )
    })

    return [workerStore, setWorkerStore]
}
