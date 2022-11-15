globalThis.snapshots = [];

globalThis.saveSnapshot = ({ f: frame, m: maxSnapshots }) => {
    globalThis.snapshots[frame % maxSnapshots] = JSON.stringify(globalThis.jsState);
}
globalThis.loadSnapshot = ({ f: frame, m: maxSnapshots, e: entityMap }) => {
    globalThis.jsState = JSON.parse(globalThis.snapshots[frame % maxSnapshots]);

    if (globalThis.jsState.entityLists && entityMap.length > 0) {
        for (const listName in globalThis.jsState.entityLists) {
            const entityList = globalThis.jsState.entityLists[listName];
            for (const i in entityList) {
                const entity = entityList[i];
                for (const { t: to, f: from } of entityMap) {
                    if (from[0] == entity[0] && from[1] == entity[1]) {
                        entityList[i] = to;
                    }
                }
            }
        }
    }
}