declare namespace MapElement {
  function getSpawnedEntities(): Entity[]
}

declare namespace Assets {
  function getHandleId(relative_path: string): HandleId;
}