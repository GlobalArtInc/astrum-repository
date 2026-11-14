import { Button, Callout, Code, Dialog, Flex, Text } from '@radix-ui/themes';
import { TriangleAlert } from 'lucide-react';
import { useState } from 'react';
import { api, useInvalidateApi } from '../../../api';
import { type Translate, useT } from '../../../i18n';
import {
  CodeBlock,
  CopyButton,
  DateTime,
  type Errors,
  FormError,
  NumberField,
  type Option,
  SelectField,
  TextAreaField,
  useForm,
} from '../../../ui';
import {
  type CreatedShareLink,
  INITIAL_SHARE_VALUES,
  type Lifetime,
  SHARE_LINKS,
  type ShareTarget,
  type ShareValues,
  shareRequest,
} from '../../security/share-links/model';
import { explainRefusal } from '../../security/shared/messages';
import styles from './ShareDialog.module.css';

export interface ShareDialogProps extends ShareTarget {
  open: boolean;
  onOpenChange(open: boolean): void;
}

function isWholeNumber(value: number | null, minimum: number): boolean {
  return value !== null && Number.isInteger(value) && value >= minimum;
}

export function shareProblems(t: Translate, values: ShareValues, prefix: boolean): Errors<ShareValues> {
  const problems: Errors<ShareValues> = {};
  if (values.lifetime === 'custom' && !isWholeNumber(values.hours, 1)) {
    problems.hours = t('browse.share.hoursInvalid');
  }
  if (!prefix && values.maxDownloads !== null && !isWholeNumber(values.maxDownloads, 1)) {
    problems.maxDownloads = t('browse.share.limitInvalid');
  }
  return problems;
}

export interface ShareResultProps {
  link: CreatedShareLink;
  onDone(): void;
}

function ShareResult({ link, onDone }: ShareResultProps) {
  const t = useT();
  return (
    <Flex direction="column" gap="4">
      <Callout.Root color="amber" role="alert">
        <Callout.Icon>
          <TriangleAlert size={16} aria-hidden="true" />
        </Callout.Icon>
        <Callout.Text>{t('browse.share.once')}</Callout.Text>
      </Callout.Root>
      <Flex direction="column" gap="2">
        <Text as="p" size="2" weight="medium">
          {t('browse.share.url')}
        </Text>
        <Flex align="center" gap="2" wrap="wrap">
          <Code size="2" variant="soft" className={styles.url} data-testid="share-url">
            {link.url}
          </Code>
          <CopyButton text={link.url} label={t('browse.share.copy')} />
        </Flex>
      </Flex>
      {link.prefix ? (
        <Text as="p" size="2" color="gray">
          {t('browse.share.folderNote')}
        </Text>
      ) : (
        <CodeBlock code={`curl -fLO '${link.url}'`} copyLabel={t('browse.share.copy')} />
      )}
      <Text as="p" size="2" color="gray">
        {t('browse.share.expires')} <DateTime value={link.expiresAt} />
      </Text>
      <Flex justify="end">
        <Button onClick={onDone}>{t('browse.share.done')}</Button>
      </Flex>
    </Flex>
  );
}

export interface ShareFormProps {
  target: ShareTarget;
  onDone(): void;
}

function ShareForm({ target, onDone }: ShareFormProps) {
  const t = useT();
  const invalidate = useInvalidateApi();
  const [created, setCreated] = useState<CreatedShareLink | null>(null);
  const form = useForm<ShareValues>({
    initial: INITIAL_SHARE_VALUES,
    validate: (values) => shareProblems(t, values, target.prefix),
    onSubmit: async (values) => {
      const link = await explainRefusal(t, () =>
        api.post<CreatedShareLink>(SHARE_LINKS, shareRequest(target, values)),
      );
      await invalidate(SHARE_LINKS);
      setCreated(link);
    },
  });
  if (created) {
    return <ShareResult link={created} onDone={onDone} />;
  }
  const lifetimes: Option[] = [
    { value: '1h', label: t('browse.share.preset.1h') },
    { value: '1d', label: t('browse.share.preset.1d') },
    { value: '7d', label: t('browse.share.preset.7d') },
    { value: 'custom', label: t('browse.share.preset.custom') },
  ];
  return (
    <form onSubmit={form.submit} noValidate>
      <Flex direction="column" gap="4">
        <SelectField
          label={t('browse.share.lifetime')}
          name="lifetime"
          value={form.values.lifetime}
          options={lifetimes}
          onChange={(value) => form.set('lifetime', value as Lifetime)}
        />
        {form.values.lifetime === 'custom' && (
          <NumberField
            label={t('browse.share.hours')}
            hint={t('browse.share.hoursHint')}
            min={1}
            step={1}
            required
            {...form.field('hours')}
          />
        )}
        {!target.prefix && (
          <NumberField
            label={t('browse.share.maxDownloads')}
            hint={t('browse.share.maxDownloadsHint')}
            min={1}
            step={1}
            {...form.field('maxDownloads')}
          />
        )}
        <TextAreaField
          label={t('browse.share.note')}
          hint={t('browse.share.noteHint')}
          rows={2}
          {...form.field('note')}
        />
        <FormError message={form.formError} />
        <Flex gap="3" justify="end" wrap="wrap">
          <Dialog.Close>
            <Button type="button" variant="soft" color="gray">
              {t('browse.share.cancel')}
            </Button>
          </Dialog.Close>
          <Button type="submit" loading={form.submitting}>
            {t('browse.share.create')}
          </Button>
        </Flex>
      </Flex>
    </form>
  );
}

export function ShareDialog({ open, onOpenChange, repository, path, prefix }: ShareDialogProps) {
  const t = useT();
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Content maxWidth="520px">
        <Dialog.Title>{prefix ? t('browse.share.folderTitle') : t('browse.share.title')}</Dialog.Title>
        <Dialog.Description size="2" color="gray" mb="4">
          {prefix ? t('browse.share.folderText', { path }) : t('browse.share.text', { path })}
        </Dialog.Description>
        {open && <ShareForm target={{ repository, path, prefix }} onDone={() => onOpenChange(false)} />}
      </Dialog.Content>
    </Dialog.Root>
  );
}
