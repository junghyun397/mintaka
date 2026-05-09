import { createEffect, type JSX, ParentProps } from "solid-js"
import { IconXMark } from "./icons"

export function Modal(props: ParentProps & {
    id: string,
    title: JSX.Element,
    open: boolean,
    onClose: () => void,
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
            <div class="flex items-center justify-between px-4 pt-3 pb-2">
                <h1 class="text font-bold">{props.title}</h1>
                <form method="dialog" class="flex items-center">
                    <button class="btn btn-square p-0.5 btn-xs btn-primary">
                        <IconXMark />
                    </button>
                </form>
            </div>
            <div class="divider my-0" />
            <div class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-4">
                {props.children}
            </div>
        </div>
        <form method="dialog" class="modal-backdrop"><button /></form>
    </dialog>
}
