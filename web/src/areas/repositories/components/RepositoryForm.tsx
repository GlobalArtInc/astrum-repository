import { Button, Callout } from '@radix-ui/themes';
import { ChevronDown, ChevronRight, Info, ShieldCheck, TriangleAlert } from 'lucide-react';
import { type FormEvent, useEffect, useRef, useState } from 'react';
import { INTERNAL, useApi, V1 } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { describeError, FormError, UnsavedChangesGuard, useForm, useToast } from '../../../ui';
import { TRUSTSTORE_READ } from '../../outbound/api';
import { remoteServer } from '../../outbound/model';
import { CertificateDialog } from '../../outbound/ssl/CertificateDialog';
import type { UiRepository } from '../api';
import type { RepositoryDocument } from '../model/document';
import {
  domainAddress,
  domainHost,
  fromDocument,
  hasSection,
  onFormFields,
  type RepositoryValues,
  storedSecrets,
  toDocument,
  validate,
} from '../model/form';
import { type Spec, sections } from '../model/spec';
import { checkDomain, DomainStatus, domainWarning, useDomainCheck } from './DomainCheck';
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
import { FooterError } from './SaveError';
import { SuggestPort } from './SuggestPort';

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
  onFailure?(message: string | null): void;
}

const NONE = '_none';

export const CLEANUP_POLICY_NAMES = `${INTERNAL}/ui/cleanup-policies/names`;

interface PolicyName {
  name: string;
  format: string;
}

function useCleanupOptions(format: string, applied: readonly string[], enabled: boolean): string[] {
  const names = useApi<PolicyName[]>(
    enabled ? `${CLEANUP_POLICY_NAMES}?${new URLSearchParams({ format })}` : null,
  );
  const known = (names.data ?? []).map((policy) => policy.name);
  return [...known, ...applied.filter((name) => !known.includes(name))];
}

export const ROUTING_RULES = `${V1}/routing-rules`;

interface RoutingRuleChoice {
  name: string;
  mode: string;
}

function useRoutingOptions(current: string, enabled: boolean): Array<{ value: string; label: string }> {
  const rules = useApi<RoutingRuleChoice[]>(enabled ? ROUTING_RULES : null);
  const known = (rules.data ?? []).map((rule) => ({
    value: rule.name,
    label: `${rule.name} (${rule.mode})`,
  }));
  if (current !== '' && !known.some((option) => option.value === current)) {
    known.unshift({ value: current, label: current });
  }
  return known;
}

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

const DEPLOY_POLICIES = [
  { value: 'STRICT', label: 'repositories.form.deployPolicies.STRICT' },
  { value: 'PERMISSIVE', label: 'repositories.form.deployPolicies.PERMISSIVE' },
];

const NUGET_VERSIONS = [
  { value: 'V2', label: 'repositories.form.nugetVersions.V2' },
  { value: 'V3', label: 'repositories.form.nugetVersions.V3' },
];

const CONAN_VERSIONS = [
  { value: 'V1', label: 'repositories.form.conanVersions.V1' },
  { value: 'V2', label: 'repositories.form.conanVersions.V2' },
];

const DOMAIN_ROUTINGS = [
  { value: 'ROOT', label: 'repositories.form.domainRoutings.ROOT' },
  { value: 'PATH', label: 'repositories.form.domainRoutings.PATH' },
];

const INDEX_TYPES = [
  { value: 'REGISTRY', label: 'repositories.form.indexTypes.REGISTRY' },
  { value: 'HUB', label: 'repositories.form.indexTypes.HUB' },
  { value: 'CUSTOM', label: 'repositories.form.indexTypes.CUSTOM' },
];

