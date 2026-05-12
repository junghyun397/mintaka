import { Modal, ModalControlProps } from "./Modal"

export function Branch(props: ModalControlProps) {
    return <Modal
        id="history_modal"
        title="History"
        open={props.open}
        onClose={props.onClose}
    >
    </Modal>
}
