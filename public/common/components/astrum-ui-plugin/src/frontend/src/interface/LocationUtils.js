

export function isPageHashIncluding(paths) {
  return paths.some(path => window.location.hash.includes(path));
}

export function scrollToUsageCenter() {
  const isPreviewUI = window.location.hash.startsWith('#preview');

  // In the Preview UI the Usage Center is no longer a scrollable section on the
  // dashboard overview; it lives in the dedicated "Usage Metrics" tab. Route the
  // user straight to that tab (ASTRUM-53862). The Classic UI still scrolls to the
  // in-page Usage Center element.
  if (isPreviewUI) {
    window.location.hash = '#preview/browse/welcome?tab=usage-metrics';
    return;
  }

  const targetPath = '#browse/welcome';
  const targetElementId = 'as-usage-center';

  function scrollToElement() {
    const usageCenterElement = document.getElementById(targetElementId);
    if (usageCenterElement) {
      usageCenterElement.scrollIntoView({ behavior: 'smooth' });
    }
  }

  if (window.location.hash !== targetPath) {
    window.location.hash = targetPath;
    setTimeout(scrollToElement, 200);
  } else {
    scrollToElement();
  }
}