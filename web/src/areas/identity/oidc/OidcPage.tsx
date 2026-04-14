import { Button, Flex, Spinner } from '@radix-ui/themes';
import { useQueryClient } from '@tanstack/react-query';
import { ShieldCheck } from 'lucide-react';
import { type FormEvent, useId, useMemo, useState } from 'react';
import { ApiError, api, fieldName, useApi, useInvalidateApi } from '../../../api';
import { STATE_KEY, useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { describeError, ErrorState, FormError, Page, UnsavedChangesGuard, useToast } from '../../../ui';
import { OIDC, type OidcSettings } from '../api';
import { Area, Check, Input, Password, Section } from '../shared/Fields';
import layout from '../shared/Layout.module.css';
import {
  fromSettings,
  type OidcErrors,
  type OidcValues,
  secureUrl,
  toOidcRequest,
  validateOidc,
} from './model';
import styles from './Oidc.module.css';

function OidcForm({ settings }: { settings: OidcSettings }) {
  const t = useT();
  const toast = useToast();
  const client = useQueryClient();
  const invalidate = useInvalidateApi();
  const canWrite = useCan('astrum:settings:update');
  const redirectId = useId();
  const initial = useMemo(() => fromSettings(settings), [settings]);
  const [baseline, setBaseline] = useState(initial);
  const [values, setValues] = useState(initial);
  const [errors, setErrors] = useState<OidcErrors>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [secretStored, setSecretStored] = useState(settings.clientSecretSet);
  const dirty = JSON.stringify(values) !== JSON.stringify(baseline);
  const readOnly = !canWrite;
  const labels = {
    authorizationCustomParams: t('identity.oidc.authorizationParams'),
    tokenRequestCustomParams: t('identity.oidc.tokenParams'),
    exactMatchClaims: t('identity.oidc.exactClaims'),
  };

  const set = <K extends keyof OidcValues>(name: K, value: OidcValues[K]) => {
    setValues((current) => ({ ...current, [name]: value }));
    setErrors((current) => {
      if (!(name in current)) {
        return current;
      }
      const next = { ...current };
      delete next[name];
      return next;
    });
  };

  const bind = (name: keyof OidcValues) => ({
    name,
    value: String(values[name]),
    onChange: (value: string) => set(name, value as never),
    error: errors[name],
    disabled: readOnly,
  });

  const trustable = [values.issuerUrl, values.idpTokenUrl, values.idpJwksUrl].some(
    (url) => url.trim() !== '' && secureUrl(url),
  );

  const save = async (event: FormEvent) => {
    event.preventDefault();
    const found = validateOidc(t, values, labels);
    setErrors(found);
    if (Object.keys(found).length > 0) {
      return;
    }
    setSaving(true);
    setFormError(null);
    try {
      const body = toOidcRequest(values, secretStored);
      await api.put(OIDC, body);
      const stored = body.clientSecret === null ? secretStored : body.clientSecret !== '';
      const next = { ...values, clientSecret: '', removeSecret: false };
      setSecretStored(stored);
      setBaseline(next);
      setValues(next);
      toast.success(t('identity.oidc.saved'));
      await Promise.all([invalidate(OIDC), client.invalidateQueries({ queryKey: STATE_KEY })]);
    } catch (failure) {
      if (failure instanceof ApiError && failure.status === 400 && failure.fieldErrors.length > 0) {
        const found: OidcErrors = {};
        const unmatched: string[] = [];
        for (const problem of failure.fieldErrors) {
          const id = fieldName(problem.id) as keyof OidcValues;
          if (id in values && !found[id]) {
            found[id] = problem.message;
          } else {
            unmatched.push(problem.message);
          }
        }
        setErrors(found);
        setFormError(unmatched.length > 0 ? unmatched.join(' ') : null);
      } else {
        setFormError(describeError(t, failure));
      }
    } finally {
      setSaving(false);
    }
  };

  const discard = () => {
    setValues(baseline);
    setErrors({});
    setFormError(null);
  };

  return (
    <form className={styles.form} onSubmit={save} noValidate>
      {!readOnly && (
        <div className={styles.bar}>
          <Button type="button" variant="outline" onClick={discard}>
            {t('identity.oidc.discard')}
          </Button>
          <Button type="submit" loading={saving} disabled={!dirty}>
            {t('identity.oidc.save')}
          </Button>
        </div>
      )}
      <div className={styles.body}>
        <Section title={t('identity.oidc.settings')} id="oidc-settings">
          <div className={styles.redirect}>
            <span className={styles.redirectLabel} id={redirectId}>
              {t('identity.oidc.redirectUri')}
            </span>
            <output className={styles.value} aria-labelledby={redirectId}>
              {settings.redirectUri}
            </output>
            <p className={styles.redirectHelp}>{t('identity.oidc.redirectUriHelp')}</p>
          </div>
          <Input
            label={t('identity.oidc.issuerUrl')}
            help={t('identity.oidc.issuerUrlHelp')}
            {...bind('issuerUrl')}
          />
          <Input
            label={t('identity.oidc.clientId')}
            help={t('identity.oidc.clientIdHelp')}
            required
            {...bind('clientId')}
          />
          <Password
            label={t('identity.oidc.clientSecret')}
            help={
              secretStored
                ? `${t('identity.oidc.clientSecretHelp')} ${t('identity.oidc.clientSecretStored')}`
                : t('identity.oidc.clientSecretHelp')
            }
            placeholder={secretStored && !values.removeSecret ? '••••••••' : undefined}
            {...bind('clientSecret')}
            disabled={readOnly || values.removeSecret}
          />
          {secretStored && !readOnly && (
            <Check
              label={t('identity.oidc.removeSecret')}
              name="removeSecret"
              checked={values.removeSecret}
              onChange={(checked) => {
                set('removeSecret', checked);
                if (checked) {
                  set('clientSecret', '');
                }
              }}
            />
          )}
          <Input
            label={t('identity.oidc.authorizationUrl')}
            help={t('identity.oidc.authorizationUrlHelp')}
            {...bind('idpAuthorizationUrl')}
          />
          <Input
            label={t('identity.oidc.logoutUrl')}
            help={t('identity.oidc.logoutUrlHelp')}
            {...bind('idpLogoutUrl')}
          />
          <Input
            label={t('identity.oidc.tokenUrl')}
            help={t('identity.oidc.tokenUrlHelp')}
            {...bind('idpTokenUrl')}
          />
          <Input
            label={t('identity.oidc.jwksUrl')}
            help={t('identity.oidc.jwksUrlHelp')}
            {...bind('idpJwksUrl')}
          />
          <Check
            label={t('identity.oidc.trustStore')}
            help={trustable ? t('identity.oidc.trustStoreHelp') : t('identity.oidc.trustStoreInsecure')}
            name="useTrustStore"
            checked={values.useTrustStore}
            disabled={readOnly || (!trustable && !values.useTrustStore)}
            onChange={(checked) => set('useTrustStore', checked)}
          >
            <Button
              type="button"
              size="1"
              variant="outline"
              color="gray"
              className={styles.certificate}
              disabled
            >
              <ShieldCheck size={16} aria-hidden="true" />
              {t('identity.oidc.viewCertificate')}
            </Button>
          </Check>
        </Section>
        <Section title={t('identity.oidc.claimMappings')} id="oidc-claims">
          <Input
            label={t('identity.oidc.usernameClaim')}
            help={t('identity.oidc.usernameClaimHelp')}
            {...bind('usernameClaim')}
          />
          <Input
            label={t('identity.oidc.firstNameClaim')}
            help={t('identity.oidc.firstNameClaimHelp')}
            {...bind('firstNameClaim')}
          />
          <Input
            label={t('identity.oidc.lastNameClaim')}
            help={t('identity.oidc.lastNameClaimHelp')}
            {...bind('lastNameClaim')}
          />
          <Input
            label={t('identity.oidc.emailClaim')}
            help={t('identity.oidc.emailClaimHelp')}
            {...bind('emailClaim')}
          />
          <Input
            label={t('identity.oidc.groupsClaim')}
            help={t('identity.oidc.groupsClaimHelp')}
            {...bind('groupsClaim')}
          />
        </Section>
        <Section title={t('identity.oidc.jwtSettings')} id="oidc-jwt">
          <Input
            label={t('identity.oidc.jwsAlgorithm')}
            help={t('identity.oidc.jwsAlgorithmHelp')}
            {...bind('idpJwsAlgorithm')}
          />
          <Area
            mono={false}
            label={t('identity.oidc.jwks')}
            help={t('identity.oidc.jwksHelp')}
            rows={4}
            {...bind('idpJwks')}
          />
        </Section>
        <Section title={t('identity.oidc.advanced')} id="oidc-advanced">
          <Area
            mono={false}
            label={t('identity.oidc.authorizationParams')}
            help={t('identity.oidc.authorizationParamsHelp')}
            rows={3}
            {...bind('authorizationCustomParams')}
          />
          <Area
            mono={false}
            label={t('identity.oidc.tokenParams')}
            help={t('identity.oidc.tokenParamsHelp')}
            rows={3}
            {...bind('tokenRequestCustomParams')}
          />
          <Area
            mono={false}
            label={t('identity.oidc.exactClaims')}
            help={t('identity.oidc.exactClaimsHelp')}
            rows={3}
            {...bind('exactMatchClaims')}
          />
        </Section>
        <FormError message={formError} />
      </div>
      <UnsavedChangesGuard when={dirty && !saving && !readOnly} />
    </form>
  );
}

export default function OidcPage() {
  const t = useT();
  const settings = useApi<OidcSettings>(OIDC);
  return (
    <Page
      className={layout.page}
      title={t('identity.oidc.title')}
      description={t('identity.oidc.description')}
      breadcrumbs={[
        { label: t('identity.crumbs.settings'), to: '/settings' },
        { label: t('identity.oidc.title') },
      ]}
    >
      {settings.isPending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : settings.error ? (
        <ErrorState error={settings.error} onRetry={() => settings.refetch()} />
      ) : (
        <OidcForm settings={settings.data} />
      )}
    </Page>
  );
}
