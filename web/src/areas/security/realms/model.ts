import type { Realm } from '../api';

export const LOCAL_REALM = 'AstrumAuthenticatingRealm';

export function realmLabel(realm: Realm): string {
  return realm.name || realm.id;
}

export function activeRealms(available: readonly Realm[], active: readonly string[]): Realm[] {
  const byId = new Map(available.map((realm) => [realm.id, realm]));
  return active.map((id) => byId.get(id) ?? { id, name: id });
}

export function inactiveRealms(available: readonly Realm[], active: readonly string[]): Realm[] {
  return available.filter((realm) => !active.includes(realm.id));
}

export function moveTo(active: readonly string[], from: number, to: number): string[] {
  if (from < 0 || from >= active.length) {
    return [...active];
  }
  const target = Math.max(0, Math.min(active.length - 1, to));
  const next = [...active];
  const [moved] = next.splice(from, 1);
  if (moved !== undefined) {
    next.splice(target, 0, moved);
  }
  return next;
}

export function moveBy(active: readonly string[], id: string, delta: number): string[] {
  const index = active.indexOf(id);
  return index < 0 ? [...active] : moveTo(active, index, index + delta);
}

export function activate(active: readonly string[], id: string): string[] {
  return active.includes(id) ? [...active] : [...active, id];
}

export function deactivate(active: readonly string[], id: string): string[] {
  return active.filter((item) => item !== id);
}

export function sameOrder(left: readonly string[], right: readonly string[]): boolean {
  return left.length === right.length && left.every((item, index) => item === right[index]);
}
