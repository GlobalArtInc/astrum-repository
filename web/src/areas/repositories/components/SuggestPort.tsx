import { useState } from 'react';
import { api, INTERNAL, withQuery } from '../../../api';
import { useT } from '../../../i18n';
import { describeError, useToast } from '../../../ui';
import styles from './fields.module.css';

export const SUGGEST_PORT = `${INTERNAL}/ui/docker/suggest-port`;

export function SuggestPort({
  other,
  disabled,
  onSuggest,
}: {
  other: number | null;
  disabled?: boolean;
  onSuggest(port: number): void;
}) {
  const t = useT();
  const toast = useToast();
  const [busy, setBusy] = useState(false);
  return (
    <p className={styles.suggest}>
      <button
        type="button"
        className={styles.suggestLink}
        disabled={disabled || busy}
        onClick={async () => {
          setBusy(true);
          try {
            onSuggest(await api.get<number>(withQuery(SUGGEST_PORT, { exclude: other })));
          } catch (error) {
            toast.error(describeError(t, error));
          } finally {
            setBusy(false);
          }
        }}
      >
        {t('repositories.form.suggestPort')}
      </button>
    </p>
  );
}
