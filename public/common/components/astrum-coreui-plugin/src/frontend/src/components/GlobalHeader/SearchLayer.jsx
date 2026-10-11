import React, {useEffect, useRef} from 'react';
import {IconButton} from '@radix-ui/themes';
import {ArrowLeft} from 'lucide-react';
import {useTransitionHook} from '@uirouter/react';
import SearchRadix from './SearchRadix';
import {trapTabKey} from '../../utils/focusTrap';
import UIStrings from '../../constants/UIStrings';

import './SearchLayer.scss';

const {HEADER} = UIStrings.SHELL;

/**
 * The header search on phones and tablets: the search field of the header, alone on a layer over the whole screen.
 * The field gets the focus when the layer opens; the layer closes with its back button, on Esc and once a search or
 * a suggestion has been opened.
 */
export default function SearchLayer({onClose}) {
  const layerRef = useRef(null);

  useTransitionHook('onSuccess', {}, () => {
    onClose();
  });

  useEffect(() => {
    const openedFrom = document.activeElement;
    layerRef.current?.querySelector('input')?.focus();

    const onKeyDown = (event) => {
      if (event.key === 'Escape' && !event.defaultPrevented) {
        onClose();
        return;
      }
      trapTabKey(event, layerRef.current);
    };
    document.addEventListener('keydown', onKeyDown);
    return () => {
      document.removeEventListener('keydown', onKeyDown);
      openedFrom?.focus?.();
    };
  }, [onClose]);

  return (
    <div
      ref={layerRef}
      className="as-search-layer"
      role="dialog"
      aria-modal="true"
      aria-label={HEADER.SEARCH_LABEL}
      data-testid="search-layer"
    >
      <div className="as-search-layer__bar">
        <IconButton
          variant="ghost"
          color="gray"
          size="3"
          className="as-search-layer__close"
          aria-label={HEADER.CLOSE_SEARCH}
          onClick={onClose}
        >
          <ArrowLeft size={20} aria-hidden />
        </IconButton>
        <div className="as-search-layer__field">
          <SearchRadix isPreviewUI />
        </div>
      </div>
    </div>
  );
}
