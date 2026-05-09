import { Modal } from "./Modal"

export function About(props: { open: boolean, onClose: () => void }) {
    return <Modal
        id="about_modal"
        title="About App"
        open={props.open}
        onClose={props.onClose}
    >
        <a class="link text-sm" target="_blank" rel="noopener" href="https://github.com/junghyun397/mintaka">
            github.com/junghyun397/mintaka
        </a>
    </Modal>
}
