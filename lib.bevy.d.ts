declare namespace Deno {
  namespace core {
    function opSync(op: string, ...args: any[]): any;
  }
}

// log.s
declare function trace(...args: any): void;
declare function debug(...args: any): void;
declare function info(...args: any): void;
declare function warn(...args: any): void;
declare function error(...args: any): void;

// ecs.js
declare interface BevyScript {
  first?: () => void;
  preUpdate?: () => void;
  update?: () => void;
  postUpdate?: () => void;
  last?: () => void;
}

declare class ComponentId {
  index: number;
}

type ComponentInfo = {
  id: ComponentId;
  name: string;
  size: number;
};

type QueryDescriptor = {
  components: ComponentId[];
};

type Primitive = number | string | boolean;
interface Value {
  [path: string | number]: Value | Primitive | undefined;
}

type BevyType<T> = {
  typeName: string;
};

type ExtractBevyType<T> = T extends BevyType<infer U>
  ? U
  : T extends ComponentId
  ? Value
  : never;
type MapQueryArgs<Q> = { [C in keyof Q]: ExtractBevyType<Q[C]> };

type QueryParameter = BevyType<unknown> | ComponentId;
type QueryItem<Q> = {
  entity: Entity;
  components: MapQueryArgs<Q>;
};

declare class QueryItems<Q> extends Array<QueryItem<Q>> {
  get(entity: Entity): MapQueryArgs<Q> | undefined;
}

declare class World {
  get components(): ComponentInfo[];
  get resources(): ComponentInfo[];
  get entities(): Entity[];

  resource(componentId: ComponentId): Value | null;
  resource<T>(type: BevyType<T>): T | null;

  query<Q extends QueryParameter[]>(...query: Q): QueryItems<Q>;
  get<Q extends QueryParameter[]>(
    entity: Entity,
    ...components: Q
  ): MapQueryArgs<Q>;
}

declare let world: World;
