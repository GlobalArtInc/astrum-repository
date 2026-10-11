

export function isPageHashIncluding(paths) {
  return paths.some(path => window.location.hash.includes(path));
}

export function scrollToUsageCenter() {
  // The Usage Center is no longer a scrollable section on the dashboard overview;
  // it lives in the dedicated "Usage Metrics" tab. Route the user straight to that
  // tab (ASTRUM-53862).
  window.location.hash = '#browse/welcome?tab=usage-metrics';
}