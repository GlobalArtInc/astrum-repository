
import React, {useState, useEffect, useCallback, useRef} from 'react';
import {
  NxButton,
  NxFontAwesomeIcon,
  NxH2,
  NxInfoAlert,
  NxModal,
} from '@globalart/react-shared-components';
import {faCopy} from '@fortawesome/free-solid-svg-icons';

import UIStrings from '../../../../constants/UIStrings';
import {TOKEN_MODAL_AUTO_CLOSE_SECONDS} from './ServiceAccountTokensHelper';
import './ServiceAccountTokens.scss';

const LABELS = UIStrings.SERVICE_ACCOUNT_TOKENS.TOKEN_MODAL;

export default function ServiceAccountTokensTokenModal({token, onClose}) {
  const [copied, setCopied] = useState(false);
  const [copyFailed, setCopyFailed] = useState(false);
  const [secondsLeft, setSecondsLeft] = useState(TOKEN_MODAL_AUTO_CLOSE_SECONDS);
  const intervalRef = useRef(null);
  const onCloseRef = useRef(onClose);

  useEffect(() => {
    onCloseRef.current = onClose;
  });

  useEffect(() => {
    intervalRef.current = setInterval(() => {
      setSecondsLeft((prev) => {
        if (prev <= 1) {
          clearInterval(intervalRef.current);
          onCloseRef.current();
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, []);

  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(token)
      .then(() => {
        setCopyFailed(false);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      })
      .catch(() => {
        setCopied(false);
        setCopyFailed(true);
      });
  }, [token]);

  return (
    <NxModal onCancel={onClose} aria-labelledby="token-reveal-modal">
      <NxModal.Header>
        <NxH2 id="token-reveal-modal">{LABELS.TITLE}</NxH2>
      </NxModal.Header>
      <NxModal.Content>
        <NxInfoAlert role="alert">
          <strong>{LABELS.WARNING}</strong>
        </NxInfoAlert>
        <span className="as-sa-sr-only" role="status" aria-live="polite">
          {copied ? LABELS.COPY_ANNOUNCEMENT : ''}
        </span>
        <div className="as-sa-token-reveal">
          <code className="as-sa-token-value">{token}</code>
          <NxButton type="button" variant="icon-only" className="as-sa-copy-btn" onClick={handleCopy} title={copied ? LABELS.COPIED_BUTTON : LABELS.COPY_BUTTON}>
            <NxFontAwesomeIcon icon={faCopy} />
          </NxButton>
        </div>
        {copyFailed && (
          <p className="as-sa-copy-error" role="alert" data-testid="sa-token-copy-error">
            {LABELS.COPY_FAILED}
          </p>
        )}
      </NxModal.Content>
      <footer className="nx-footer as-sa-token-footer">
        <span className="as-sa-auto-close-notice">{LABELS.AUTO_CLOSE_NOTICE(secondsLeft)}</span>
        <div className="nx-btn-bar">
          <NxButton type="button" onClick={onClose}>
            {LABELS.DONE_BUTTON}
          </NxButton>
        </div>
      </footer>
    </NxModal>
  );
}
