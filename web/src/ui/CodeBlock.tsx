import styles from './CodeBlock.module.css';
import { CopyButton } from './CopyButton';

export interface CodeBlockProps {
  code: string;
  title?: string;
  copyLabel?: string;
}

export function CodeBlock({ code, title, copyLabel }: CodeBlockProps) {
  return (
    <div className={styles.block}>
      {title ? (
        <div className={styles.head}>
          <span>{title}</span>
          <CopyButton text={code} label={copyLabel} />
        </div>
      ) : (
        <div className={styles.floating}>
          <CopyButton text={code} label={copyLabel} />
        </div>
      )}
      <pre className={title ? styles.pre : `${styles.pre} ${styles.withFloating}`}>
        <code>{code}</code>
      </pre>
    </div>
  );
}
