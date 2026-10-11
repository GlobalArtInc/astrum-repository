import React, { useCallback, useId, useMemo, useRef, useState } from 'react';
import { Flex, Text, Theme } from '@radix-ui/themes';

import UIStrings from '../../constants/UIStrings';
import type { UsageSnippet } from '../../pages/search/details/dependencySnippets/usage';
import { CopyButton } from './CopyButton';

import './Usage.scss';

export interface UsageBlockProps {
  /** The snippets, one tab each. */
  snippets: readonly UsageSnippet[];
  /** Heading above the tabs; the block's own title by default. */
  title?: string;
  /** Called after a snippet was copied. */
  onCopied?: (snippet: UsageSnippet) => void;
  /** Called when a snippet could not be copied. */
  onCopyFailed?: (snippet: UsageSnippet) => void;
}

/**
 * The "Usage" block: one tab per client, a hint for the chosen one and its snippet in a dark code box
 * with a copy button. Renders nothing without snippets.
 */
export function UsageBlock({ snippets, title, onCopied, onCopyFailed }: UsageBlockProps): JSX.Element | null {
  const strings = UIStrings.BROWSE.USAGE;
  const baseId = useId();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const tabsRef = useRef<Array<HTMLButtonElement | null>>([]);

  const selected = useMemo(
    () => snippets.find((snippet) => snippet.id === selectedId) ?? snippets[0],
    [snippets, selectedId],
  );

  const select = useCallback((index: number) => {
    const snippet = snippets[index];
    if (snippet) {
      setSelectedId(snippet.id);
      tabsRef.current[index]?.focus();
    }
  }, [snippets]);

  const handleKeyDown = useCallback((event: React.KeyboardEvent, index: number) => {
    const last = snippets.length - 1;
    const next: Record<string, number> = {
      ArrowRight: index === last ? 0 : index + 1,
      ArrowLeft: index === 0 ? last : index - 1,
      Home: 0,
      End: last,
    };
    if (event.key in next) {
      event.preventDefault();
      select(next[event.key]);
    }
  }, [snippets.length, select]);

  if (!selected) {
    return null;
  }

  const tabId = (snippet: UsageSnippet) => `${baseId}-tab-${snippet.id}`;
  const panelId = `${baseId}-panel`;

  return (
    <Flex direction="column" gap="3" className="usage-block" data-testid="usage-block">
      <Text as="div" size="3" weight="bold">{title ?? strings.TITLE}</Text>
      <div role="tablist" aria-label={strings.CLIENTS} className="usage-block__tabs">
        {snippets.map((snippet, index) => {
          const isSelected = snippet.id === selected.id;
          return (
            <button
              key={snippet.id}
              ref={(element) => { tabsRef.current[index] = element; }}
              type="button"
              role="tab"
              id={tabId(snippet)}
              aria-selected={isSelected}
              aria-controls={panelId}
              tabIndex={isSelected ? 0 : -1}
              className="usage-block__tab"
              onClick={() => setSelectedId(snippet.id)}
              onKeyDown={(event) => handleKeyDown(event, index)}
            >
              {snippet.label}
            </button>
          );
        })}
      </div>
      {selected.description && (
        <Text as="p" size="2" color="gray" className="usage-block__hint">
          {selected.description}
        </Text>
      )}
      <Theme appearance="dark" hasBackground={false} className="usage-block__theme">
        <div
          role="tabpanel"
          id={panelId}
          aria-labelledby={tabId(selected)}
          className="usage-block__code"
        >
          <pre className="usage-block__pre">
            <code>{selected.snippetText}</code>
          </pre>
          <CopyButton
            text={selected.snippetText}
            label={strings.COPY_LABEL(selected.label)}
            onCopied={() => onCopied?.(selected)}
            onCopyFailed={() => onCopyFailed?.(selected)}
          />
        </div>
      </Theme>
    </Flex>
  );
}

export default UsageBlock;
