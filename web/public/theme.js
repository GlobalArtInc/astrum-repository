(() => {
  let preference = null;
  try {
    preference = window.localStorage.getItem('astrum-theme-preference');
  } catch {}
  const light =
    preference === 'light' ||
    (preference === 'system' && window.matchMedia('(prefers-color-scheme: light)').matches);
  document.documentElement.dataset.theme = light ? 'light' : 'dark';
})();
