import { Modal } from "./Modal"

export function Branch(props: { open: boolean, onClose: () => void }) {
    return <Modal
        id="history_modal"
        title="History"
        open={props.open}
        onClose={props.onClose}
    >
    </Modal>
}
