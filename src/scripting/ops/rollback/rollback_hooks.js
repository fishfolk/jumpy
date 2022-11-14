globalThis.snapshots = [];

globalThis.saveSnapshot = ({ f: frame, m: maxSnapshots }) => {
    globalThis.snapshots[frame % maxSnapshots] = JSON.stringify(globalThis.jsState);
}
globalThis.loadSnapshot = ({ f: frame, m: maxSnapshots, e: entityMap }) => {
    // FIXME: Map entities
    globalThis.jsState = JSON.parse(globalThis.snapshots[frame % maxSnapshots]);
}