
import { getContextPath, enforceHashRouting } from '../hashEnforcement';

describe('getContextPath', () => {
  let scriptEl;

  beforeEach(() => {
    // Remove any scripts added by previous tests
    document.querySelectorAll('script[data-test-rapture]').forEach(el => el.remove());
  });

  it('returns empty string when no rapture script is present', () => {
    expect(getContextPath()).toBe('');
  });

  it('returns empty string for root context path', () => {
    scriptEl = document.createElement('script');
    scriptEl.setAttribute('data-test-rapture', '');
    scriptEl.src = 'http://localhost:8081/static/rapture/resources/astrum-coreui-bundle.js';
    document.head.appendChild(scriptEl);

    expect(getContextPath()).toBe('');
  });

  it('returns context path when bundle is served under a non-root path', () => {
    scriptEl = document.createElement('script');
    scriptEl.setAttribute('data-test-rapture', '');
    scriptEl.src = 'https://example.com/astrum/static/rapture/resources/astrum-coreui-bundle.js';
    document.head.appendChild(scriptEl);

    expect(getContextPath()).toBe('/astrum');
  });

  it('returns nested context path', () => {
    scriptEl = document.createElement('script');
    scriptEl.setAttribute('data-test-rapture', '');
    scriptEl.src = 'https://example.com/company/astrum/static/rapture/resources/astrum-coreui-bundle.js';
    document.head.appendChild(scriptEl);

    expect(getContextPath()).toBe('/company/astrum');
  });
});

describe('enforceHashRouting', () => {
  let replaceSpy;

  function setLocation({ pathname, hash = '', search = '', origin = 'https://example.com' }) {
    delete window.location;
    window.location = { pathname, hash, search, origin, replace: jest.fn() };
    replaceSpy = window.location.replace;
  }

  function addRaptureScript(contextPath) {
    const script = document.createElement('script');
    script.setAttribute('data-test-rapture', '');
    script.src = `https://example.com${contextPath}/static/rapture/resources/astrum-coreui-bundle.js`;
    document.head.appendChild(script);
    return script;
  }

  beforeEach(() => {
    delete window.__nxHashEnforced;
    document.querySelectorAll('script[data-test-rapture]').forEach(el => el.remove());
  });

  it('does nothing when hash is already present', () => {
    setLocation({ pathname: '/admin/system/tasks', hash: '#/admin/system/tasks' });
    enforceHashRouting();
    expect(replaceSpy).not.toHaveBeenCalled();
  });

  it('does nothing when pathname is root', () => {
    setLocation({ pathname: '/' });
    enforceHashRouting();
    expect(replaceSpy).not.toHaveBeenCalled();
  });

  it('does not run twice (guards with __nxHashEnforced)', () => {
    setLocation({ pathname: '/admin/system/tasks' });
    window.__nxHashEnforced = true;
    enforceHashRouting();
    expect(replaceSpy).not.toHaveBeenCalled();
  });

  it('redirects non-hash URL to hash URL for root context path', () => {
    setLocation({ pathname: '/admin/system/tasks', origin: 'http://localhost:8081' });
    addRaptureScript('');
    enforceHashRouting();
    expect(replaceSpy).toHaveBeenCalledWith('http://localhost:8081/#/admin/system/tasks');
  });

  it('preserves context path when redirecting for non-root context path', () => {
    setLocation({ pathname: '/astrum/admin/system/tasks', origin: 'https://example.com' });
    addRaptureScript('/astrum');
    enforceHashRouting();
    expect(replaceSpy).toHaveBeenCalledWith('https://example.com/astrum/#/admin/system/tasks');
  });

  it('preserves context path for browse routes', () => {
    setLocation({ pathname: '/astrum/browse/components', origin: 'https://example.com' });
    addRaptureScript('/astrum');
    enforceHashRouting();
    expect(replaceSpy).toHaveBeenCalledWith('https://example.com/astrum/#/browse/components');
  });

  it('does not redirect when already at context path root', () => {
    setLocation({ pathname: '/astrum/', origin: 'https://example.com' });
    addRaptureScript('/astrum');
    enforceHashRouting();
    expect(replaceSpy).not.toHaveBeenCalled();
  });

  it('does not redirect when pathname equals context path without trailing slash', () => {
    setLocation({ pathname: '/astrum', origin: 'https://example.com' });
    addRaptureScript('/astrum');
    enforceHashRouting();
    expect(replaceSpy).not.toHaveBeenCalled();
  });

  it('preserves search query in the redirect URL', () => {
    setLocation({ pathname: '/astrum/admin/system/tasks', search: '?debug', origin: 'https://example.com' });
    addRaptureScript('/astrum');
    enforceHashRouting();
    expect(replaceSpy).toHaveBeenCalledWith('https://example.com/astrum/?debug#/admin/system/tasks');
  });

  it('falls back to stripping all leading slashes when no rapture script is found', () => {
    setLocation({ pathname: '/admin/system/tasks', origin: 'https://example.com' });
    // No rapture script added — contextPath defaults to ''
    enforceHashRouting();
    expect(replaceSpy).toHaveBeenCalledWith('https://example.com/#/admin/system/tasks');
  });
});
