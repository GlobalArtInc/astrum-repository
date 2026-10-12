export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
export type JsonObject = { [key: string]: Json };
export type RepositoryDocument = JsonObject;

export function isObject(value: unknown): value is JsonObject {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

export function clone<T extends Json>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

export function getPath(document: JsonObject, path: readonly string[]): Json | undefined {
  let current: Json | undefined = document;
  for (const key of path) {
    if (!isObject(current)) {
      return undefined;
    }
    current = current[key];
  }
  return current;
}

export function setPath(document: JsonObject, path: readonly string[], value: Json): void {
  let current = document;
  for (const key of path.slice(0, -1)) {
    const next = current[key];
    if (isObject(next)) {
      current = next;
    } else {
      const created: JsonObject = {};
      current[key] = created;
      current = created;
    }
  }
  const last = path[path.length - 1];
  if (last !== undefined) {
    current[last] = value;
  }
}

export function sameJson(left: unknown, right: unknown): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function canonical(value: Json | undefined): Json | undefined {
  if (Array.isArray(value)) {
    return value.map((item) => canonical(item) ?? null);
  }
  if (isObject(value)) {
    const sorted: JsonObject = {};
    for (const key of Object.keys(value).sort()) {
      const inner = canonical(value[key]);
      if (inner !== undefined) {
        sorted[key] = inner;
      }
    }
    return sorted;
  }
  return value;
}
