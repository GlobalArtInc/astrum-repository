import { Button, Dialog, Flex, RadioGroup, Text } from '@radix-ui/themes';
import { useState } from 'react';
import { ApiError, api, V1 } from '../../../api';
import { type Translate, useT } from '../../../i18n';
import {
  CheckboxField,
  type Column,
  DataTable,
  describeError,
  FileSize,
  FormError,
  SelectField,
  StatusBadge,
  TextField,
  type Tone,
} from '../../../ui';
import {
  CONFLICT_MODES,
  type ConflictMode,
  isConflictMode,
  itemLabel,
  type PromotionItem,
  type PromotionResponse,
  type PromotionResult,
  type PromotionStatus,
  type PromotionTarget,
  promotionRequest,
} from './model';

export interface PromoteDialogProps {
  open: boolean;
  onOpenChange(open: boolean): void;
  source: string;
  format: string;
  items: PromotionItem[];
  targets: PromotionTarget[];
}

interface ResultRow {
  id: string;
  label: string;
  result: PromotionResult;
}

type Run = 'preview' | 'promote';

const STATUS_TONES: Record<PromotionStatus, Tone> = {
  copied: 'success',
  unchanged: 'neutral',
  skipped: 'neutral',
  conflict: 'warning',
  forbidden: 'danger',
  notFound: 'danger',
  unsupported: 'danger',
  failed: 'danger',
};

const STATUS_KEYS: Record<PromotionStatus, string> = {
  copied: 'browse.promote.status.copied',
  unchanged: 'browse.promote.status.unchanged',
  skipped: 'browse.promote.status.skipped',
  conflict: 'browse.promote.status.conflict',
  forbidden: 'browse.promote.status.forbidden',
  notFound: 'browse.promote.status.notFound',
  unsupported: 'browse.promote.status.unsupported',
  failed: 'browse.promote.status.failed',
};

interface ConflictCopy {
  label: string;
  hint: string;
}

const CONFLICT_KEYS: Record<ConflictMode, ConflictCopy> = {
  fail: { label: 'browse.promote.conflictFail', hint: 'browse.promote.conflictFailHint' },
  skip: { label: 'browse.promote.conflictSkip', hint: 'browse.promote.conflictSkipHint' },
  replace: { label: 'browse.promote.conflictReplace', hint: 'browse.promote.conflictReplaceHint' },
};

function failureText(t: Translate, failure: unknown): string {
  return failure instanceof ApiError && !failure.isNetwork ? failure.message : describeError(t, failure);
}

interface ResultTableProps {
  label: string;
  results: PromotionResult[];
  items: PromotionItem[];
}

function ResultTable({ label, results, items }: ResultTableProps) {
  const t = useT();
  const rows: ResultRow[] = results.map((result, index) => ({
    id: String(index),
    label: itemLabel(result.item ?? items[index]),
    result,
  }));
  const columns: Column<ResultRow>[] = [
    { id: 'item', header: t('browse.promote.column.item'), cell: (row) => row.label, primary: true },
    {
      id: 'status',
      header: t('browse.promote.column.status'),
      cell: (row) => (
        <StatusBadge tone={STATUS_TONES[row.result.status]}>{t(STATUS_KEYS[row.result.status])}</StatusBadge>
      ),
    },
    {
      id: 'assets',
      header: t('browse.promote.column.assets'),
      cell: (row) => (
        <Flex direction="column">
          <Text size="2">{t('browse.promote.assets', { count: row.result.assetCount ?? 0 })}</Text>
          <Text size="1" color="gray">
            {t('browse.promote.copied')}: <FileSize bytes={row.result.bytesCopied ?? 0} />
            {' · '}
            {t('browse.promote.reused')}: <FileSize bytes={row.result.bytesReused ?? 0} />
          </Text>
        </Flex>
      ),
    },
    {
      id: 'details',
      header: t('browse.promote.column.details'),
      cell: (row) => (
        <Flex direction="column" gap="1">
          {row.result.error && (
            <Text size="2" color="red">
              {row.result.error}
            </Text>
          )}
          {row.result.warning && (
            <Text size="2" color="amber">
              {row.result.warning}
            </Text>
          )}
          {(row.result.skippedChildren ?? 0) > 0 && (
            <Text size="1" color="amber">
              {t('browse.promote.skippedChildren', { count: row.result.skippedChildren ?? 0 })}
            </Text>
          )}
          {row.result.sourceKept === true && (
            <StatusBadge tone="warning">{t('browse.promote.sourceKept')}</StatusBadge>
          )}
          {row.result.retryable === true && (
            <Text size="1" color="gray">
              {t('browse.promote.retryHint')}
            </Text>
          )}
        </Flex>
      ),
    },
  ];
  return <DataTable<ResultRow> label={label} columns={columns} rows={rows} getRowId={(row) => row.id} />;
}

