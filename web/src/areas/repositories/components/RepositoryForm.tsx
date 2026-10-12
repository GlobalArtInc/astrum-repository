import { Button, Callout, Heading, Text } from '@radix-ui/themes';
import { Info } from 'lucide-react';
import { type FormEvent, type ReactNode, useEffect, useRef, useState } from 'react';
import { useT } from '../../../i18n';
import {
  FormError,
  formatName,
  NumberField,
  OrderedListField,
  SecretField,
  SelectField,
  StatusBadge,
  SwitchField,
  TextAreaField,
  TextField,
  TypeBadge,
  UnsavedChangesGuard,
  useForm,
} from '../../../ui';
import type { UiRepository } from '../api';
import type { RepositoryDocument } from '../model/document';
import {
  fromDocument,
  hasSection,
  onFormFields,
  type RepositoryValues,
  storedSecrets,
  toDocument,
  validate,
} from '../model/form';
import { type Spec, sections } from '../model/spec';
import styles from '../repositories.module.css';

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
  onSubmit(document: RepositoryDocument): Promise<void>;
  onSaved?(document: RepositoryDocument): void;
  onCancel?(): void;
  onDirtyChange?(dirty: boolean): void;
}

const NONE = '_none';

const WRITE_POLICIES = [
  { value: 'ALLOW', label: 'repositories.form.writePolicies.ALLOW' },
  { value: 'ALLOW_ONCE', label: 'repositories.form.writePolicies.ALLOW_ONCE' },
  { value: 'DENY', label: 'repositories.form.writePolicies.DENY' },
];

