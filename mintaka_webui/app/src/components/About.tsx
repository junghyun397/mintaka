import { Modal, Section, type ModalControlProps } from "./Modal"
import { getRustyRenjuVersion } from "rusty-renju-web/rusty-renju"
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
                <p><b>rusty-renju</b> v{getRustyRenjuVersion()}</p>
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
