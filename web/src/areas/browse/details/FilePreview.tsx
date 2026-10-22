import { SegmentedControl, Skeleton, Text } from '@radix-ui/themes';
import { useQuery } from '@tanstack/react-query';
import { type ReactNode, useState } from 'react';
import { api } from '../../../api';
import { formatBytes, useI18n, useT } from '../../../i18n';
import { CopyButton, describeError } from '../../../ui';
import detailStyles from './Details.module.css';
import { Markdown } from './markdown';
import styles from './Preview.module.css';
import { contentPath, decodeText, IMAGE_LIMIT, linesOf, previewKind, TEXT_LIMIT } from './preview';

export interface FilePreviewProps {
  repository: string;
  file: string;
  name: string;
  contentType: string;
  size: number;
}

type Mode = 'rendered' | 'source';

function Frame({ actions, children }: { actions?: ReactNode; children: ReactNode }) {
  const t = useT();
  return (
    <section className={detailStyles.card} aria-label={t('browse.preview.title')}>
      <div className={styles.head}>
        <Text as="p" size="3" weight="bold" className={detailStyles.sectionTitle}>
          {t('browse.preview.title')}
        </Text>
        {actions && <div className={styles.actions}>{actions}</div>}
      </div>
      {children}
    </section>
  );
}

function Note({ children }: { children: ReactNode }) {
  return (
    <Text as="p" size="2" color="gray" className={styles.note}>
      {children}
    </Text>
  );
}

function SourceView({ text }: { text: string }) {
  const lines = linesOf(text);
  return (
    <div className={styles.source}>
      <pre className={styles.numbers} aria-hidden="true">
        {lines.map((_, index) => index + 1).join('\n')}
      </pre>
      <pre className={styles.text}>
        <code>{lines.join('\n')}</code>
      </pre>
    </div>
  );
}

function TextPreview({ url, markdown }: { url: string; markdown: boolean }) {
  const t = useT();
  const [mode, setMode] = useState<Mode>('rendered');
  const query = useQuery({
    queryKey: ['browse-preview', url],
    queryFn: ({ signal }) => api.get<ArrayBuffer>(url, { bytes: true, signal }),
    staleTime: 30_000,
  });
  if (query.isPending) {
    return (
      <Frame>
        <Skeleton height="160px" />
      </Frame>
    );
  }
  if (query.isError) {
    return (
      <Frame>
        <Note>{describeError(t, query.error)}</Note>
      </Frame>
    );
  }
  const text = decodeText(query.data);
  if (text === null) {
    return (
      <Frame>
        <Note>{t('browse.preview.binary')}</Note>
      </Frame>
    );
  }
  if (text === '') {
    return (
      <Frame>
        <Note>{t('browse.preview.empty')}</Note>
      </Frame>
    );
  }
  const rendered = markdown && mode === 'rendered';
  return (
    <Frame
      actions={
        <>
          {markdown && (
            <SegmentedControl.Root
              size="1"
              value={mode}
              onValueChange={(next) => setMode(next === 'source' ? 'source' : 'rendered')}
              aria-label={t('browse.preview.mode')}
            >
              <SegmentedControl.Item value="rendered">{t('browse.preview.rendered')}</SegmentedControl.Item>
              <SegmentedControl.Item value="source">{t('browse.preview.source')}</SegmentedControl.Item>
            </SegmentedControl.Root>
          )}
          <CopyButton text={text} label={t('browse.preview.copy')} />
        </>
      }
    >
      <div className={styles.body} data-rendered={rendered}>
        {rendered ? <Markdown source={text} /> : <SourceView text={text} />}
      </div>
    </Frame>
  );
}

function ImagePreview({ url, name }: { url: string; name: string }) {
  const t = useT();
  const [broken, setBroken] = useState(false);
  return (
    <Frame>
      {broken ? (
        <Note>{t('browse.preview.imageFailed')}</Note>
      ) : (
        <div className={styles.image}>
          <img src={url} alt={name} onError={() => setBroken(true)} />
        </div>
      )}
    </Frame>
  );
}

export function FilePreview({ repository, file, name, contentType, size }: FilePreviewProps) {
  const t = useT();
  const { language } = useI18n();
  const kind = previewKind(contentType, name, size);
  const url = contentPath(repository, file);
  switch (kind) {
    case 'binary':
      return null;
    case 'large':
      return (
        <Frame>
          <Note>
            {t('browse.preview.large', {
              limit: formatBytes(language, contentType.startsWith('image/') ? IMAGE_LIMIT : TEXT_LIMIT),
            })}
          </Note>
        </Frame>
      );
    case 'image':
      return <ImagePreview key={url} url={url} name={name} />;
    default:
      return <TextPreview key={url} url={url} markdown={kind === 'markdown'} />;
  }
}
