

/**
 * Refreshes the current React page, handling URL parameter encoding
 * @param {Object} router - UI-Router instance from useRouter()
 */
export function refreshReactPage(router) {
  if (!router) {
    console.error('refreshReactPage: router is required');
    return;
  }

  const currentHash = window.location.hash;
  const colonIndex = currentHash.indexOf(':');

  // If URL has parameters after ':', re-encode them and reload via router
  if (colonIndex > 0) {
    const currentState = router.stateService.current;
    const currentParams = router.stateService.params;
    const param = currentHash.substring(colonIndex + 1);
    const encodedParam = encodeURIComponent(decodeURIComponent(param));

    // Find the parameter name from current route params (e.g., itemId, keyword, etc.)
    const paramName = Object.keys(currentParams).find(key => currentParams[key] && key !== '#');

    if (paramName) {
      router.stateService.go(currentState.name, {
        [paramName]: `:${encodedParam}`
      }, {
        reload: true,
        inherit: false,
        location: 'replace'
      });
      return;
    }
  }
  router.stateService.reload();
}
