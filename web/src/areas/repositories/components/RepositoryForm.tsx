import { Button, Callout } from '@radix-ui/themes';
import { ChevronDown, ChevronRight, Info, TriangleAlert } from 'lucide-react';
import { type FormEvent, useEffect, useRef, useState } from 'react';
import { useT } from '../../../i18n';
import { FormError, UnsavedChangesGuard, useForm } from '../../../ui';
import type { UiRepository } from '../api';
import type { RepositoryDocument } from '../model/document';
import {
  cleanupPolicies,
  fromDocument,
  onFormFields,
  type RepositoryValues,
  storedSecrets,
  toDocument,
  validate,
} from '../model/form';
import { type Spec, sections } from '../model/spec';
import { CleanupPolicies, MembersInput } from './FormParts';
import {
  CheckInput,
  NumberInput,
  SecretInput,
  Section,
  SelectInput,
  TextAreaInput,
  TextInput,
} from './fields';
import styles from './fields.module.css';

export interface BlobStoreChoice {
  name: string;
  groups: string[];
}

export interface RepositoryFormProps {
  spec: Spec;
  creating: boolean;
  document: RepositoryDocument;
  readOnly?: boolean;
  repositories: readonly UiRepository[];
  blobStores: readonly BlobStoreChoice[] | null;
  submitLabel: string;
  formId?: string;
  onSubmit(document: RepositoryDocument): Promise<void>;
  onSaved?(document: RepositoryDocument): void;
  onCancel?(): void;
  onDirtyChange?(dirty: boolean): void;
  onReadyChange?(ready: boolean): void;
}

const NONE = '_none';

const WRITE_POLICIES = [
  { value: 'ALLOW_ONCE', label: 'repositories.form.writePolicies.ALLOW_ONCE' },
  { value: 'ALLOW', label: 'repositories.form.writePolicies.ALLOW' },
  { value: 'DENY', label: 'repositories.form.writePolicies.DENY' },
];

const VERSION_POLICIES = [
  { value: 'RELEASE', label: 'repositories.form.versionPolicies.RELEASE' },
  { value: 'SNAPSHOT', label: 'repositories.form.versionPolicies.SNAPSHOT' },
  { value: 'MIXED', label: 'repositories.form.versionPolicies.MIXED' },
];

const LAYOUT_POLICIES = [
  { value: 'STRICT', label: 'repositories.form.layoutPolicies.STRICT' },
  { value: 'PERMISSIVE', label: 'repositories.form.layoutPolicies.PERMISSIVE' },
];

const DISPOSITIONS = [
  { value: 'INLINE', label: 'repositories.form.dispositions.INLINE' },
  { value: 'ATTACHMENT', label: 'repositories.form.dispositions.ATTACHMENT' },
];

const NUGET_VERSIONS = [
  { value: 'V2', label: 'repositories.form.nugetVersions.V2' },
  { value: 'V3', label: 'repositories.form.nugetVersions.V3' },
];

const INDEX_TYPES = [
  { value: 'REGISTRY', label: 'repositories.form.indexTypes.REGISTRY' },
  { value: 'HUB', label: 'repositories.form.indexTypes.HUB' },
  { value: 'CUSTOM', label: 'repositories.form.indexTypes.CUSTOM' },
];

const REMOTE_EXAMPLES: Record<string, string> = {
  maven2: 'https://repo1.maven.org/maven2/',
  npm: 'https://registry.npmjs.org/',
  pypi: 'https://pypi.org/',
  docker: 'https://registry-1.docker.io',
  helm: 'https://charts.helm.sh/stable/',
  raw: 'https://example.org/files/',
};

export function sameName(left: string, right: string): boolean {
  return left.toLowerCase() === right.toLowerCase();
}

export function memberCandidates(
  repositories: readonly UiRepository[],
  format: string,
  self: string,
): UiRepository[] {
  return repositories.filter(
    (repository) => repository.format === format && (self === '' || !sameName(repository.name, self)),
  );
}

