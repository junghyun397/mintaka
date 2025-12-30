import { useContext } from "solid-js";
import { AppContext } from "../context";

export function StatusMessage() {
    const { workerStore } = useContext(AppContext)!

    const statusMessage = () => {
        switch (1+2) {
            case 1: return "Downloading mintaka engine."
            case 2: return "Compiling mintaka engine."
            case 3: return "Mintaka engine is waiting for your move."
        }
    }

    return <p class="text-sm leading-tight text-base-content/70">
        {statusMessage()}
    </p>
}
