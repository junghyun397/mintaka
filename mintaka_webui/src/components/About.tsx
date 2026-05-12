import { Modal, ModalControlProps, Section } from "./Modal"
import { rustyRenjuVersion } from "../wasm/pkg/rusty_renju_wasm"
import pkg from '../../package.json'
import { useContext } from "solid-js"
import { AppContext } from "../context"

export function About(props: ModalControlProps) {
    const { runtimeSelectors } = useContext(AppContext)!

    return <Modal
        id="about_modal"
        title="About App"
        open={props.open}
        onClose={props.onClose}
    >
        <div class="flex flex-col gap-4">
            <Section title="Version">
                <p><b>webui</b> v{pkg.version}</p>
                <p><b>rusty-renju</b> v{rustyRenjuVersion()}</p>
                <p><b>mintaka</b> {runtimeSelectors.version()}</p>
            </Section>
            <Section title="Author">
                <p>JeongHyeon Choi</p>
            </Section>
            <Section title="Repository">
                <a class="link text-sm" target="_blank" rel="noopener" href="https://github.com/junghyun397/mintaka">
                    https://github.com/junghyun397/mintaka
                </a>
            </Section>
        </div>
    </Modal>
}
