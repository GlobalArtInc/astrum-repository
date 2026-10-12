import { Theme } from '@radix-ui/themes';
import styles from './CodeBlock.module.css';
import { CopyButton } from './CopyButton';

export interface CodeBlockProps {
  code: string;
  title?: string;
  copyLabel?: string;
}

export function CodeBlock({ code, title, copyLabel }: CodeBlockProps) {
  return (
    <Theme appearance="dark" hasBackground={false} className={styles.theme}>
      <div className={styles.block}>
        {title && <div className={styles.head}>{title}</div>}
        <div className={styles.row}>
          <pre className={styles.pre}>
            <code>{code}</code>
          </pre>
          <CopyButton text={code} label={copyLabel} variant="text" />
        </div>
      </div>
    </Theme>
  );
}
