export function openFresh(path: string): void {
  window.location.replace(path);
}

export function leaveFor(url: string): void {
  window.location.assign(url);
}