export function PromoteDialog({ open, onOpenChange, source, format, items, targets }: PromoteDialogProps) {
  const t = useT();
  const [target, setTarget] = useState('');
  const [move, setMove] = useState(false);
  const [conflict, setConflict] = useState<ConflictMode>('fail');
  const [tags, setTags] = useState('');
  const [running, setRunning] = useState<Run | null>(null);
  const [failure, setFailure] = useState<unknown>(null);
  const [preview, setPreview] = useState<PromotionResult[] | null>(null);
  const [results, setResults] = useState<PromotionResult[] | null>(null);
  const busy = running !== null;
  const gated = move && preview === null;
  const canPreview = target !== '' && items.length > 0 && !busy;
  const canRun = canPreview && !gated;
  const edit =
    <Value,>(set: (value: Value) => void) =>
    (value: Value) => {
      set(value);
      setPreview(null);
    };
  const change = (next: boolean) => {
    if (busy) {
      return;
    }
    if (!next) {
      setPreview(null);
      setResults(null);
      setFailure(null);
    }
    onOpenChange(next);
  };
  const send = async (run: Run) => {
    setRunning(run);
    setFailure(null);
    try {
      const response = await api.post<PromotionResponse>(
        `${V1}/promotions`,
        promotionRequest(source, items, { target, move, conflict, tags }, run === 'preview'),
      );
      if (run === 'preview') {
        setPreview(response.results);
      } else {
        setResults(response.results);
      }
    } catch (error) {
      setFailure(error);
    } finally {
      setRunning(null);
    }
  };
  return (
    <Dialog.Root open={open} onOpenChange={change}>
      <Dialog.Content maxWidth="760px">
        <Dialog.Title>{t('browse.promote.title', { source })}</Dialog.Title>
        <Dialog.Description size="2" color="gray">
          {t('browse.promote.description')}
        </Dialog.Description>
        <Flex direction="column" gap="4" mt="4">
          <Text size="2" color="gray">
            {t('browse.promote.count', { count: items.length })}
          </Text>
          <SelectField
            label={t('browse.promote.target')}
            value={target}
            onChange={edit(setTarget)}
            options={targets.map((candidate) => ({ value: candidate.name, label: candidate.name }))}
            placeholder={t('browse.promote.targetPlaceholder')}
            disabled={busy}
            required
          />
          <CheckboxField
            label={t('browse.promote.move')}
            value={move}
            onChange={edit(setMove)}
            description={t('browse.promote.moveWarning')}
            disabled={busy}
          />
          <Flex direction="column" gap="2">
            <Text size="2" weight="medium">
              {t('browse.promote.conflict')}
            </Text>
            <RadioGroup.Root
              value={conflict}
              onValueChange={(value) => {
                if (isConflictMode(value)) {
                  edit(setConflict)(value);
                }
              }}
              disabled={busy}
            >
              <Flex direction="column" gap="2">
                {CONFLICT_MODES.map((mode) => (
                  <Text as="label" key={mode} size="2">
                    <Flex gap="2" align="start">
                      <RadioGroup.Item value={mode} />
                      <span>
                        {t(CONFLICT_KEYS[mode].label)}
                        <Text as="span" size="1" color="gray" style={{ display: 'block' }}>
                          {t(CONFLICT_KEYS[mode].hint)}
                        </Text>
                      </span>
                    </Flex>
                  </Text>
                ))}
              </Flex>
            </RadioGroup.Root>
          </Flex>
          {format === 'docker' && (
            <TextField
              label={t('browse.promote.extraTags')}
              hint={t('browse.promote.extraTagsHint')}
              value={tags}
              onChange={edit(setTags)}
              disabled={busy}
            />
          )}
        </Flex>
        <FormError message={failure === null ? null : failureText(t, failure)} />
        {preview !== null && (
          <Flex mt="4">
            <ResultTable label={t('browse.promote.previewLabel')} results={preview} items={items} />
          </Flex>
        )}
        {results !== null && (
          <Flex mt="4">
            <ResultTable label={t('browse.promote.resultsLabel')} results={results} items={items} />
          </Flex>
        )}
        <Flex gap="3" mt="4" justify="end" wrap="wrap">
          <Button variant="soft" color="gray" disabled={busy} onClick={() => change(false)}>
            {t('ui.confirm.cancel')}
          </Button>
          <Button
            variant="soft"
            disabled={!canPreview}
            loading={running === 'preview'}
            onClick={() => void send('preview')}
          >
            {t('browse.promote.preview')}
          </Button>
          <Button disabled={!canRun} loading={running === 'promote'} onClick={() => void send('promote')}>
            {t('browse.promote.run')}
          </Button>
        </Flex>
        {gated && (
          <Text as="p" size="2" color="amber" mt="3" align="right">
            {t('browse.promote.previewFirst')}
          </Text>
        )}
      </Dialog.Content>
    </Dialog.Root>
  );
}
