import { createEffect, type JSX, ParentProps } from "solid-js"

export function Modal(props: ParentProps & {
    id: string,
    titleId: string,
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
        aria-labelledby={props.titleId}
        class="modal"
        onClose={props.onClose}
    >
        <div class="modal-box flex max-h-[calc(100dvh-10rem)] max-w-lg flex-col bg-base-200 p-0">
            <div class="flex items-center justify-between px-4 pt-3 pb-2">
                <h1 id={props.titleId} class="text leading-none font-bold">{props.title}</h1>
                <form method="dialog" class="flex items-center">
                    <button class="btn btn-xs btn-primary" aria-label="Close">X</button>
                </form>
            </div>
            <div class="divider my-0" />
            <div class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-4">
                {props.children}
            </div>
        </div>
        <form method="dialog" class="modal-backdrop">
            <button aria-label="Close"></button>
        </form>
    </dialog>
}