export function RepositoryForm(props: RepositoryFormProps) {
  const t = useT();
  const { spec, creating, document } = props;
  const readOnly = props.readOnly === true;
  const stored = creating ? { password: false, bearerToken: false } : storedSecrets(document);
  const [saved, setSaved] = useState<RepositoryDocument | null>(null);
  const [advanced, setAdvanced] = useState(false);
  const form = useForm<RepositoryValues>({
    initial: fromDocument(document),
    validate: (values) => validate(values, { spec, creating, stored }, t),
    onSubmit: async (values) => {
      const next = toDocument(values, document);
      try {
        await props.onSubmit(next);
      } catch (error) {
        throw onFormFields(error, spec);
      }
      setSaved(next);
    },
  });
  const { values } = form;
  const { onSaved, onDirtyChange, onReadyChange } = props;

  useEffect(() => {
    onDirtyChange?.(form.dirty);
  }, [form.dirty, onDirtyChange]);

  const ready = values.name.trim() !== '' && (!creating || values.blobStoreName !== '');
  useEffect(() => {
    onReadyChange?.(ready);
  }, [ready, onReadyChange]);

  useEffect(() => {
    if (saved && !form.dirty) {
      setSaved(null);
      onSaved?.(saved);
    }
  }, [saved, form.dirty, onSaved]);

  const field = <K extends keyof RepositoryValues>(name: K) => ({
    ...form.field(name),
    disabled: readOnly,
  });
  const options = (list: Array<{ value: string; label: string }>) =>
    list.map((option) => ({ value: option.value, label: t(option.label) }));

  const candidates = memberCandidates(props.repositories, spec.format, values.name).map((repository) => ({
    name: repository.name,
    type: repository.type,
  }));
  const byName = new Map(props.repositories.map((repository) => [repository.name.toLowerCase(), repository]));
  const writableOptions = [
    { value: NONE, label: t('repositories.form.writableNone') },
    ...values.memberNames
      .filter((name) => {
        const found = byName.get(name.toLowerCase());
        return !found || found.type === 'hosted' || name === values.writableMember;
      })
      .map((name) => ({ value: name, label: name })),
  ];
  if (values.writableMember !== '' && !values.memberNames.includes(values.writableMember)) {
    writableOptions.push({ value: values.writableMember, label: values.writableMember });
  }

  const selectable = props.blobStores?.filter((store) => store.groups.length === 0) ?? [];
  const blobStoreOptions = selectable.map((store) => ({ value: store.name, label: store.name }));
  if (
    values.blobStoreName !== '' &&
    !blobStoreOptions.some((option) => option.value === values.blobStoreName)
  ) {
    blobStoreOptions.push({ value: values.blobStoreName, label: values.blobStoreName });
  }

  const authTypes = [
    { value: NONE, label: 'repositories.form.authTypes.none' },
    { value: 'username', label: 'repositories.form.authTypes.username' },
    { value: 'ntlm', label: 'repositories.form.authTypes.ntlm' },
    ...(spec.format !== 'maven2' || values.authType === 'bearerToken'
      ? [{ value: 'bearerToken', label: 'repositories.form.authTypes.bearerToken' }]
      : []),
  ];
  const routingOptions = [
    { value: NONE, label: t('repositories.form.routingRuleNone') },
    ...(values.routingRuleName !== ''
      ? [{ value: values.routingRuleName, label: values.routingRuleName }]
      : []),
  ];

  const authentication = values.authType;
  const showAdvanced = advanced || Boolean(form.errors.retries || form.errors.timeout);
  const formElement = useRef<HTMLFormElement>(null);
  const submit = async (event: FormEvent) => {
    await form.submit(event);
    window.requestAnimationFrame(() => {
      formElement.current?.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus();
    });
  };
  const disposition = (key: 'mavenContentDisposition' | 'rawContentDisposition', hint: string) => (
    <>
      <SelectInput
        {...field(key)}
        label={t('repositories.form.contentDisposition')}
        hint={hint}
        options={options(DISPOSITIONS)}
      />
      {values[key] === 'INLINE' && (
        <Callout.Root color="amber" size="1">
          <Callout.Icon>
            <TriangleAlert size={16} />
          </Callout.Icon>
          <Callout.Text>{t('repositories.form.inlineWarning')}</Callout.Text>
        </Callout.Root>
      )}
    </>
  );
  const remoteExample = REMOTE_EXAMPLES[spec.format];

  return (
    <form
      id={props.formId}
      ref={formElement}
      className={styles.form}
      data-width={creating ? 'narrow' : undefined}
      onSubmit={(event) => void submit(event)}
      noValidate
      aria-label={props.submitLabel}
    >
      <UnsavedChangesGuard when={form.dirty && !readOnly && !creating} />
      <div className={styles.content}>
        {readOnly && (
          <Callout.Root color="gray" variant="surface" mb="5">
            <Callout.Icon>
              <Info size={16} />
            </Callout.Icon>
            <Callout.Text>{t('repositories.form.readOnly')}</Callout.Text>
          </Callout.Root>
        )}

        <Section first title={t('repositories.form.sections.general')}>
          <TextInput
            {...field('name')}
            disabled={readOnly || !creating}
            label={t('repositories.form.name')}
            hint={creating ? t('repositories.form.nameHint') : t('repositories.form.nameFixed')}
            required
            autoFocus={creating}
            maxLength={200}
          />
          <CheckInput
            {...field('online')}
            label={t('repositories.form.online')}
            description={t('repositories.form.onlineHint')}
          />
        </Section>

        {sections.docker(spec) && (
          <Section
            title={t('repositories.form.sections.docker')}
            description={t('repositories.form.sections.dockerText')}
          >
            <SelectInput
              name="pathEnabled"
              error={form.errors.pathEnabled}
              value={values.pathEnabled ? 'path' : 'connectors'}
              onChange={(value) => form.set('pathEnabled', value === 'path')}
              disabled={readOnly}
              label={t('repositories.form.routingMode')}
              hint={t('repositories.form.routingModeHint')}
              options={[
                { value: 'path', label: t('repositories.form.routingModes.path') },
                { value: 'connectors', label: t('repositories.form.routingModes.connectors') },
              ]}
            />
            {!values.pathEnabled && (
              <>
                <NumberInput
                  {...field('httpPort')}
                  label={t('repositories.form.httpPort')}
                  hint={t('repositories.form.httpPortHint')}
                  placeholder={t('repositories.form.httpPortPlaceholder')}
                  min={1}
                  max={65535}
                />
                <NumberInput
                  {...field('httpsPort')}
                  label={t('repositories.form.httpsPort')}
                  hint={t('repositories.form.httpsPortHint')}
                  placeholder={t('repositories.form.httpsPortPlaceholder')}
                  min={1}
                  max={65535}
                />
                <TextInput
                  {...field('subdomain')}
                  label={t('repositories.form.subdomain')}
                  hint={t('repositories.form.subdomainHint')}
                  placeholder={t('repositories.form.subdomainPlaceholder')}
                />
              </>
            )}
            <CheckInput
              {...field('forceBasicAuth')}
              label={t('repositories.form.forceBasicAuth')}
              description={t('repositories.form.forceBasicAuthHint')}
            />
            <CheckInput
              {...field('v1Enabled')}
              label={t('repositories.form.v1Enabled')}
              description={t('repositories.form.v1EnabledHint')}
            />
          </Section>
        )}

        {sections.maven(spec) && (
          <Section
            title={t('repositories.form.sections.maven')}
            description={t('repositories.form.sections.mavenText')}
          >
            <SelectInput
              {...field('versionPolicy')}
              label={t('repositories.form.versionPolicy')}
              hint={t('repositories.form.versionPolicyHint')}
              required
              options={options(VERSION_POLICIES)}
            />
            <SelectInput
              {...field('layoutPolicy')}
              label={t('repositories.form.layoutPolicy')}
              hint={t('repositories.form.layoutPolicyHint')}
              options={options(LAYOUT_POLICIES)}
            />
            {disposition('mavenContentDisposition', t('repositories.form.mavenDispositionHint'))}
          </Section>
        )}

        {sections.raw(spec) && (
          <Section
            title={t('repositories.form.sections.raw')}
            description={t('repositories.form.sections.rawText')}
          >
            {disposition('rawContentDisposition', t('repositories.form.rawDispositionHint'))}
          </Section>
        )}

        {sections.nugetProxy(spec) && (
          <Section
            title={t('repositories.form.sections.nuget')}
            description={t('repositories.form.sections.nugetText')}
          >
            <SelectInput
              {...field('nugetVersion')}
              label={t('repositories.form.nugetVersion')}
              options={options(NUGET_VERSIONS)}
            />
            <NumberInput
              {...field('queryCacheItemMaxAge')}
              label={t('repositories.form.queryCacheItemMaxAge')}
              hint={t('repositories.form.queryCacheItemMaxAgeHint')}
              min={0}
            />
          </Section>
        )}

        <Section title={t('repositories.form.sections.storage')}>
          <SelectInput
            {...field('blobStoreName')}
            disabled={readOnly || !creating}
            label={t('repositories.form.blobStore')}
            hint={creating ? t('repositories.form.blobStoreHint') : t('repositories.form.blobStoreFixed')}
            placeholder={t('repositories.form.blobStorePlaceholder')}
            required
            options={blobStoreOptions}
          />
          <CheckInput
            {...field('strictContentTypeValidation')}
            label={t('repositories.form.strict')}
            description={t('repositories.form.strictHint')}
          />
        </Section>

        {sections.writePolicy(spec) && (
          <Section title={t('repositories.form.sections.hosted')}>
            <SelectInput
              {...field('writePolicy')}
              label={t('repositories.form.writePolicy')}
              hint={t('repositories.form.writePolicyHint')}
              options={options(WRITE_POLICIES)}
            />
            {spec.format !== 'docker' && (
              <CheckInput
                {...field('proprietaryComponents')}
                label={t('repositories.form.proprietary')}
                description={t('repositories.form.proprietaryHint')}
              />
            )}
          </Section>
        )}

        {sections.proxy(spec) && (
          <Section title={t('repositories.form.sections.proxy')}>
            <TextInput
              {...field('remoteUrl')}
              type="url"
              label={t('repositories.form.remoteUrl')}
              hint={
                spec.format === 'nuget'
                  ? t('repositories.form.remoteUrlHintNuget')
                  : t('repositories.form.remoteUrlHint', {
                      example: remoteExample
                        ? t('repositories.form.remoteUrlExample', { url: remoteExample })
                        : '',
                    })
              }
              required
            />
            {values.remoteUrl.trim().toLowerCase().startsWith('https://') && (
              <CheckInput
                {...field('useTrustStore')}
                label={t('repositories.form.useTrustStore')}
                description={t('repositories.form.useTrustStoreHint')}
              />
            )}
            <CheckInput
              {...field('preserveEncodedCharacters')}
              label={t('repositories.form.preserveEncoded')}
              description={t('repositories.form.preserveEncodedHint')}
            />
            <div className={styles.group}>
              <span className={styles.subLabel}>{t('repositories.form.blocking')}</span>
              <CheckInput
                {...field('blocked')}
                label={t('repositories.form.blocked')}
                description={t('repositories.form.blockedHint')}
              />
              <CheckInput
                {...field('autoBlock')}
                label={t('repositories.form.autoBlock')}
                description={t('repositories.form.autoBlockHint')}
              />
            </div>
            <NumberInput
              {...field('contentMaxAge')}
              label={t('repositories.form.contentMaxAge')}
              hint={t('repositories.form.contentMaxAgeHint')}
              min={-1}
            />
            <NumberInput
              {...field('metadataMaxAge')}
              label={t('repositories.form.metadataMaxAge')}
              hint={t('repositories.form.metadataMaxAgeHint')}
              min={-1}
            />
          </Section>
        )}

        {sections.dockerProxy(spec) && (
          <Section
            title={t('repositories.form.sections.dockerIndex')}
            description={t('repositories.form.sections.dockerIndexText')}
          >
            <SelectInput
              {...field('indexType')}
              label={t('repositories.form.indexType')}
              hint={t('repositories.form.indexTypeHint')}
              options={options(INDEX_TYPES)}
            />
            {values.indexType === 'CUSTOM' && (
              <TextInput
                {...field('indexUrl')}
                type="url"
                label={t('repositories.form.indexUrl')}
                hint={t('repositories.form.indexUrlHint')}
                required
              />
            )}
            <CheckInput
              {...field('cacheForeignLayers')}
              label={t('repositories.form.cacheForeignLayers')}
              description={t('repositories.form.cacheForeignLayersHint')}
            />
            {values.cacheForeignLayers && (
              <TextAreaInput
                {...field('foreignLayerUrlWhitelist')}
                label={t('repositories.form.foreignLayerUrlWhitelist')}
                hint={t('repositories.form.foreignLayerUrlWhitelistHint')}
              />
            )}
          </Section>
        )}

        {sections.proxy(spec) && (
          <>
            <Section
              title={t('repositories.form.sections.routing')}
              description={t('repositories.form.sections.routingText')}
            >
              <SelectInput
                name="routingRuleName"
                error={form.errors.routingRuleName}
                value={values.routingRuleName === '' ? NONE : values.routingRuleName}
                onChange={(value) => form.set('routingRuleName', value === NONE ? '' : value)}
                disabled={readOnly}
                label={t('repositories.form.routingRule')}
                hint={t('repositories.form.routingRuleHint')}
                options={routingOptions}
              />
            </Section>
            <Section title={t('repositories.form.sections.negativeCache')}>
              <CheckInput
                {...field('negativeCacheEnabled')}
                label={t('repositories.form.negativeCacheEnabled')}
                description={t('repositories.form.negativeCacheEnabledHint')}
              />
              <NumberInput
                {...field('timeToLive')}
                label={t('repositories.form.timeToLive')}
                hint={t('repositories.form.timeToLiveHint')}
                min={0}
              />
            </Section>
          </>
        )}

        {sections.group(spec) && (
          <Section title={t('repositories.form.sections.group')}>
            <MembersInput
              value={values.memberNames}
              onChange={(next) => form.set('memberNames', next)}
              candidates={candidates}
              error={form.errors.memberNames}
              disabled={readOnly}
            />
            {sections.writableMember(spec) && (
              <SelectInput
                name="writableMember"
                error={form.errors.writableMember}
                value={values.writableMember === '' ? NONE : values.writableMember}
                onChange={(value) => form.set('writableMember', value === NONE ? '' : value)}
                disabled={readOnly}
                label={t('repositories.form.writableMember')}
                hint={t('repositories.form.writableMemberHint')}
                options={writableOptions}
              />
            )}
          </Section>
        )}

        {!sections.group(spec) && (
          <Section title={t('repositories.form.sections.cleanup')}>
            <CleanupPolicies applied={cleanupPolicies(document)} />
          </Section>
        )}

        {sections.proxy(spec) && (
          <Section title={t('repositories.form.sections.http')}>
            <SelectInput
              name="authType"
              error={form.errors.authType}
              value={authentication === '' ? NONE : authentication}
              onChange={(value) => form.set('authType', value === NONE ? '' : value)}
              disabled={readOnly}
              label={t('repositories.form.authType')}
              hint={t('repositories.form.authTypeHint')}
              options={options(authTypes)}
            />
            {(authentication === 'username' || authentication === 'ntlm') && (
              <>
                <TextInput
                  {...field('username')}
                  label={t('repositories.form.username')}
                  autoComplete="off"
                  required
                />
                <SecretInput
                  {...field('password')}
                  label={t('repositories.form.password')}
                  stored={stored.password}
                  required
                />
              </>
            )}
            {authentication === 'ntlm' && (
              <>
                <TextInput {...field('ntlmHost')} label={t('repositories.form.ntlmHost')} />
                <TextInput {...field('ntlmDomain')} label={t('repositories.form.ntlmDomain')} />
              </>
            )}
            {authentication === 'bearerToken' && (
              <SecretInput
                {...field('bearerToken')}
                label={t('repositories.form.bearerToken')}
                stored={stored.bearerToken}
                required
              />
            )}
            {authentication !== '' && sections.preemptive(spec) && (
              <CheckInput
                {...field('preemptive')}
                label={t('repositories.form.preemptive')}
                description={t('repositories.form.preemptiveHint')}
              />
            )}
            <button
              type="button"
              className={styles.toggle}
              aria-expanded={showAdvanced}
              onClick={() => setAdvanced(!showAdvanced)}
            >
              {showAdvanced ? (
                <ChevronDown size={16} aria-hidden="true" />
              ) : (
                <ChevronRight size={16} aria-hidden="true" />
              )}
              {t('repositories.form.requestSettings')}
            </button>
            {showAdvanced && (
              <>
                <TextInput
                  {...field('userAgentSuffix')}
                  label={t('repositories.form.userAgentSuffix')}
                  hint={t('repositories.form.userAgentSuffixHint')}
                />
                <NumberInput
                  {...field('retries')}
                  label={t('repositories.form.retries')}
                  hint={t('repositories.form.retriesHint')}
                  min={0}
                  max={10}
                />
                <NumberInput
                  {...field('timeout')}
                  label={t('repositories.form.timeout')}
                  hint={t('repositories.form.timeoutHint')}
                  min={1}
                  max={3600}
                />
                <CheckInput
                  {...field('enableCircularRedirects')}
                  label={t('repositories.form.enableCircularRedirects')}
                  description={t('repositories.form.enableCircularRedirectsHint')}
                />
                <CheckInput
                  {...field('enableCookies')}
                  label={t('repositories.form.enableCookies')}
                  description={t('repositories.form.enableCookiesHint')}
                />
              </>
            )}
          </Section>
        )}

        <FormError message={form.formError} />
      </div>
      {!readOnly && !creating && (
        <div className={styles.footer}>
          {form.dirty && <span className={styles.unsaved}>{t('repositories.form.unsaved')}</span>}
          <Button type="button" variant="outline" onClick={props.onCancel}>
            {t('repositories.form.cancel')}
          </Button>
          <Button type="submit" loading={form.submitting} disabled={!form.dirty}>
            {props.submitLabel}
          </Button>
        </div>
      )}
    </form>
  );
}
