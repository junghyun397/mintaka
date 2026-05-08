import { createSignal } from "solid-js"
import { Portal } from "solid-js/web"
import { IconInformationCircle } from "./icons"
import { Modal } from "./Modal"

export function AboutButton() {
    const [openAbout, setOpenAbout] = createSignal(false)

    return <>
        <button
            class="btn btn-square"
            onClick={() => setOpenAbout(true)}
        >
            <IconInformationCircle />
        </button>
        <Portal>
            <Modal
                id="about_modal"
                titleId="about-title"
                title="About mintaka WebUI"
                open={openAbout()}
                onClose={() => setOpenAbout(false)}
            >
                <a class="link text-sm" target="_blank" rel="noopener" href="https://github.com/junghyun397/mintaka">
                    github.com/junghyun397/mintaka
                </a>
            </Modal>
        </Portal>
    </>
}