const REMOTE_EXAMPLES: Record<string, string> = {
  alpine: 'https://dl-cdn.alpinelinux.org/alpine/',
  ansible: 'https://galaxy.ansible.com',
  bazel: 'https://bcr.bazel.build',
  cargo: 'https://index.crates.io',
  chef: 'https://supermarket.chef.io',
  cocoapods: 'https://cdn.cocoapods.org/',
  composer: 'https://repo.packagist.org',
  conda: 'https://conda.anaconda.org/conda-forge/',
  conan: 'https://center2.conan.io',
  apt: 'https://example.com/repository/',
  go: 'https://example.com/repository/',
  maven2: 'https://repo1.maven.org/maven2/',
  nix: 'https://cache.nixos.org',
  npm: 'https://registry.npmjs.org/',
  openvsx: 'https://open-vsx.org',
  p2: 'https://download.eclipse.org/releases/2024-03/',
  pacman: 'https://geo.mirror.pkgbuild.com/',
  pub: 'https://pub.dev',
  puppet: 'https://forgeapi.puppet.com',
  pypi: 'https://pypi.org/',
  r: 'https://cloud.r-project.org/',
  docker: 'https://registry-1.docker.io',
  helm: 'https://charts.helm.sh/stable/',
  hex: 'https://repo.hex.pm',
  huggingface: 'https://huggingface.co/',
  raw: 'https://example.org/files/',
  swift: 'https://packages.example.com/',
  terraform: 'https://registry.terraform.io',
  vagrant: 'https://vagrantcloud.com',
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
  const [viewingCertificate, setViewingCertificate] = useState(false);
  const canViewCertificate = useCan(TRUSTSTORE_READ);
  const [failure, setFailure] = useState<string | null>(null);
  const refused = useRef(false);
  const toast = useToast();
  const { onFailure } = props;
  const form = useForm<RepositoryValues>({
    initial: fromDocument(document),
    validate: (values) => validate(values, { spec, creating, stored }, t),
    onSubmit: async (values) => {
      const next = toDocument(values, document);
      const host = domainHost(values.domainHost);
      const dns =
        host !== '' && host !== domainHost(fromDocument(document).domainHost)
          ? await checkDomain(host)
          : null;
      setFailure(null);
      onFailure?.(null);
      refused.current = false;
      try {
        await props.onSubmit(next);
      } catch (error) {
        refused.current = true;
        const message = describeError(t, error);
        setFailure(message);
        onFailure?.(message);
        toast.error(message);
        throw onFormFields(error, spec);
      }
      setSaved(next);
      const warning = domainWarning(dns, t);
      if (warning) {
        toast.show({ tone: 'info', title: t('repositories.form.domainCheck.saved'), description: warning });
      }
    },
  });
  const footer = !readOnly && !creating;
  const { values } = form;
  const { onSaved, onDirtyChange, onReadyChange } = props;
  const canManagePolicies = useCan('astrum:cleanup-policies:read');
  const cleanupOptions = useCleanupOptions(spec.format, values.cleanupPolicies, !sections.group(spec));
  const routingRules = useRoutingOptions(values.routingRuleName, sections.proxy(spec));
  const domainCheck = useDomainCheck(values.domainHost);
  const address = domainAddress(values, window.location.protocol);
  const shownAddress =
    address && spec.format === 'docker'
      ? `${address.replace(/^[a-z]+:\/\//, '').replace(/\/$/, '')}/<image>`
      : address;

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
  const routingOptions = [{ value: NONE, label: t('repositories.form.routingRuleNone') }, ...routingRules];

  const authentication = values.authType;
  const showAdvanced = advanced || Boolean(form.errors.retries || form.errors.timeout);
  const formElement = useRef<HTMLFormElement>(null);
  const submit = async (event: FormEvent) => {
    await form.submit(event);
    if (refused.current) {
      return;
    }
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
      <UnsavedChangesGuard when={form.dirty && !readOnly} />
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
                  footer={
                    <SuggestPort
                      other={values.httpsPort}
                      disabled={readOnly}
                      onSuggest={(port) => form.set('httpPort', port)}
                    />
                  }
                />
                <NumberInput
                  {...field('httpsPort')}
                  label={t('repositories.form.httpsPort')}
                  hint={t('repositories.form.httpsPortHint')}
                  placeholder={t('repositories.form.httpsPortPlaceholder')}
                  min={1}
                  max={65535}
                  footer={
                    <SuggestPort
                      other={values.httpPort}
                      disabled={readOnly}
                      onSuggest={(port) => form.set('httpsPort', port)}
                    />
                  }
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

        <Section
          title={t('repositories.form.sections.domain')}
          description={t('repositories.form.sections.domainText')}
        >
          <TextInput
            {...field('domainHost')}
            label={t('repositories.form.domainHost')}
            hint={t('repositories.form.domainHostHint')}
            placeholder={spec.format === 'docker' ? 'docker.example.com' : 'packages.example.com'}
            autoComplete="off"
          />
          {values.domainHost.trim() !== '' && !form.errors.domainHost && <DomainStatus check={domainCheck} />}
          {values.domainHost.trim() !== '' && (
            <SelectInput
              name="domainRouting"
              error={form.errors.domainRouting}
              value={values.domainRouting === 'PATH' ? 'PATH' : 'ROOT'}
              onChange={(value) => form.set('domainRouting', value)}
              disabled={readOnly}
              label={t('repositories.form.domainRouting')}
              hint={t('repositories.form.domainRoutingHint')}
              options={options(DOMAIN_ROUTINGS)}
            />
          )}
          {values.domainHost.trim() !== '' && values.domainRouting === 'PATH' && (
            <TextInput
              {...field('domainPath')}
              label={t('repositories.form.domainPath')}
              hint={t('repositories.form.domainPathHint')}
              placeholder={values.name.trim() || 'prod'}
              autoComplete="off"
            />
          )}
          {shownAddress && (
            <p className={styles.hint}>{t('repositories.form.domainAddress', { address: shownAddress })}</p>
          )}
        </Section>

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

        {sections.terraform(spec) && (
          <Section
            title={t('repositories.form.sections.terraform')}
            description={t('repositories.form.sections.terraformText')}
          >
            <TextInput
              {...field('registryHost')}
              label={t('repositories.form.registryHost')}
              hint={t('repositories.form.registryHostHint')}
            />
            {spec.type === 'proxy' && (
              <TextInput
                {...field('upstreamHost')}
                label={t('repositories.form.upstreamHost')}
                hint={t('repositories.form.upstreamHostHint')}
              />
            )}
            <CheckInput
              {...field('requireAuthentication')}
              label={t('repositories.form.requireAuthentication')}
              description={t('repositories.form.requireAuthenticationHint')}
            />
          </Section>
        )}

        {sections.cargo(spec) && (
          <Section
            title={t('repositories.form.sections.cargo')}
            description={t('repositories.form.sections.cargoText')}
          >
            <CheckInput
              {...field('cargoRequireAuthentication')}
              label={t('repositories.form.requireAuthentication')}
              description={t('repositories.form.requireAuthenticationHint')}
            />
          </Section>
        )}

        {sections.nix(spec) && (
          <Section
            title={t('repositories.form.sections.nix')}
            description={t('repositories.form.sections.nixText')}
          >
            <TextInput
              {...field('nixStoreDir')}
              label={t('repositories.form.nixStoreDir')}
              hint={t('repositories.form.nixStoreDirHint')}
            />
            <NumberInput
              {...field('nixPriority')}
              label={t('repositories.form.nixPriority')}
              hint={t('repositories.form.nixPriorityHint')}
              min={0}
              max={1000}
            />
            <CheckInput
              {...field('nixWantMassQuery')}
              label={t('repositories.form.nixWantMassQuery')}
              description={t('repositories.form.nixWantMassQueryHint')}
            />
          </Section>
        )}

        {sections.conanProxy(spec) && (
          <Section
            title={t('repositories.form.sections.conan')}
            description={t('repositories.form.sections.conanText')}
          >
            <SelectInput
              {...field('conanVersion')}
              disabled={readOnly || !creating}
              label={t('repositories.form.conanVersion')}
              hint={t('repositories.form.conanVersionHint')}
              options={options(CONAN_VERSIONS)}
            />
          </Section>
        )}

        {sections.apt(spec) && (
          <Section
            title={t('repositories.form.sections.apt')}
            description={t('repositories.form.sections.aptText')}
          >
            {spec.type === 'proxy' && (
              <CheckInput
                {...field('aptEnforceDistribution')}
                label={t('repositories.form.aptEnforceDistribution')}
                description={t('repositories.form.aptEnforceDistributionHint')}
              />
            )}
            <TextInput
              {...field('aptDistribution')}
              label={t('repositories.form.aptDistribution')}
              hint={t('repositories.form.aptDistributionHint')}
              required={spec.type === 'hosted'}
            />
            {spec.type === 'proxy' && (
              <CheckInput
                {...field('aptFlat')}
                label={t('repositories.form.aptFlat')}
                description={t('repositories.form.aptFlatHint')}
              />
            )}
          </Section>
        )}

        {sections.apt(spec) && (
          <Section
            title={t('repositories.form.sections.aptSigning')}
            description={t('repositories.form.sections.aptSigningText')}
          >
            <TextAreaInput
              {...field('aptKeypair')}
              label={t('repositories.form.aptKeypair')}
              hint={
                !creating && hasSection(document, 'aptSigning')
                  ? t('repositories.form.aptKeypairStored')
                  : t('repositories.form.aptKeypairHint')
              }
              required={spec.type === 'hosted' && creating}
            />
            <SecretInput
              {...field('aptPassphrase')}
              label={t('repositories.form.aptPassphrase')}
              hint={t('repositories.form.aptPassphraseHint')}
            />
          </Section>
        )}

        {sections.pacmanSigning(spec) && (
          <Section
            title={t('repositories.form.sections.pacmanSigning')}
            description={t(
              spec.type === 'group'
                ? 'repositories.form.sections.pacmanSigningGroupText'
                : 'repositories.form.sections.pacmanSigningText',
              { key: `${values.name.trim() || '<name>'}.key` },
            )}
          >
            <TextAreaInput
              {...field('pacmanKeypair')}
              label={t('repositories.form.pacmanKeypair')}
              hint={
                !creating && hasSection(document, 'pacmanSigning')
                  ? t('repositories.form.pacmanKeypairStored')
                  : t('repositories.form.pacmanKeypairHint')
              }
            />
            <SecretInput
              {...field('pacmanPassphrase')}
              label={t('repositories.form.pacmanPassphrase')}
              hint={t('repositories.form.pacmanPassphraseHint')}
            />
          </Section>
        )}
        {sections.alpineSigning(spec) && (
          <Section
            title={t('repositories.form.sections.alpineSigning')}
            description={t(
              spec.type === 'group'
                ? 'repositories.form.sections.alpineSigningGroupText'
                : 'repositories.form.sections.alpineSigningText',
              { key: `${values.name.trim() || '<name>'}.rsa.pub` },
            )}
          >
            <TextAreaInput
              {...field('alpineKeypair')}
              label={t('repositories.form.alpineKeypair')}
              hint={
                !creating && hasSection(document, 'alpineSigning')
                  ? t('repositories.form.alpineKeypairStored')
                  : t('repositories.form.alpineKeypairHint')
              }
            />
            <SecretInput
              {...field('alpinePassphrase')}
              label={t('repositories.form.alpinePassphrase')}
              hint={t('repositories.form.alpinePassphraseHint')}
            />
          </Section>
        )}

        {sections.hexSigning(spec) && (
          <Section
            title={t('repositories.form.sections.hexSigning')}
            description={t('repositories.form.sections.hexSigningText')}
          >
            <TextAreaInput
              {...field('hexKeypair')}
              label={t('repositories.form.hexKeypair')}
              hint={
                !creating && hasSection(document, 'hexSigning')
                  ? t('repositories.form.hexKeypairStored')
                  : t('repositories.form.hexKeypairHint')
              }
            />
          </Section>
        )}

        {sections.yum(spec) && (
          <Section
            title={t('repositories.form.sections.yum')}
            description={t('repositories.form.sections.yumText')}
          >
            <NumberInput
              {...field('repodataDepth')}
              label={t('repositories.form.repodataDepth')}
              hint={t('repositories.form.repodataDepthHint')}
              min={0}
              max={5}
              required
            />
            <SelectInput
              {...field('deployPolicy')}
              label={t('repositories.form.deployPolicy')}
              hint={t('repositories.form.deployPolicyHint')}
              options={options(DEPLOY_POLICIES)}
            />
          </Section>
        )}

        {sections.terraformSigning(spec) && (
          <Section
            title={t('repositories.form.sections.terraformSigning')}
            description={t('repositories.form.sections.terraformSigningText')}
          >
            <TextAreaInput
              {...field('signingKeypair')}
              label={t('repositories.form.signingKeypair')}
              hint={t('repositories.form.signingKeypairHint')}
            />
            <SecretInput
              {...field('signingPassphrase')}
              label={t('repositories.form.signingPassphrase')}
              hint={
                creating
                  ? t('repositories.form.signingPassphraseHint')
                  : t('repositories.form.signingPassphraseEditHint')
              }
              stored={false}
            />
          </Section>
        )}

        {sections.yumSigning(spec) && (
          <Section
            title={t('repositories.form.sections.yumSigning')}
            description={t('repositories.form.sections.yumSigningText')}
          >
            <TextAreaInput
              {...field('yumKeypair')}
              label={t('repositories.form.yumKeypair')}
              hint={
                !creating && hasSection(document, 'yumSigning')
                  ? t('repositories.form.yumKeypairStored')
                  : t('repositories.form.yumKeypairHint')
              }
            />
            <SecretInput
              {...field('yumPassphrase')}
              label={t('repositories.form.yumPassphrase')}
              hint={t('repositories.form.yumPassphraseHint')}
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
              <div>
                <CheckInput
                  {...field('useTrustStore')}
                  label={t('repositories.form.useTrustStore')}
                  description={t('repositories.form.useTrustStoreHint')}
                />
                {canViewCertificate && (
                  <div className={styles.certificateRow}>
                    <button
                      type="button"
                      className={styles.certificateButton}
                      disabled={remoteServer(values.remoteUrl) === null}
                      onClick={() => setViewingCertificate(true)}
                    >
                      <ShieldCheck size={16} aria-hidden="true" />
                      {t('repositories.form.viewCertificate')}
                    </button>
                  </div>
                )}
              </div>
            )}
            {viewingCertificate && remoteServer(values.remoteUrl) && (
              <CertificateDialog
                open={viewingCertificate}
                onOpenChange={setViewingCertificate}
                host={remoteServer(values.remoteUrl)?.host ?? ''}
                port={remoteServer(values.remoteUrl)?.port ?? 443}
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
            <CleanupPolicies
              options={cleanupOptions}
              value={values.cleanupPolicies}
              onChange={(value) => form.set('cleanupPolicies', value)}
              disabled={readOnly}
              error={form.errors.cleanupPolicies}
              manage={canManagePolicies ? '/admin/repository/cleanup-policies' : undefined}
            />
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

        {!footer && !onFailure && <FormError message={form.formError} />}
      </div>
      {footer && (
        <div className={styles.footer}>
          <FooterError message={failure} />
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
