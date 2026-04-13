import { Badge, Button, Callout, Flex, Spinner } from '@radix-ui/themes';
import { ArrowLeft, CircleAlert, CircleCheck, KeyRound, Play, Trash2 } from 'lucide-react';
import { type FormEvent, useEffect, useMemo, useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { ApiError, api, fieldName, path, useApi, useInvalidateApi, withQuery } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import {
  ConfirmDialog,
  describeError,
  ErrorState,
  FormError,
  Page,
  UnsavedChangesGuard,
  useToast,
} from '../../../ui';
import {
  LDAP,
  LDAP_TEMPLATES,
  LDAP_VERIFY_CONNECTION,
  LDAP_VERIFY_MAPPING,
  type LdapServer,
  type LdapTemplate,
  type LdapUser,
} from '../api';
import { failureText } from '../shared/errors';
import { Check, Choice, Input, Password, Section } from '../shared/Fields';
import layout from '../shared/Layout.module.css';
import { PasswordDialog, TestLoginDialog } from './Dialogs';
import { LDAP_PAGE } from './links';
import {
  AUTH_SCHEMES,
  applyTemplate,
  DEFAULT_PORTS,
  EMPTY_LDAP,
  fromServer,
  type LdapErrors,
  type LdapValues,
  needsCredentials,
  type ProtocolChoice,
  sameBindTarget,
  secure,
  serverUrl,
  toRequest,
  usesRealm,
  validateConnection,
  validateMapping,
} from './model';
import styles from './Wizard.module.css';

const CONNECTION_FIELDS: Array<keyof LdapValues> = [
  'name',
  'protocol',
  'useTrustStore',
  'host',
  'port',
  'searchBase',
  'authScheme',
  'authRealm',
  'authUsername',
  'authPassword',
  'connectionTimeoutSeconds',
  'connectionRetryDelaySeconds',
  'maxIncidentsCount',
];

const SERVER_FIELDS: Record<string, keyof LdapValues> = {
  useStartTls: 'protocol',
};

type Verified<T> = { key: string; ok: true; value: T } | { key: string; ok: false; message: string };
type Asking = { purpose: 'connection' | 'mapping' } | null;

function same(left: LdapValues, right: LdapValues): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

function Wizard({ server, names }: { server?: LdapServer; names: string[] }) {
  const t = useT();
  const toast = useToast();
  const navigate = useNavigate();
  const invalidate = useInvalidateApi();
  const editing = server !== undefined;
  const canWrite = useCan(editing ? 'astrum:ldap:update' : 'astrum:ldap:create');
  const canDelete = useCan('astrum:ldap:delete');
  const initial = useMemo(() => (server ? fromServer(server) : EMPTY_LDAP), [server]);
  const [values, setValues] = useState<LdapValues>(initial);
  const [errors, setErrors] = useState<LdapErrors>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [step, setStep] = useState<1 | 2>(1);
  const [saving, setSaving] = useState(false);
  const [leaving, setLeaving] = useState<string | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [asking, setAsking] = useState<Asking>(null);
  const [testing, setTesting] = useState(false);
  const [verifying, setVerifying] = useState<'connection' | 'mapping' | null>(null);
  const [connection, setConnection] = useState<Verified<null> | null>(null);
  const [mapping, setMapping] = useState<Verified<LdapUser[]> | null>(null);
  const [template, setTemplate] = useState('');
  const templates = useApi<LdapTemplate[]>(step === 2 ? LDAP_TEMPLATES : null);
  const otherNames = names.filter((name) => name !== server?.name);
  const passwordStored = editing && server !== undefined && sameBindTarget(values, server);
  const request = toRequest(values, server);
  const requestKey = JSON.stringify(request);
  const existingName = server?.name;
  const dirty = !same(values, initial);

  useEffect(() => {
    if (leaving !== null) {
      navigate(leaving);
    }
  }, [leaving, navigate]);

  const set = <K extends keyof LdapValues>(name: K, value: LdapValues[K]) => {
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

  const bind = <K extends keyof LdapValues>(name: K) => ({
    name,
    value: values[name] as LdapValues[K] & string,
    onChange: (value: string) => set(name, value as LdapValues[K]),
    error: errors[name],
  });

  const connectionProblems = validateConnection(t, values, { names: otherNames, passwordStored });
  const connectionValid = Object.keys(connectionProblems).length === 0;
  const mappingValid = Object.keys(validateMapping(t, values)).length === 0;

  const changeProtocol = (protocol: ProtocolChoice) => {
    setValues((current) => ({
      ...current,
      protocol,
      port: current.port === DEFAULT_PORTS[current.protocol] ? DEFAULT_PORTS[protocol] : current.port,
      useTrustStore: secure(protocol) ? current.useTrustStore : false,
    }));
  };

  const scrollTop = () => document.scrollingElement?.scrollTo?.({ top: 0 });

  const next = (event?: FormEvent) => {
    event?.preventDefault();
    const found = validateConnection(t, values, { names: otherNames, passwordStored });
    setErrors(found);
    if (Object.keys(found).length === 0) {
      setFormError(null);
      setStep(2);
      scrollTop();
    }
  };

  const back = () => {
    setStep(1);
    setFormError(null);
    scrollTop();
  };

  const placeErrors = (failure: ApiError) => {
    const found: LdapErrors = {};
    const unmatched: string[] = [];
    for (const problem of failure.fieldErrors) {
      const id = fieldName(problem.id);
      const field = (SERVER_FIELDS[id] ?? (id in values ? id : null)) as keyof LdapValues | null;
      if (field && !found[field]) {
        found[field] = problem.message;
      } else {
        unmatched.push(problem.message);
      }
    }
    setErrors(found);
    setFormError(unmatched.length > 0 ? unmatched.join(' ') : null);
    if (Object.keys(found).some((field) => CONNECTION_FIELDS.includes(field as keyof LdapValues))) {
      setStep(1);
    }
  };

  const save = async (event?: FormEvent) => {
    event?.preventDefault();
    const found = {
      ...validateConnection(t, values, { names: otherNames, passwordStored }),
      ...validateMapping(t, values),
    };
    setErrors(found);
    if (Object.keys(found).length > 0) {
      if (Object.keys(found).some((field) => CONNECTION_FIELDS.includes(field as keyof LdapValues))) {
        setStep(1);
      }
      return;
    }
    setSaving(true);
    setFormError(null);
    try {
      if (server) {
        await api.put(`${LDAP}${path`/${server.name}`}`, request);
        toast.success(t('identity.ldap.updated', { name: request.name }));
      } else {
        await api.post(LDAP, request);
        toast.success(t('identity.ldap.created', { name: request.name }));
      }
      await invalidate(LDAP);
      setLeaving(LDAP_PAGE);
    } catch (failure) {
      if (failure instanceof ApiError && failure.status === 400 && failure.fieldErrors.length > 0) {
        placeErrors(failure);
      } else {
        setFormError(describeError(t, failure));
      }
    } finally {
      setSaving(false);
    }
  };

  const remove = async () => {
    if (!server) {
      return;
    }
    await api.delete(`${LDAP}${path`/${server.name}`}`);
    toast.success(t('identity.ldap.deleted', { name: server.name }));
    await invalidate(LDAP);
    setLeaving(LDAP_PAGE);
  };

  const needsPassword = () => editing && needsCredentials(values.authScheme) && values.authPassword === '';

  const runConnection = async (body: object, key: string) => {
    setVerifying('connection');
    try {
      await api.post(withQuery(LDAP_VERIFY_CONNECTION, { existingServerName: existingName }), body);
      setConnection({ key, ok: true, value: null });
    } catch (failure) {
      setConnection({ key, ok: false, message: failureText(t, failure) });
    } finally {
      setVerifying(null);
    }
  };

  const runMapping = async (body: object, key: string) => {
    setVerifying('mapping');
    try {
      const users = await api.post<LdapUser[]>(
        withQuery(LDAP_VERIFY_MAPPING, { existingServerName: existingName }),
        body,
      );
      setMapping({ key, ok: true, value: users ?? [] });
    } catch (failure) {
      setMapping({ key, ok: false, message: failureText(t, failure) });
    } finally {
      setVerifying(null);
    }
  };

  const verify = (purpose: 'connection' | 'mapping') => {
    if (needsPassword()) {
      setAsking({ purpose });
      return;
    }
    if (purpose === 'connection') {
      void runConnection(request, requestKey);
    } else {
      void runMapping(request, requestKey);
    }
  };

  const verifyWith = (password: string) => {
    const purpose = asking?.purpose;
    setAsking(null);
    const filled = { ...values, authPassword: password };
    setValues(filled);
    const body = toRequest(filled, server);
    const key = JSON.stringify(body);
    if (purpose === 'connection') {
      void runConnection(body, key);
    } else if (purpose === 'mapping') {
      void runMapping(body, key);
    }
  };

  const shownConnection = connection && connection.key === requestKey ? connection : null;
  const shownMapping = mapping && mapping.key === requestKey ? mapping : null;
  const credentials = needsCredentials(values.authScheme);
  const readOnly = !canWrite;

  const actions = (
    <div className={styles.bar}>
      <div className={styles.buttons}>
        <div className={styles.buttonGroup}>
          <Button type="button" variant="outline" onClick={() => setLeaving(LDAP_PAGE)}>
            {t('identity.ldap.cancel')}
          </Button>
          {editing && canDelete && (
            <Button
              type="button"
              size="3"
              color="red"
              className={styles.deleteButton}
              onClick={() => setDeleting(true)}
            >
              <Trash2 size={16} aria-hidden="true" />
              {t('identity.ldap.delete')}
            </Button>
          )}
        </div>
        <div className={styles.buttonGroup}>
          {step === 2 && (
            <Button type="button" variant="soft" onClick={back}>
              <ArrowLeft size={16} aria-hidden="true" />
              {t('identity.ldap.back')}
            </Button>
          )}
          {step === 1 ? (
            <Button type="submit" form="ldap-wizard" disabled={!connectionValid}>
              {t('identity.ldap.continue')}
            </Button>
          ) : (
            !readOnly && (
              <Button
                type="submit"
                form="ldap-wizard"
                loading={saving}
                disabled={!mappingValid || !connectionValid}
              >
                {editing ? t('identity.ldap.save') : t('identity.ldap.createSubmit')}
              </Button>
            )
          )}
        </div>
      </div>
      <div
        className={styles.progress}
        role="progressbar"
        aria-label={t('identity.ldap.progress', { step, total: 2 })}
        aria-valuemin={1}
        aria-valuemax={2}
        aria-valuenow={step}
      >
        <span className={styles.segment} data-done="true" />
        <span className={styles.segment} data-done={step === 2 || undefined} />
      </div>
    </div>
  );

  const stepOne = (
    <>
      <h2 className={styles.stepTitle}>{t('identity.ldap.connection')}</h2>
      <Section title={t('identity.ldap.serverConfiguration')} id="ldap-server">
        <Input
          label={t('identity.ldap.nameLabel')}
          help={t('identity.ldap.nameHelp')}
          required
          disabled={readOnly}
          {...bind('name')}
        />
        <div className={styles.endpoint}>
          <Choice<ProtocolChoice>
            label={t('identity.ldap.protocol')}
            help={t('identity.ldap.protocolHelp')}
            required
            disabled={readOnly}
            name="protocol"
            value={values.protocol}
            error={errors.protocol}
            onChange={changeProtocol}
            options={[
              { value: 'ldap', label: t('identity.ldap.protocolLdap') },
              { value: 'ldaps', label: t('identity.ldap.protocolLdaps') },
              { value: 'starttls', label: t('identity.ldap.protocolStartTls') },
            ]}
          />
          <Input
            label={t('identity.ldap.host')}
            help={t('identity.ldap.hostHelp')}
            required
            disabled={readOnly}
            placeholder="ldap.example.com"
            {...bind('host')}
          />
          <Input
            label={t('identity.ldap.port')}
            help={t('identity.ldap.portHelp')}
            required
            disabled={readOnly}
            inputMode="numeric"
            {...bind('port')}
          />
        </div>
        {secure(values.protocol) && (
          <Check
            label={t('identity.ldap.trustStore')}
            help={t('identity.ldap.trustStoreHelp')}
            name="useTrustStore"
            checked={values.useTrustStore}
            disabled={readOnly}
            onChange={(checked) => set('useTrustStore', checked)}
          />
        )}
        <Input
          label={t('identity.ldap.searchBase')}
          help={t('identity.ldap.searchBaseHelp')}
          required
          disabled={readOnly}
          placeholder="dc=example,dc=com"
          {...bind('searchBase')}
        />
      </Section>
      <Section title={t('identity.ldap.authentication')} id="ldap-authentication">
        <Choice
          label={t('identity.ldap.authScheme')}
          required
          disabled={readOnly}
          name="authScheme"
          value={values.authScheme}
          error={errors.authScheme}
          onChange={(value) => set('authScheme', value)}
          options={AUTH_SCHEMES.map((scheme) => ({
            value: scheme,
            label: t(
              scheme === 'NONE'
                ? 'identity.ldap.schemeNone'
                : scheme === 'SIMPLE'
                  ? 'identity.ldap.schemeSimple'
                  : scheme === 'DIGEST_MD5'
                    ? 'identity.ldap.schemeDigest'
                    : 'identity.ldap.schemeCram',
            ),
          }))}
        />
        {usesRealm(values.authScheme) && (
          <Input
            label={t('identity.ldap.realm')}
            help={t('identity.ldap.realmHelp')}
            disabled={readOnly}
            {...bind('authRealm')}
          />
        )}
        {credentials && (
          <>
            <Input
              label={t('identity.ldap.username')}
              help={t('identity.ldap.usernameHelp')}
              required
              disabled={readOnly}
              placeholder="cn=admin,dc=example,dc=com"
              {...bind('authUsername')}
            />
            <Password
              label={t('identity.ldap.password')}
              help={editing ? t('identity.ldap.passwordKeep') : t('identity.ldap.passwordHelp')}
              required={!editing}
              disabled={readOnly}
              {...bind('authPassword')}
            />
          </>
        )}
      </Section>
      <Section title={t('identity.ldap.connectionSettings')} id="ldap-connection">
        <div className={styles.three}>
          <Input
            label={t('identity.ldap.timeout')}
            help={t('identity.ldap.timeoutHelp')}
            disabled={readOnly}
            inputMode="numeric"
            {...bind('connectionTimeoutSeconds')}
          />
          <Input
            label={t('identity.ldap.retry')}
            help={t('identity.ldap.retryHelp')}
            disabled={readOnly}
            inputMode="numeric"
            {...bind('connectionRetryDelaySeconds')}
          />
          <Input
            label={t('identity.ldap.incidents')}
            help={t('identity.ldap.incidentsHelp')}
            disabled={readOnly}
            inputMode="numeric"
            {...bind('maxIncidentsCount')}
          />
        </div>
      </Section>
      <div className={styles.verify}>
        <div className={styles.panel}>
          <div className={styles.panelRow}>
            <Button
              type="button"
              size="3"
              variant="outline"
              color="gray"
              className={layout.neutralButton}
              disabled={!connectionValid || verifying !== null}
              loading={verifying === 'connection'}
              onClick={() => verify('connection')}
            >
              <Play size={16} aria-hidden="true" />
              {t('identity.ldap.verifyConnection')}
            </Button>
            {shownConnection?.ok && (
              <span className={styles.ok} role="status">
                <CircleCheck size={16} aria-hidden="true" />
                {t('identity.ldap.connectionOk')}
              </span>
            )}
          </div>
          {shownConnection && !shownConnection.ok && (
            <Callout.Root color="red" role="alert">
              <Callout.Icon>
                <CircleAlert size={16} aria-hidden="true" />
              </Callout.Icon>
              <Flex direction="column" gap="1">
                <Callout.Text weight="medium">{t('identity.ldap.connectionFailed')}</Callout.Text>
                <Callout.Text size="1">{shownConnection.message}</Callout.Text>
              </Flex>
            </Callout.Root>
          )}
        </div>
      </div>
    </>
  );

  const groups = values.ldapGroupsAsRoles;
  const stepTwo = (
    <>
      <h2 className={styles.stepTitle}>{t('identity.ldap.userGroup')}</h2>
      {!readOnly && (
        <div className={styles.template}>
          <Choice
            label={t('identity.ldap.template')}
            help={t('identity.ldap.templateHelp')}
            name="template"
            value={template}
            placeholder={t('identity.ldap.templatePlaceholder')}
            error={templates.error ? t('identity.ldap.templatesFailed') : undefined}
            onChange={(name) => {
              setTemplate(name);
              const chosen = templates.data?.find((candidate) => candidate.name === name);
              if (chosen) {
                setValues((current) => applyTemplate(current, chosen));
                setErrors({});
              }
            }}
            options={(templates.data ?? []).map((candidate) => ({
              value: candidate.name,
              label: candidate.name,
            }))}
          />
        </div>
      )}
      <Section title={t('identity.ldap.userMapping')} id="ldap-users" first>
        <Input
          label={t('identity.ldap.userBaseDn')}
          help={t('identity.ldap.userBaseDnHelp')}
          disabled={readOnly}
          {...bind('userBaseDn')}
        />
        <Check
          label={t('identity.ldap.userSubtree')}
          help={t('identity.ldap.userSubtreeHelp')}
          name="userSubtree"
          checked={values.userSubtree}
          disabled={readOnly}
          onChange={(checked) => set('userSubtree', checked)}
        />
        <Input
          label={t('identity.ldap.objectClass')}
          help={t('identity.ldap.objectClassHelp')}
          required
          disabled={readOnly}
          {...bind('userObjectClass')}
        />
        <Input
          label={t('identity.ldap.filterLabelField')}
          help={t('identity.ldap.filterHelp')}
          disabled={readOnly}
          placeholder="(memberOf=cn=users,dc=example,dc=com)"
          {...bind('userLdapFilter')}
        />
        <div className={styles.two}>
          <Input
            label={t('identity.ldap.userId')}
            help={t('identity.ldap.userIdHelp')}
            required
            disabled={readOnly}
            {...bind('userIdAttribute')}
          />
          <Input
            label={t('identity.ldap.realName')}
            help={t('identity.ldap.realNameHelp')}
            required
            disabled={readOnly}
            {...bind('userRealNameAttribute')}
          />
          <Input
            label={t('identity.ldap.email')}
            required
            disabled={readOnly}
            {...bind('userEmailAddressAttribute')}
          />
          <Input
            label={t('identity.ldap.passwordAttribute')}
            help={t('identity.ldap.passwordAttributeHelp')}
            disabled={readOnly}
            placeholder="userPassword"
            {...bind('userPasswordAttribute')}
          />
        </div>
      </Section>
      <Section title={t('identity.ldap.groupMapping')} id="ldap-groups">
        <Check
          label={t('identity.ldap.groupsAsRoles')}
          help={t('identity.ldap.groupsAsRolesHelp')}
          name="ldapGroupsAsRoles"
          checked={groups}
          disabled={readOnly}
          onChange={(checked) => set('ldapGroupsAsRoles', checked)}
        />
        {groups && (
          <Choice
            label={t('identity.ldap.groupType')}
            required
            disabled={readOnly}
            name="groupType"
            value={values.groupType}
            error={errors.groupType}
            onChange={(value) => set('groupType', value)}
            options={[
              { value: 'DYNAMIC', label: t('identity.ldap.groupDynamic') },
              { value: 'STATIC', label: t('identity.ldap.groupStatic') },
            ]}
          />
        )}
        {groups && values.groupType === 'DYNAMIC' && (
          <Input
            label={t('identity.ldap.memberOf')}
            help={t('identity.ldap.memberOfHelp')}
            required
            disabled={readOnly}
            {...bind('userMemberOfAttribute')}
          />
        )}
        {groups && values.groupType === 'STATIC' && (
          <>
            <Input
              label={t('identity.ldap.groupBaseDn')}
              help={t('identity.ldap.groupBaseDnHelp')}
              disabled={readOnly}
              {...bind('groupBaseDn')}
            />
            <Check
              label={t('identity.ldap.groupSubtree')}
              help={t('identity.ldap.groupSubtreeHelp')}
              name="groupSubtree"
              checked={values.groupSubtree}
              disabled={readOnly}
              onChange={(checked) => set('groupSubtree', checked)}
            />
            <div className={styles.two}>
              <Input
                label={t('identity.ldap.groupObjectClass')}
                required
                disabled={readOnly}
                {...bind('groupObjectClass')}
              />
              <Input
                label={t('identity.ldap.groupId')}
                required
                disabled={readOnly}
                {...bind('groupIdAttribute')}
              />
              <Input
                label={t('identity.ldap.groupMember')}
                required
                disabled={readOnly}
                {...bind('groupMemberAttribute')}
              />
              <Input
                label={t('identity.ldap.groupMemberFormat')}
                help={t('identity.ldap.groupMemberFormatHelp')}
                required
                disabled={readOnly}
                {...bind('groupMemberFormat')}
              />
            </div>
          </>
        )}
      </Section>
      <div className={styles.verify}>
        <div className={styles.panel}>
          <div className={styles.panelRow}>
            <Button
              type="button"
              size="3"
              variant="outline"
              color="gray"
              className={layout.neutralButton}
              disabled={!mappingValid || !connectionValid || verifying !== null}
              loading={verifying === 'mapping'}
              onClick={() => verify('mapping')}
            >
              <Play size={16} aria-hidden="true" />
              {t('identity.ldap.verifyMapping')}
            </Button>
            <Button
              type="button"
              size="3"
              variant="outline"
              color="gray"
              className={layout.neutralButton}
              disabled={!mappingValid || !connectionValid || verifying !== null}
              onClick={() => setTesting(true)}
            >
              <KeyRound size={16} aria-hidden="true" />
              {t('identity.ldap.verifyLogin')}
            </Button>
            {shownMapping?.ok && (
              <Badge color="green" size="2">
                {t('identity.ldap.userCount', { count: shownMapping.value.length })}
              </Badge>
            )}
          </div>
          {shownMapping?.ok && shownMapping.value.length > 0 && (
            <>
              <p className={styles.found}>
                {t('identity.ldap.foundUsers', { count: shownMapping.value.length })}
              </p>
              <div className={styles.usersFrame}>
                <table className={styles.users} aria-label={t('identity.ldap.foundUsersLabel')}>
                  <thead>
                    <tr>
                      <th>{t('identity.ldap.userIdColumn')}</th>
                      <th>{t('identity.ldap.nameColumn')}</th>
                      <th className={styles.hidePhone}>{t('identity.ldap.emailColumn')}</th>
                      <th>{t('identity.ldap.rolesColumn')}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {shownMapping.value.map((user) => (
                      <tr key={user.username}>
                        <td>{user.username}</td>
                        <td>{user.realName ?? ''}</td>
                        <td className={styles.hidePhone}>{user.email ?? ''}</td>
                        <td>{user.membership.join(', ')}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </>
          )}
          {shownMapping && !shownMapping.ok && (
            <Callout.Root color="red" role="alert">
              <Callout.Icon>
                <CircleAlert size={16} aria-hidden="true" />
              </Callout.Icon>
              <Flex direction="column" gap="1">
                <Callout.Text weight="medium">{t('identity.ldap.mappingFailed')}</Callout.Text>
                <Callout.Text size="1">{shownMapping.message}</Callout.Text>
              </Flex>
            </Callout.Root>
          )}
        </div>
      </div>
    </>
  );

  return (
    <div className={styles.wizard} data-step={step}>
      {actions}
      <form id="ldap-wizard" className={styles.body} onSubmit={step === 1 ? next : save} noValidate>
        <div className={styles.steps}>
          {step === 1 ? stepOne : stepTwo}
          <FormError message={formError} />
        </div>
      </form>
      <UnsavedChangesGuard when={dirty && leaving === null && !readOnly} />
      <ConfirmDialog
        open={deleting}
        onOpenChange={setDeleting}
        onConfirm={remove}
        title={t('identity.ldap.deleteTitle')}
        description={t('identity.ldap.deleteText', { name: server?.name ?? '' })}
        confirmLabel={t('identity.ldap.deleteServer')}
        cancelLabel={t('identity.ldap.cancel')}
        danger
      />
      {asking && (
        <PasswordDialog
          text={
            asking.purpose === 'connection'
              ? t('identity.ldap.passwordForConnection')
              : t('identity.ldap.passwordForMapping')
          }
          onVerify={verifyWith}
          onClose={() => setAsking(null)}
        />
      )}
      {testing && (
        <TestLoginDialog
          server={request}
          url={serverUrl({ protocol: request.protocol, host: request.host, port: request.port ?? 0 })}
          existingName={existingName}
          onClose={() => setTesting(false)}
        />
      )}
    </div>
  );
}

export default function LdapServerPage() {
  const t = useT();
  const { name } = useParams();
  const creating = name === undefined;
  const servers = useApi<LdapServer[]>(LDAP);
  const server = useApi<LdapServer>(creating ? null : `${LDAP}${path`/${name}`}`);
  const names = (servers.data ?? []).map((candidate) => candidate.name);
  const breadcrumbs = [
    { label: t('identity.crumbs.settings'), to: '/settings' },
    { label: t('identity.ldap.title'), to: LDAP_PAGE },
    { label: creating ? t('identity.crumbs.create') : (name ?? '') },
  ];
  const pending = servers.isPending || (!creating && server.isPending);
  const failure = !creating && server.error ? server.error : servers.error;
  return (
    <Page
      className={layout.page}
      title={creating ? t('identity.ldap.createTitle') : t('identity.ldap.editTitle', { name: name ?? '' })}
      description={creating ? t('identity.ldap.createText') : t('identity.ldap.editText')}
      breadcrumbs={breadcrumbs}
    >
      {pending ? (
        <Flex justify="center" py="8">
          <Spinner size="3" />
        </Flex>
      ) : failure ? (
        <ErrorState
          error={failure}
          title={
            failure instanceof ApiError && failure.status === 404
              ? t('identity.ldap.notFound', { name: name ?? '' })
              : undefined
          }
        />
      ) : (
        <Wizard key={name ?? 'create'} server={creating ? undefined : server.data} names={names} />
      )}
    </Page>
  );
}
