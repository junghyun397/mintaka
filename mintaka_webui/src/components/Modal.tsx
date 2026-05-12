import { createEffect, type JSX, ParentProps } from "solid-js"
import { IconXMark } from "./icons"

export type ModalControlProps = {
    open: boolean,
    onClose: () => void,
}

export function Modal(props: ParentProps & ModalControlProps & {
    id: string,
    title: JSX.Element,
}) {
    let dialogRef: HTMLDialogElement | undefined

    createEffect(() => {
        if (props.open) {
            if (!dialogRef?.open) dialogRef?.showModal()
            return
        }

        if (dialogRef?.open) dialogRef.close()
    })

    return <dialog
        ref={ref => dialogRef = ref}
        id={props.id}
        class="modal"
        onClose={props.onClose}
    >
        <div class="modal-box flex max-h-[calc(100dvh-10rem)] max-w-lg flex-col bg-base-200 p-0">
            <div class="flex items-center justify-between px-4 py-3">
                <h1 class="text font-bold">{props.title}</h1>
                <form method="dialog" class="flex items-center">
                    <button class="btn btn-square p-0.5 btn-xs btn-primary">
                        <IconXMark />
                    </button>
                </form>
            </div>
            <div class="border-t border-base-content/10" />
            <div class="flex min-h-0 flex-1 overflow-y-auto p-4">
                {props.children}
            </div>
        </div>
        <form method="dialog" class="modal-backdrop"><button /></form>
    </dialog>
}

export function Section(props: ParentProps & { title: string }) {
    return <div class="flex flex-col gap-1">
        <h3 class="text font-bold">{props.title}</h3>
        {props.children}
    </div>
}
