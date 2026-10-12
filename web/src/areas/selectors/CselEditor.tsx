import { Button, Flex, Text, TextArea } from '@radix-ui/themes';
import { CircleAlert, CircleCheck } from 'lucide-react';
import { useId, useRef } from 'react';
import { useT } from '../../i18n';
import { FieldShell } from '../../ui';
import { insertAt } from './model';
import styles from './Selectors.module.css';

const TOKENS = [
  { token: 'format', help: 'selectors.csel.format' },
  { token: 'path', help: 'selectors.csel.path' },
  { token: '==', help: 'selectors.csel.equals' },
  { token: '!=', help: 'selectors.csel.notEquals' },
  { token: '=~', help: 'selectors.csel.matches' },
  { token: '=^', help: 'selectors.csel.startsWith' },
  { token: 'and', help: 'selectors.csel.and' },
  { token: 'or', help: 'selectors.csel.or' },
  { token: '( )', help: 'selectors.csel.group' },
] as const;

const EXAMPLES = [
  { expression: 'format == "raw"', help: 'selectors.csel.exampleRaw' },
  { expression: 'format == "maven2" and path =^ "/org/example/"', help: 'selectors.csel.exampleMaven' },
  { expression: 'format == "npm" and path =^ "/@team/"', help: 'selectors.csel.exampleNpm' },
  { expression: 'path =~ ".*/release/.*\\.jar"', help: 'selectors.csel.exampleRegex' },
  {
    expression: '(format == "docker" or format == "npm") and path != "/latest"',
    help: 'selectors.csel.exampleGroup',
  },
] as const;

export interface CselEditorProps {
  value: string;
  onChange(value: string): void;
  error?: string;
  problem?: string | null;
  valid?: boolean;
  disabled?: boolean;
}

export function CselEditor({ value, onChange, error, problem, valid, disabled = false }: CselEditorProps) {
  const t = useT();
  const id = useId();
  const area = useRef<HTMLTextAreaElement>(null);

  const insert = (token: string) => {
    const element = area.current;
    const start = element?.selectionStart ?? value.length;
    const end = element?.selectionEnd ?? value.length;
    const text = token === '( )' ? '()' : token;
    const next = insertAt(value, text, start, end);
    onChange(next.text);
    requestAnimationFrame(() => {
      if (element) {
        element.focus();
        const caret = token === '( )' ? next.caret - 1 : next.caret;
        element.setSelectionRange(caret, caret);
      }
    });
  };

  const statusId = `${id}-status`;

  return (
    <div className={styles.editor}>
      <FieldShell
        id={id}
        label={t('selectors.form.expression')}
        required
        error={error}
        hint={t('selectors.form.expressionHint')}
      >
        <TextArea
          ref={area}
          id={id}
          name="expression"
          value={value}
          rows={4}
          disabled={disabled}
          spellCheck={false}
          autoCapitalize="off"
          autoComplete="off"
          placeholder='format == "maven2" and path =^ "/org/"'
          className={styles.code}
          aria-invalid={error || problem ? true : undefined}
          aria-describedby={[`${id}-hint`, error ? `${id}-error` : '', statusId].filter(Boolean).join(' ')}
          onChange={(event) => onChange(event.target.value)}
        />
      </FieldShell>
      <div id={statusId} aria-live="polite" className={styles.status}>
        {problem ? (
          <Text size="1" color="red">
            <CircleAlert size={14} aria-hidden="true" />
            {t('selectors.form.invalid', { message: problem })}
          </Text>
        ) : valid && value.trim() !== '' ? (
          <Text size="1" color="green">
            <CircleCheck size={14} aria-hidden="true" />
            {t('selectors.form.valid')}
          </Text>
        ) : null}
      </div>
      {!disabled && (
        <div className={styles.hints}>
          <Flex direction="column" gap="2">
            <Text size="2" weight="medium">
              {t('selectors.csel.syntax')}
            </Text>
            <ul className={styles.tokens}>
              {TOKENS.map((item) => (
                <li key={item.token}>
                  <Button
                    type="button"
                    size="1"
                    variant="soft"
                    color="gray"
                    className={styles.token}
                    aria-label={t('selectors.csel.insert', { token: item.token, help: t(item.help) })}
                    onClick={() => insert(item.token)}
                  >
                    <code>{item.token}</code>
                  </Button>
                  <Text size="1" color="gray">
                    {t(item.help)}
                  </Text>
                </li>
              ))}
            </ul>
          </Flex>
          <Flex direction="column" gap="2">
            <Text size="2" weight="medium">
              {t('selectors.csel.examples')}
            </Text>
            <ul className={styles.examples}>
              {EXAMPLES.map((example) => (
                <li key={example.expression}>
                  <button
                    type="button"
                    className={styles.example}
                    aria-label={t('selectors.csel.use', { expression: example.expression })}
                    onClick={() => {
                      onChange(example.expression);
                      area.current?.focus();
                    }}
                  >
                    <code>{example.expression}</code>
                    <Text size="1" color="gray">
                      {t(example.help)}
                    </Text>
                  </button>
                </li>
              ))}
            </ul>
          </Flex>
        </div>
      )}
    </div>
  );
}
