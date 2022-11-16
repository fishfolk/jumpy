const cloneObj = x => JSON.parse(JSON.stringify(x));
const jsEntityEq = (ent1, ent2) => {
    return ent1[0] == ent2[0] && ent1[1] == ent2[1];
}
if (!globalThis.Script) {
    globalThis.Script = {}
}

globalThis.Script.getInfo = () => {
    return bevyModJsScriptingOpSync('jumpy_script_get_info');
}

globalThis.Script.state = (init) => {
    const scriptId = Script.getInfo().path;
    if (!globalThis.jsState) globalThis.jsState = {};
    if (!globalThis.jsState.script) globalThis.jsState.script = {};
    if (!globalThis.jsState.script[scriptId]) globalThis.jsState.script[scriptId] = cloneObj(init) || {};
    return globalThis.jsState.script[scriptId];
}

globalThis.Script.getEntityList = (listName) => {
    if (!globalThis.jsState) globalThis.jsState = {};
    if (!globalThis.jsState.entityLists) globalThis.jsState.entityLists = {};
    if (!globalThis.jsState.entityLists[listName]) globalThis.jsState.entityLists[listName] = [];
    return globalThis.jsState.entityLists[listName].map(e => EntityRef.fromJs(e));
}

globalThis.Script.addEntityToList = (listName, entity) => {
    if (!globalThis.jsState) globalThis.jsState = {};
    if (!globalThis.jsState.entityLists) globalThis.jsState.entityLists = {};
    if (!globalThis.jsState.entityLists[listName]) globalThis.jsState.entityLists[listName] = [];
    let list = globalThis.jsState.entityLists[listName];
    const jsEntity = EntityRef.toJs(entity);
    list.push(jsEntity);
}

globalThis.Script.entityListContains = (listName, entity) => {
    if (!globalThis.jsState) globalThis.jsState = {};
    if (!globalThis.jsState.entityLists) globalThis.jsState.entityLists = {};
    if (!globalThis.jsState.entityLists[listName]) globalThis.jsState.entityLists[listName] = [];
    let list = globalThis.jsState.entityLists[listName];
    const jsEntity = EntityRef.toJs(entity);

    // Look for entity in list
    for (const item of list) {
        if (jsEntityEq(item, jsEntity)) {
            return true;
        }
    }

    return false;
}

globalThis.Script.removeEntityFromList = (listName, entity) => {
    if (!globalThis.jsState) globalThis.jsState = {};
    if (!globalThis.jsState.entityLists) globalThis.jsState.entityLists = {};
    if (!globalThis.jsState.entityLists[listName]) globalThis.jsState.entityLists[listname] = [];
    let list = globalThis.jsState.entityLists[listName];
    const jsEntity = EntityRef.toJs(entity);
    globalThis.jsState.entityLists[listName] = list.filter(x => !jsEntityEq(x, jsEntity));
}

globalThis.Script.clearEntityList = (listName) => {
    if (!globalThis.jsState) globalThis.jsState = {};
    if (!globalThis.jsState.entityLists) globalThis.jsState.entityLists = {};
    globalThis.jsState.entityLists[listName] = [];
}

globalThis.Script.entityStates = () => {
    if (!globalThis.jsState) globalThis.jsState = {};
    if (!globalThis.jsState.entity) globalThis.jsState.script = {};
    if (!globalThis.jsState.entity[scriptId]) globalThis.jsState.script[scriptId] = {};
    return globalThis.jsState.entity[scriptId];
}

globalThis.Script.getEntityState = (entity, init) => {
    const jsEntity = EntityRef.toJs(entity);
    const entityKey = JSON.stringify(jsEntity);
    const scriptId = Script.getInfo().path;
    if (!globalThis.jsState.entity) globalThis.jsState.entity = {};
    if (!globalThis.jsState.entity[scriptId]) globalThis.jsState.entity[scriptId] = {};
    if (!globalThis.jsState.entity[scriptId][entityKey]) globalThis.jsState.entity[scriptId][entityKey] = cloneObj(init) || {};
    return globalThis.jsState.entity[scriptId][entityKey];
}

globalThis.Script.setEntityState = (entity, state) => {
    const jsEntity = EntityRef.toJs(entity);
    const entityKey = JSON.stringify(jsEntity);
    const scriptId = Script.getInfo().path;
    if (!globalThis.jsState.entity) globalThis.jsState.entity = {};
    if (!globalThis.jsState.entity[scriptId]) globalThis.jsState.entity[scriptId] = {};
    globalThis.jsState.entity[scriptId][entityKey] = state;
}