import { Callout } from '@radix-ui/themes';
import { Folder, Info, Network, TriangleAlert } from 'lucide-react';
import { type FormEvent, type ReactNode, useEffect, useRef, useState } from 'react';
import { useT } from '../../../i18n';
import { FormError, UnsavedChangesGuard, useForm } from '../../../ui';
import { MembersInput } from '../../repositories/components/FormParts';
import {
  CheckInput,
  NumberInput,
  SecretInput,
  SelectInput,
  TextAreaInput,
  TextInput,
} from '../../repositories/components/fields';
import type { StoreType, UiBlobStore } from '../api';
import styles from '../blobstores.module.css';
import {
  ENCRYPTION_TYPES,
  FILL_POLICIES,
  fromDocument,
  onFormFields,
  QUOTA_TYPES,
  type StoreDocument,
  type StoreValues,
  sameAddress,
  storedSecret,
  toDocument,
  validate,
} from '../model';
import { typeLabel } from './bits';

const NONE = '_none';

export type Step = 'basic' | 'credentials' | 'advanced';
export const STEPS: readonly Step[] = ['basic', 'credentials', 'advanced'];

const STEP_OF: Partial<Record<keyof StoreValues, Step>> = {
  name: 'basic',
  path: 'basic',
  bucket: 'basic',
  region: 'basic',
  prefix: 'basic',
  expiration: 'basic',
  members: 'basic',
  fillPolicy: 'basic',
  url: 'basic',
  token: 'credentials',
  certificate: 'credentials',
  accessKeyId: 'credentials',
  secretAccessKey: 'credentials',
  role: 'credentials',
  sessionToken: 'credentials',
};

export function stepOf(key: string): Step {
  return STEP_OF[key as keyof StoreValues] ?? 'advanced';
}

const QUOTA_LABELS: Record<(typeof QUOTA_TYPES)[number], string> = {
  spaceUsedQuota: 'blobstores.quota.spaceUsedQuota',
  spaceRemainingQuota: 'blobstores.quota.spaceRemainingQuota',
};

const FILL_LABELS: Record<(typeof FILL_POLICIES)[number], string> = {
  writeToFirst: 'blobstores.group.writeToFirst',
  roundRobin: 'blobstores.group.roundRobin',
  mostFreeSpace: 'blobstores.group.mostFreeSpace',
};

const ENCRYPTION_LABELS: Record<(typeof ENCRYPTION_TYPES)[number], string> = {
  s3ManagedEncryption: 'blobstores.s3.s3ManagedEncryption',
  kmsManagedEncryption: 'blobstores.s3.kmsManagedEncryption',
};

export interface BlobStoreFormProps {
  type: StoreType;
  creating: boolean;
  storeName?: string;
  document: StoreDocument;
  readOnly?: boolean;
  stores: readonly UiBlobStore[];
  submitLabel: string;
  formId?: string;
  step?: Step;
  onStepChange?(step: Step): void;
  onReadyChange?(ready: Record<Step, boolean>): void;
  onDirtyChange?(dirty: boolean): void;
  onSubmit(document: StoreDocument): Promise<void>;
  onSaved?(document: StoreDocument): void;
}

function Section({
  title,
  description,
  icon,
  children,
  hidden,
}: {
  title: string;
  description?: string;
  icon?: ReactNode;
  children: ReactNode;
  hidden?: boolean;
}) {
  return (
    <section className={styles.section} aria-label={title} hidden={hidden}>
      <div className={styles.sectionHead}>
        {icon}
        <span>
          <h2 className={styles.sectionTitle}>{title}</h2>
          {description && <p className={styles.sectionText}>{description}</p>}
        </span>
      </div>
      {children}
    </section>
  );
}

export function memberChoices(stores: readonly UiBlobStore[], self: string): UiBlobStore[] {
  return stores.filter(
    (store) =>
      store.type !== 'Group' &&
      store.name !== self &&
      store.repositoryCount === 0 &&
      (store.groups.length === 0 || store.groups.includes(self)),
  );
}

