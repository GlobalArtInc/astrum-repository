

import {useCallback, useEffect, useMemo, useState} from 'react';
import {ExtJS, useForm} from '@globalart/astrum-ui-plugin';
import {useRouter} from '@uirouter/react';
import {createLoginFormMachine} from './loginFormMachine';

export default function useLoginPage() {
  const router = useRouter();

  const machine = useMemo(() => createLoginFormMachine(), []);
  const form = useForm(machine);

  const edition = ExtJS.useState(() => ExtJS.state().getEdition?.()) || 'PRO';
  const samlEnabled = !!ExtJS.useState(() => ExtJS.state().getValue('samlEnabled', false));
  const oauth2Enabled = !!ExtJS.useState(() => ExtJS.state().getValue('oauth2Enabled', false));
  const contextPath = ExtJS.useState(() => ExtJS.state().getValue('astrum-context-path', '')) || '';
  const showSsoLogin = samlEnabled || oauth2Enabled;
  const showAnonymousAccess = !!ExtJS.useState(() => ExtJS.state().getValue('anonymousUsername', false));

  // Initial admin password file path info
  const adminPasswordFilePath = ExtJS.useState(() => ExtJS.state().getValue('admin.password.file', null));
  const onboardingRequired = !!ExtJS.useState(() => ExtJS.state().getValue('onboarding.required', false));
  const isCloudEnvironment = !!ExtJS.useState(() => ExtJS.state().getValue('isCloud', false));

  const contextPrefix = contextPath === '/' ? '' : contextPath;

  const handleSsoLogin = useCallback(() => {
    const returnTo = router.globals.params.returnTo;
    const hash = returnTo ? `hash=${encodeURIComponent(returnTo)}` : '';
    let basePath = '';
    if (samlEnabled) {
      basePath = `${contextPrefix}/saml`;
    } else if (oauth2Enabled) {
      basePath = `${contextPrefix}/oidc/login`;
    }
    form.send({type: 'SSO_LOGIN', url: `${basePath}?${hash}`});
  }, [form.send, router, samlEnabled, oauth2Enabled, contextPrefix]);

  const isSsoRedirecting = form.state.context.isSsoRedirecting ?? false;
  const rateLimitWarning = form.state.context.rateLimitWarning ?? false;
  const retryAfterSeconds = form.state.context.retryAfterSeconds ?? null;

  const [secondsLeft, setSecondsLeft] = useState(null);

  useEffect(() => {
    if (retryAfterSeconds == null) {
      setSecondsLeft(null);
      return;
    }
    setSecondsLeft(retryAfterSeconds);
    const intervalId = setInterval(() => {
      setSecondsLeft((prev) => {
        if (prev !== null && prev <= 1) {
          clearInterval(intervalId);
          return null;
        }
        return prev !== null ? prev - 1 : null;
      });
    }, 1000);
    return () => clearInterval(intervalId);
  }, [retryAfterSeconds]);

  const isRateLimited = secondsLeft !== null;

  const handleContinueWithoutLogin = useCallback(() => {
    window.location.hash = '#browse/welcome';
  }, []);

  return {
    ...form,
    edition,
    showSsoLogin,
    showAnonymousAccess,
    isSsoRedirecting,
    rateLimitWarning,
    secondsLeft,
    isRateLimited,
    handleSsoLogin,
    handleContinueWithoutLogin,
    adminPasswordFilePath,
    onboardingRequired,
    isCloudEnvironment,
  };
}