const AUTH_TYPES = [
  { value: NONE, label: 'repositories.form.authTypes.none' },
  { value: 'username', label: 'repositories.form.authTypes.username' },
  { value: 'ntlm', label: 'repositories.form.authTypes.ntlm' },
  { value: 'bearerToken', label: 'repositories.form.authTypes.bearerToken' },
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

const INDEX_TYPES = [
  { value: 'REGISTRY', label: 'repositories.form.indexTypes.REGISTRY' },
  { value: 'HUB', label: 'repositories.form.indexTypes.HUB' },
  { value: 'CUSTOM', label: 'repositories.form.indexTypes.CUSTOM' },
];

export function Section({
  title,
  description,
  children,
}: {
  title: string;
  description?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section className={styles.section} aria-label={title}>
      <div className={styles.sectionHead}>
        <Heading as="h2" size="4">
          {title}
        </Heading>
        {description && (
          <Text as="p" size="2" color="gray">
            {description}
          </Text>
        )}
      </div>
      <div className={styles.fields}>{children}</div>
    </section>
  );
}

function Wide({ children }: { children: ReactNode }) {
  return <div className={styles.wide}>{children}</div>;
}

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
  const onSaved = props.onSaved;
  const onDirtyChange = props.onDirtyChange;

  useEffect(() => {
    onDirtyChange?.(form.dirty);
  }, [form.dirty, onDirtyChange]);

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

  const candidates = memberCandidates(props.repositories, spec.format, values.name);
  const byName = new Map(props.repositories.map((repository) => [repository.name.toLowerCase(), repository]));
  const memberOptions = [
    ...candidates.map((repository) => ({ value: repository.name, label: repository.name })),
    ...values.memberNames
      .filter((name) => !candidates.some((repository) => sameName(repository.name, name)))
      .map((name) => ({ value: name, label: name })),
  ];
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

  const selectable = props.blobStores?.filter((store) => store.groups.length === 0) ?? null;
  const blobStoreField =
    creating && selectable ? (
      <SelectField
        {...field('blobStoreName')}
        label={t('repositories.form.blobStore')}
        hint={t('repositories.form.blobStoreHint')}
        required
        options={selectable.map((store) => ({ value: store.name, label: store.name }))}
      />
    ) : (
      <TextField
        {...field('blobStoreName')}
        disabled={readOnly || !creating}
        label={t('repositories.form.blobStore')}
        hint={creating ? t('repositories.form.blobStoreHint') : t('repositories.form.blobStoreFixed')}
        required={creating}
      />
    );

  const authentication = values.authType;
  const formElement = useRef<HTMLFormElement>(null);
  const submit = async (event: FormEvent) => {
    await form.submit(event);
    window.requestAnimationFrame(() => {
      formElement.current?.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus();
    });
  };

  return (
    <form
      ref={formElement}
      className={styles.form}
      onSubmit={(event) => void submit(event)}
      noValidate
      aria-label={props.submitLabel}
    >
      <UnsavedChangesGuard when={form.dirty && !readOnly} />
      {readOnly && (
        <Callout.Root color="gray" variant="surface">
          <Callout.Icon>
            <Info size={16} />
          </Callout.Icon>
          <Callout.Text>{t('repositories.form.readOnly')}</Callout.Text>
        </Callout.Root>
      )}
      {creating && (
        <Section title={t('repositories.form.sections.general')}>
          <TextField
            {...field('name')}
            label={t('repositories.form.name')}
            hint={t('repositories.form.nameHint')}
            required
            autoFocus
            maxLength={200}
          />
          <Wide>
            <SwitchField
              {...field('online')}
              label={t('repositories.form.online')}
              description={t('repositories.form.onlineHint')}
            />
          </Wide>
        </Section>
      )}

      {sections.proxy(spec) && (
        <Section
          title={t('repositories.form.sections.proxy')}
          description={t('repositories.form.sections.proxyText')}
        >
          <Wide>
            <TextField
              {...field('remoteUrl')}
              type="url"
              label={t('repositories.form.remoteUrl')}
              hint={t('repositories.form.remoteUrlHint')}
              required
            />
          </Wide>
          <NumberField
            {...field('contentMaxAge')}
            label={t('repositories.form.contentMaxAge')}
            hint={t('repositories.form.contentMaxAgeHint')}
            min={-1}
            required
          />
          <NumberField
            {...field('metadataMaxAge')}
            label={t('repositories.form.metadataMaxAge')}
            hint={t('repositories.form.metadataMaxAgeHint')}
            min={-1}
            required
          />
        </Section>
      )}

      {sections.group(spec) && (
        <Section
          title={t('repositories.form.sections.group')}
          description={t('repositories.form.sections.groupText')}
        >
          <Wide>
            <OrderedListField
              {...field('memberNames')}
              label={t('repositories.form.members')}
              hint={t('repositories.form.membersHint')}
              options={memberOptions}
              addPlaceholder={t('repositories.form.addMember')}
              empty={t('repositories.form.noMembers')}
              noOptions={t('repositories.form.noCandidates', { format: formatName(spec.format) })}
              describe={(name) => {
                const found = byName.get(name.toLowerCase());
                return found ? (
                  <span className={styles.badges}>
                    <TypeBadge type={found.type} />
                    <StatusBadge tone={found.online ? 'success' : 'neutral'}>
                      {found.online ? t('repositories.status.online') : t('repositories.status.offline')}
                    </StatusBadge>
                  </span>
                ) : (
                  <Text size="1" color="gray">
                    {t('repositories.form.memberHidden')}
                  </Text>
                );
              }}
            />
          </Wide>
          {sections.writableMember(spec) && (
            <SelectField
              {...field('writableMember')}
              value={values.writableMember === '' ? NONE : values.writableMember}
              onChange={(value) => form.set('writableMember', value === NONE ? '' : value)}
              label={t('repositories.form.writableMember')}
              hint={t('repositories.form.writableMemberHint')}
              options={writableOptions}
            />
          )}
        </Section>
      )}

      <Section
        title={t('repositories.form.sections.storage')}
        description={t('repositories.form.sections.storageText')}
      >
        {blobStoreField}
        {sections.writePolicy(spec) && (
          <SelectField
            {...field('writePolicy')}
            label={t('repositories.form.writePolicy')}
            hint={t('repositories.form.writePolicyHint')}
            required
            options={WRITE_POLICIES.map((option) => ({ value: option.value, label: t(option.label) }))}
          />
        )}
        <Wide>
          <SwitchField
            {...field('strictContentTypeValidation')}
            label={t('repositories.form.strict')}
            description={t('repositories.form.strictHint')}
          />
        </Wide>
        {sections.latestPolicy(spec) && (
          <Wide>
            <SwitchField
              {...field('latestPolicy')}
              label={t('repositories.form.latestPolicy')}
              description={t('repositories.form.latestPolicyHint')}
            />
          </Wide>
        )}
      </Section>

      {sections.maven(spec) && (
        <Section title={t('repositories.form.sections.maven')}>
          <SelectField
            {...field('versionPolicy')}
            label={t('repositories.form.versionPolicy')}
            hint={t('repositories.form.versionPolicyHint')}
            required
            options={VERSION_POLICIES.map((option) => ({ value: option.value, label: t(option.label) }))}
          />
          <SelectField
            {...field('layoutPolicy')}
            label={t('repositories.form.layoutPolicy')}
            hint={t('repositories.form.layoutPolicyHint')}
            required
            options={LAYOUT_POLICIES.map((option) => ({ value: option.value, label: t(option.label) }))}
          />
          <SelectField
            {...field('mavenContentDisposition')}
            label={t('repositories.form.contentDisposition')}
            hint={t('repositories.form.contentDispositionHint')}
            options={DISPOSITIONS.map((option) => ({ value: option.value, label: t(option.label) }))}
          />
        </Section>
      )}

      {sections.raw(spec) && (
        <Section title={t('repositories.form.sections.raw')}>
          <SelectField
            {...field('rawContentDisposition')}
            label={t('repositories.form.contentDisposition')}
            hint={t('repositories.form.contentDispositionHint')}
            options={DISPOSITIONS.map((option) => ({ value: option.value, label: t(option.label) }))}
          />
          {sections.rawQuery(spec) && (
            <>
              <Wide>
                <SwitchField
                  {...field('forwardQueryParameters')}
                  label={t('repositories.form.forwardQueryParameters')}
                  description={t('repositories.form.forwardQueryParametersHint')}
                />
              </Wide>
              {values.forwardQueryParameters && (
                <Wide>
                  <TextAreaField
                    {...field('excludedQueryParameters')}
                    label={t('repositories.form.excludedQueryParameters')}
                    hint={t('repositories.form.excludedQueryParametersHint')}
                    rows={3}
                    monospace
                  />
                </Wide>
              )}
            </>
          )}
        </Section>
      )}

      {hasSection(document, 'npm') && (
        <Section title={t('repositories.form.sections.npm')}>
          <Wide>
            <SwitchField {...field('removeNonCataloged')} label={t('repositories.form.removeNonCataloged')} />
          </Wide>
          <Wide>
            <SwitchField {...field('removeQuarantined')} label={t('repositories.form.removeQuarantined')} />
          </Wide>
        </Section>
      )}

      {hasSection(document, 'pypi') && (
        <Section title={t('repositories.form.sections.pypi')}>
          <Wide>
            <SwitchField
              {...field('pypiRemoveQuarantined')}
              label={t('repositories.form.removeQuarantined')}
            />
          </Wide>
        </Section>
      )}

      {sections.docker(spec) && (
        <Section
          title={t('repositories.form.sections.docker')}
          description={t('repositories.form.sections.dockerText')}
        >
          <Wide>
            <SwitchField
              {...field('pathEnabled')}
              label={t('repositories.form.pathEnabled')}
              description={t('repositories.form.pathEnabledHint')}
            />
          </Wide>
          <NumberField
            {...field('httpPort')}
            label={t('repositories.form.httpPort')}
            hint={t('repositories.form.portHint')}
            min={1}
            max={65535}
          />
          <NumberField
            {...field('httpsPort')}
            label={t('repositories.form.httpsPort')}
            hint={t('repositories.form.portHint')}
            min={1}
            max={65535}
          />
          <Wide>
            <TextField
              {...field('subdomain')}
              label={t('repositories.form.subdomain')}
              hint={t('repositories.form.subdomainHint')}
            />
          </Wide>
          <Wide>
            <SwitchField
              {...field('forceBasicAuth')}
              label={t('repositories.form.forceBasicAuth')}
              description={t('repositories.form.forceBasicAuthHint')}
            />
          </Wide>
          <Wide>
            <SwitchField {...field('v1Enabled')} label={t('repositories.form.v1Enabled')} />
          </Wide>
        </Section>
      )}

      {sections.dockerProxy(spec) && (
        <Section title={t('repositories.form.sections.dockerProxy')}>
          <SelectField
            {...field('indexType')}
            label={t('repositories.form.indexType')}
            options={INDEX_TYPES.map((option) => ({ value: option.value, label: t(option.label) }))}
          />
          {values.indexType !== 'REGISTRY' && values.indexType !== '' && (
            <TextField
              {...field('indexUrl')}
              type="url"
              label={t('repositories.form.indexUrl')}
              required={values.indexType === 'CUSTOM'}
            />
          )}
          <Wide>
            <SwitchField
              {...field('cacheForeignLayers')}
              label={t('repositories.form.cacheForeignLayers')}
              description={t('repositories.form.cacheForeignLayersHint')}
            />
          </Wide>
          {values.cacheForeignLayers && (
            <Wide>
              <TextAreaField
                {...field('foreignLayerUrlWhitelist')}
                label={t('repositories.form.foreignLayerUrlWhitelist')}
                hint={t('repositories.form.foreignLayerUrlWhitelistHint')}
                rows={3}
                monospace
              />
            </Wide>
          )}
        </Section>
      )}

      {sections.proxy(spec) && (
        <>
          <Section
            title={t('repositories.form.sections.negativeCache')}
            description={t('repositories.form.sections.negativeCacheText')}
          >
            <Wide>
              <SwitchField
                {...field('negativeCacheEnabled')}
                label={t('repositories.form.negativeCacheEnabled')}
                description={t('repositories.form.negativeCacheEnabledHint')}
              />
            </Wide>
            <NumberField
              {...field('timeToLive')}
              label={t('repositories.form.timeToLive')}
              min={0}
              required
            />
          </Section>
          <Section
            title={t('repositories.form.sections.httpClient')}
            description={t('repositories.form.sections.httpClientText')}
          >
            <Wide>
              <SwitchField
                {...field('blocked')}
                label={t('repositories.form.blocked')}
                description={t('repositories.form.blockedHint')}
              />
            </Wide>
            <Wide>
              <SwitchField
                {...field('autoBlock')}
                label={t('repositories.form.autoBlock')}
                description={t('repositories.form.autoBlockHint')}
              />
            </Wide>
            <NumberField
              {...field('retries')}
              label={t('repositories.form.retries')}
              hint={t('repositories.form.retriesHint')}
              min={0}
              max={10}
            />
            <NumberField
              {...field('timeout')}
              label={t('repositories.form.timeout')}
              hint={t('repositories.form.timeoutHint')}
              min={1}
              max={3600}
            />
            <Wide>
              <TextField
                {...field('userAgentSuffix')}
                label={t('repositories.form.userAgentSuffix')}
                hint={t('repositories.form.userAgentSuffixHint')}
              />
            </Wide>
            <Wide>
              <SwitchField
                {...field('enableCircularRedirects')}
                label={t('repositories.form.enableCircularRedirects')}
              />
            </Wide>
            <Wide>
              <SwitchField {...field('enableCookies')} label={t('repositories.form.enableCookies')} />
            </Wide>
          </Section>
          <Section
            title={t('repositories.form.sections.authentication')}
            description={t('repositories.form.sections.authenticationText')}
          >
            <SelectField
              {...field('authType')}
              value={authentication === '' ? NONE : authentication}
              onChange={(value) => form.set('authType', value === NONE ? '' : value)}
              label={t('repositories.form.authType')}
              options={AUTH_TYPES.map((option) => ({ value: option.value, label: t(option.label) }))}
            />
            {(authentication === 'username' || authentication === 'ntlm') && (
              <>
                <TextField
                  {...field('username')}
                  label={t('repositories.form.username')}
                  autoComplete="off"
                  required
                />
                <SecretField
                  {...field('password')}
                  label={t('repositories.form.password')}
                  stored={stored.password}
                  required
                />
              </>
            )}
            {authentication === 'ntlm' && (
              <>
                <TextField {...field('ntlmHost')} label={t('repositories.form.ntlmHost')} />
                <TextField {...field('ntlmDomain')} label={t('repositories.form.ntlmDomain')} />
              </>
            )}
            {authentication === 'bearerToken' && (
              <SecretField
                {...field('bearerToken')}
                label={t('repositories.form.bearerToken')}
                stored={stored.bearerToken}
                required
              />
            )}
            {authentication !== '' && sections.preemptive(spec) && (
              <Wide>
                <SwitchField
                  {...field('preemptive')}
                  label={t('repositories.form.preemptive')}
                  description={t('repositories.form.preemptiveHint')}
                />
              </Wide>
            )}
          </Section>
        </>
      )}

      {!creating && (
        <Text as="p" size="1" color="gray">
          {t('repositories.form.hiddenSettings')}
        </Text>
      )}
      <FormError message={form.formError} />
      {!readOnly && (
        <div className={styles.footer}>
          {creating ? (
            <Button type="button" variant="soft" color="gray" onClick={props.onCancel}>
              {t('repositories.create.cancel')}
            </Button>
          ) : (
            <Button
              type="button"
              variant="soft"
              color="gray"
              disabled={!form.dirty || form.submitting}
              onClick={() => form.reset()}
            >
              {t('repositories.form.discard')}
            </Button>
          )}
          <Button type="submit" loading={form.submitting} disabled={!creating && !form.dirty}>
            {props.submitLabel}
          </Button>
        </div>
      )}
    </form>
  );
}