export function BlobStoreForm(props: BlobStoreFormProps) {
  const t = useT();
  const { type, creating, document } = props;
  const readOnly = props.readOnly === true;
  const storedKeyId = creating ? null : storedSecret(document);
  const storedUrl = creating || typeof document.url !== 'string' ? null : document.url;
  const [saved, setSaved] = useState<StoreDocument | null>(null);
  const form = useForm<StoreValues>({
    initial: fromDocument(document),
    validate: (values) => validate(values, { type, creating, storedKeyId, storedUrl }, t),
    onSubmit: async (values) => {
      const next = toDocument(values, document, type);
      try {
        await props.onSubmit(next);
      } catch (error) {
        throw onFormFields(error);
      }
      setSaved(next);
    },
  });
  const { values } = form;
  const { onSaved, onDirtyChange, onReadyChange, onStepChange } = props;
  useEffect(() => {
    if (saved && !form.dirty) {
      setSaved(null);
      onSaved?.(saved);
    }
  }, [saved, form.dirty, onSaved]);
  useEffect(() => {
    onDirtyChange?.(form.dirty);
  }, [form.dirty, onDirtyChange]);
  const basicReady =
    values.name.trim() !== '' &&
    (type !== 'file' || values.path.trim() !== '') &&
    (type !== 's3' || (values.bucket.trim() !== '' && values.region.trim() !== '')) &&
    (type !== 'group' || values.members.length > 0) &&
    (type !== 'remote' || values.url.trim() !== '');
  useEffect(() => {
    onReadyChange?.({ basic: basicReady, credentials: true, advanced: true });
  }, [basicReady, onReadyChange]);

  const field = <K extends keyof StoreValues>(name: K) => ({ ...form.field(name), disabled: readOnly });
  const self = creating ? values.name.trim() : (props.storeName ?? '');
  const choices = memberChoices(props.stores, self).map((store) => ({
    name: store.name,
    type: typeLabel(t, store.type),
  }));
  const keepsSecret = storedKeyId !== null && values.accessKeyId.trim() === storedKeyId;
  const keepsToken = !creating && sameAddress(values.url, storedUrl);
  const step = props.step;
  const shows = (part: Step) => step === undefined || step === part;

  const formElement = useRef<HTMLFormElement>(null);
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const errors = validate(values, { type, creating, storedKeyId, storedUrl }, t);
    const first = Object.keys(errors)[0];
    if (first && step !== undefined) {
      onStepChange?.(stepOf(first));
    }
    await form.submit();
    window.requestAnimationFrame(() => {
      formElement.current?.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus();
    });
  };

  return (
    <form
      id={props.formId}
      ref={formElement}
      className={styles.form}
      onSubmit={(event) => void submit(event)}
      noValidate
      aria-label={props.submitLabel}
    >
      <UnsavedChangesGuard when={form.dirty && !readOnly && !creating} />
      {readOnly && (
        <Callout.Root color="gray" variant="surface">
          <Callout.Icon>
            <Info size={16} />
          </Callout.Icon>
          <Callout.Text>{t('blobstores.form.readOnly')}</Callout.Text>
        </Callout.Root>
      )}

      {creating && (
        <Section
          title={t('blobstores.form.basic')}
          description={t('blobstores.form.basicText')}
          hidden={!shows('basic')}
        >
          <TextInput
            {...field('name')}
            label={t('blobstores.form.name')}
            hint={t('blobstores.form.nameHint')}
            placeholder="my-blob-store"
            required
            autoFocus
          />
          {type === 'file' && (
            <TextInput
              {...field('path')}
              label={t('blobstores.file.path')}
              hint={t('blobstores.file.pathHint')}
              placeholder="/path/to/blob/storage"
              required
            />
          )}
        </Section>
      )}

      {!creating && type === 'file' && (
        <Section
          title={t('blobstores.file.section')}
          description={t('blobstores.file.sectionText')}
          icon={<Folder size={20} aria-hidden="true" />}
        >
          <Callout.Root color="amber" variant="surface">
            <Callout.Icon>
              <TriangleAlert size={16} />
            </Callout.Icon>
            <Callout.Text>{t('blobstores.file.pathWarning')}</Callout.Text>
          </Callout.Root>
          <TextInput
            {...field('path')}
            label={t('blobstores.file.path')}
            hint={t('blobstores.file.pathHint')}
            required
          />
        </Section>
      )}

      {type === 's3' && (
        <>
          <Section
            title={t('blobstores.s3.bucketSection')}
            description={t('blobstores.s3.bucketText')}
            hidden={!shows('basic')}
          >
            <TextInput
              {...field('bucket')}
              label={t('blobstores.s3.bucket')}
              hint={t('blobstores.s3.bucketHint')}
              required
            />
            <TextInput
              {...field('region')}
              label={t('blobstores.s3.region')}
              hint={t('blobstores.s3.regionHint')}
              required
            />
            <TextInput
              {...field('prefix')}
              label={t('blobstores.s3.prefix')}
              hint={t('blobstores.s3.prefixHint')}
            />
            <NumberInput
              {...field('expiration')}
              label={t('blobstores.s3.expiration')}
              hint={t('blobstores.s3.expirationHint')}
              min={-1}
            />
          </Section>
          <Section
            title={t('blobstores.s3.credentials')}
            description={t('blobstores.s3.credentialsText')}
            hidden={!shows('credentials')}
          >
            <TextInput {...field('accessKeyId')} label={t('blobstores.s3.accessKeyId')} autoComplete="off" />
            <SecretInput
              {...field('secretAccessKey')}
              label={t('blobstores.s3.secretAccessKey')}
              stored={keepsSecret}
              required={values.accessKeyId.trim() !== ''}
            />
            <TextInput
              {...field('role')}
              label={t('blobstores.s3.role')}
              hint={t('blobstores.s3.roleHint')}
              autoComplete="off"
            />
            <SecretInput
              {...field('sessionToken')}
              label={t('blobstores.s3.sessionToken')}
              hint={t('blobstores.s3.sessionTokenHint')}
            />
          </Section>
          <Section title={t('blobstores.s3.encryption')} hidden={!shows('advanced')}>
            <SelectInput
              {...field('encryptionType')}
              value={values.encryptionType === '' ? NONE : values.encryptionType}
              onChange={(value) => form.set('encryptionType', value === NONE ? '' : value)}
              label={t('blobstores.s3.encryptionType')}
              options={[
                { value: NONE, label: t('blobstores.s3.noEncryption') },
                ...ENCRYPTION_TYPES.map((value) => ({ value, label: t(ENCRYPTION_LABELS[value]) })),
              ]}
            />
            {values.encryptionType === 'kmsManagedEncryption' && (
              <TextInput
                {...field('encryptionKey')}
                label={t('blobstores.s3.kmsKey')}
                hint={t('blobstores.s3.kmsKeyHint')}
              />
            )}
          </Section>
          <Section title={t('blobstores.s3.connection')} hidden={!shows('advanced')}>
            <TextInput
              {...field('endpoint')}
              type="url"
              label={t('blobstores.s3.endpoint')}
              hint={t('blobstores.s3.endpointHint')}
            />
            <CheckInput
              {...field('forcePathStyle')}
              label={t('blobstores.s3.pathStyle')}
              description={t('blobstores.s3.pathStyleHint')}
            />
          </Section>
        </>
      )}

      {type === 'remote' && (
        <>
          <Section
            title={t('blobstores.remote.section')}
            description={t('blobstores.remote.sectionText')}
            icon={creating ? undefined : <Network size={20} aria-hidden="true" />}
            hidden={!shows('basic')}
          >
            <TextInput
              {...field('url')}
              type="url"
              label={t('blobstores.remote.url')}
              hint={t('blobstores.remote.urlHint')}
              placeholder="https://storage1.internal:8090"
              required
            />
          </Section>
          <Section
            title={t('blobstores.remote.credentials')}
            description={t('blobstores.remote.credentialsText')}
            hidden={!shows('credentials')}
          >
            <SecretInput
              {...field('token')}
              label={t('blobstores.remote.token')}
              hint={t('blobstores.remote.tokenHint')}
              stored={keepsToken}
              required
            />
            <TextAreaInput
              {...field('certificate')}
              label={t('blobstores.remote.certificate')}
              hint={t('blobstores.remote.certificateHint')}
              rows={4}
            />
          </Section>
        </>
      )}

      {type === 'group' && (
        <Section
          title={t('blobstores.group.section')}
          description={t('blobstores.group.text')}
          hidden={!shows('basic')}
        >
          <MembersInput
            value={values.members}
            onChange={(next) => form.set('members', next)}
            candidates={choices}
            error={form.errors.members}
            disabled={readOnly}
            label={t('blobstores.group.members')}
            hint={t('blobstores.group.membersHint')}
            placeholder={t('blobstores.group.addMember')}
            empty={t('blobstores.group.noMembers')}
          />
          <SelectInput
            {...field('fillPolicy')}
            label={t('blobstores.group.fillPolicy')}
            hint={t('blobstores.group.fillPolicyHint')}
            options={FILL_POLICIES.map((value) => ({ value, label: t(FILL_LABELS[value]) }))}
          />
        </Section>
      )}

      {step === 'credentials' && type !== 's3' && type !== 'remote' && (
        <Section title={t('blobstores.wizard.noCredentials')}>
          <p className={styles.sectionText}>
            {type === 'file'
              ? t('blobstores.wizard.noCredentialsFile')
              : t('blobstores.wizard.noCredentialsGroup')}
          </p>
        </Section>
      )}

      <Section
        title={t('blobstores.quota.title')}
        description={t('blobstores.quota.text')}
        hidden={!shows('advanced')}
      >
        <CheckInput {...field('quotaEnabled')} label={t('blobstores.quota.enable')} />
        {values.quotaEnabled && (
          <>
            <SelectInput
              {...field('quotaType')}
              label={t('blobstores.quota.type')}
              options={QUOTA_TYPES.map((value) => ({ value, label: t(QUOTA_LABELS[value]) }))}
            />
            <NumberInput
              {...field('quotaLimit')}
              label={t('blobstores.quota.limit')}
              hint={
                values.quotaType === 'spaceRemainingQuota'
                  ? t('blobstores.quota.remainingHint')
                  : t('blobstores.quota.usedHint')
              }
              min={1}
              required
            />
          </>
        )}
      </Section>

      <FormError message={form.formError} />
    </form>
  );
}
