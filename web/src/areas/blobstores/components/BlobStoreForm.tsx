import { Button, Callout, Heading, Text } from '@radix-ui/themes';
import { Info } from 'lucide-react';
import { type FormEvent, type ReactNode, useEffect, useState } from 'react';
import { useT } from '../../../i18n';
import {
  FileSize,
  FormError,
  NumberField,
  OrderedListField,
  SecretField,
  SelectField,
  SwitchField,
  TextField,
  UnsavedChangesGuard,
  useForm,
} from '../../../ui';
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
  storedSecret,
  toDocument,
  validate,
} from '../model';
import { typeLabel } from './bits';

const NONE = '_none';

const QUOTA_LABELS: Record<(typeof QUOTA_TYPES)[number], string> = {
  spaceUsedQuota: 'blobstores.quota.spaceUsedQuota',
  spaceRemainingQuota: 'blobstores.quota.spaceRemainingQuota',
};

const FILL_LABELS: Record<(typeof FILL_POLICIES)[number], string> = {
  writeToFirst: 'blobstores.group.writeToFirst',
  roundRobin: 'blobstores.group.roundRobin',
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
  onSubmit(document: StoreDocument): Promise<void>;
  onSaved?(document: StoreDocument): void;
  onCancel?(): void;
}

function Section({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
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
  const [saved, setSaved] = useState<StoreDocument | null>(null);
  const form = useForm<StoreValues>({
    initial: fromDocument(document),
    validate: (values) => validate(values, { type, creating, storedKeyId }, t),
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
  const onSaved = props.onSaved;
  useEffect(() => {
    if (saved && !form.dirty) {
      setSaved(null);
      onSaved?.(saved);
    }
  }, [saved, form.dirty, onSaved]);

  const field = <K extends keyof StoreValues>(name: K) => ({ ...form.field(name), disabled: readOnly });
  const self = creating ? values.name.trim() : (props.storeName ?? '');
  const byName = new Map(props.stores.map((store) => [store.name, store]));
  const choices = memberChoices(props.stores, self);
  const memberOptions = [
    ...choices.map((store) => ({ value: store.name, label: store.name })),
    ...values.members
      .filter((name) => !choices.some((store) => store.name === name))
      .map((name) => ({ value: name, label: name })),
  ];
  const keepsSecret = storedKeyId !== null && values.accessKeyId.trim() === storedKeyId;

  const submit = (event: FormEvent) => {
    void form.submit(event);
  };

  return (
    <form className={styles.form} onSubmit={submit} noValidate aria-label={props.submitLabel}>
      <UnsavedChangesGuard when={form.dirty && !readOnly} />
      {readOnly && (
        <Callout.Root color="gray" variant="surface">
          <Callout.Icon>
            <Info size={16} />
          </Callout.Icon>
          <Callout.Text>{t('blobstores.form.readOnly')}</Callout.Text>
        </Callout.Root>
      )}
      {(creating || type === 'file') && (
        <Section title={t('blobstores.form.general')}>
          {creating && (
            <TextField
              {...field('name')}
              onChange={(name) => {
                if (type === 'file' && (values.path === '' || values.path === values.name)) {
                  form.set('path', name);
                }
                form.set('name', name);
              }}
              label={t('blobstores.form.name')}
              hint={t('blobstores.form.nameHint')}
              required
              autoFocus
            />
          )}
          {type === 'file' && (
            <TextField
              {...field('path')}
              label={t('blobstores.file.path')}
              hint={t('blobstores.file.pathHint')}
              required
            />
          )}
        </Section>
      )}

      {type === 's3' && (
        <>
          <Section title={t('blobstores.s3.bucketSection')} description={t('blobstores.s3.bucketText')}>
            <TextField
              {...field('bucket')}
              label={t('blobstores.s3.bucket')}
              hint={t('blobstores.s3.bucketHint')}
              required
            />
            <TextField
              {...field('region')}
              label={t('blobstores.s3.region')}
              hint={t('blobstores.s3.regionHint')}
              required
            />
            <TextField
              {...field('prefix')}
              label={t('blobstores.s3.prefix')}
              hint={t('blobstores.s3.prefixHint')}
            />
            <NumberField
              {...field('expiration')}
              label={t('blobstores.s3.expiration')}
              hint={t('blobstores.s3.expirationHint')}
              min={-1}
            />
          </Section>
          <Section title={t('blobstores.s3.credentials')} description={t('blobstores.s3.credentialsText')}>
            <TextField {...field('accessKeyId')} label={t('blobstores.s3.accessKeyId')} autoComplete="off" />
            <SecretField
              {...field('secretAccessKey')}
              label={t('blobstores.s3.secretAccessKey')}
              stored={keepsSecret}
              required={values.accessKeyId.trim() !== ''}
            />
            <TextField
              {...field('role')}
              label={t('blobstores.s3.role')}
              hint={t('blobstores.s3.roleHint')}
              autoComplete="off"
            />
            <SecretField
              {...field('sessionToken')}
              label={t('blobstores.s3.sessionToken')}
              hint={t('blobstores.s3.sessionTokenHint')}
            />
          </Section>
          <Section title={t('blobstores.s3.encryption')}>
            <SelectField
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
              <TextField
                {...field('encryptionKey')}
                label={t('blobstores.s3.kmsKey')}
                hint={t('blobstores.s3.kmsKeyHint')}
              />
            )}
          </Section>
          <Section title={t('blobstores.s3.connection')}>
            <Wide>
              <TextField
                {...field('endpoint')}
                type="url"
                label={t('blobstores.s3.endpoint')}
                hint={t('blobstores.s3.endpointHint')}
              />
            </Wide>
            <Wide>
              <SwitchField
                {...field('forcePathStyle')}
                label={t('blobstores.s3.pathStyle')}
                description={t('blobstores.s3.pathStyleHint')}
              />
            </Wide>
          </Section>
        </>
      )}

      {type === 'group' && (
        <Section title={t('blobstores.group.section')} description={t('blobstores.group.text')}>
          <Wide>
            <OrderedListField
              {...field('members')}
              label={t('blobstores.group.members')}
              hint={t('blobstores.group.membersHint')}
              options={memberOptions}
              addPlaceholder={t('blobstores.group.addMember')}
              empty={t('blobstores.group.noMembers')}
              noOptions={t('blobstores.group.noCandidates')}
              describe={(name) => {
                const store = byName.get(name);
                return store ? (
                  <Text size="1" color="gray">
                    {typeLabel(t, store.type)} · {t('blobstores.list.blobs', { count: store.blobCount })} ·{' '}
                    <FileSize bytes={store.totalSize} />
                  </Text>
                ) : null;
              }}
            />
          </Wide>
          <SelectField
            {...field('fillPolicy')}
            label={t('blobstores.group.fillPolicy')}
            hint={t('blobstores.group.fillPolicyHint')}
            options={FILL_POLICIES.map((value) => ({ value, label: t(FILL_LABELS[value]) }))}
          />
        </Section>
      )}

      <Section title={t('blobstores.quota.title')} description={t('blobstores.quota.text')}>
        <Wide>
          <SwitchField {...field('quotaEnabled')} label={t('blobstores.quota.enable')} />
        </Wide>
        {values.quotaEnabled && (
          <>
            <SelectField
              {...field('quotaType')}
              label={t('blobstores.quota.type')}
              options={QUOTA_TYPES.map((value) => ({ value, label: t(QUOTA_LABELS[value]) }))}
            />
            <NumberField
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
      {!readOnly && (
        <div className={styles.footer}>
          {creating ? (
            <Button type="button" variant="soft" color="gray" onClick={props.onCancel}>
              {t('blobstores.form.cancel')}
            </Button>
          ) : (
            <Button
              type="button"
              variant="soft"
              color="gray"
              disabled={!form.dirty || form.submitting}
              onClick={() => form.reset()}
            >
              {t('blobstores.form.discard')}
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
