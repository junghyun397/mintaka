export const WEB_WORKER_READY = typeof SharedArrayBuffer !== "undefined" && self.crossOriginIsolated

export const SERVER_URL = "localhost:10204"
export const SERVER_PROTOCOL = window.location.protocol.slice(0, -1)

export const PERSIST_CONFIG_VERSION = "v1"
export const MINTAKA_CONFIG_VERSION = "v1"
