
import React, { useEffect, useRef, useState } from 'react';
import PropTypes from 'prop-types';
import { NxPageMain, NxLoadingSpinner, NxErrorAlert, NxButton } from '@globalart/react-shared-components';

import { useSize } from '../../../hooks/useSize';
import { useExtComponent } from './useExtComponent';
import { useExtJsUnsavedChangesGuard } from '@globalart/astrum-ui-plugin';
import { loadExtJS, isExtJSLoaded } from '../../../utils/extJsLoader';
import './ExtJsContainer.scss';

/**
 * ExtJsContainer - Wrapper component for ExtJS views
 *
 * Phase 1: React Shell - This component now lazy-loads ExtJS on demand.
 * React no longer blocks on ExtJS initialization, improving startup time.
 */
export function ExtJsContainer({ className, extView, extParams, title, icon }) {
  const [extJsLoaded, setExtJsLoaded] = useState(isExtJSLoaded());
  const [extJsError, setExtJsError] = useState(null);
  const iconName = icon ? `x-fa fa-${icon.iconName}` : undefined;
  const extContainerRef = useRef(null);
  const wrapperRef = useRef(null);
  const size = useSize(wrapperRef);
  const extComponent = useExtComponent(extJsLoaded ? extContainerRef : null, extView, extParams, title, iconName);

  // Load ExtJS on mount
  useEffect(() => {
    if (!extJsLoaded) {
      console.info('[ExtJsContainer] Requesting ExtJS lazy load for view:', extView);
      loadExtJS()
        .then(() => {
          console.info('[ExtJsContainer] ExtJS loaded successfully for view:', extView);
          setExtJsLoaded(true);
        })
        .catch((error) => {
          console.error('[ExtJsContainer] Failed to load ExtJS for view:', extView, error);
          setExtJsError(error);
        });
    }
  }, [extJsLoaded, extView]);

  // Resize the Ext JS component when the wrapper resizes
  useEffect(() => {
    if (!extJsLoaded) {
      console.info('[ExtJsContainer] Requesting ExtJS lazy load for view:', extView);
      loadExtJS()
        .then(() => {
          console.info('[ExtJsContainer] ExtJS loaded successfully for view:', extView);
          setExtJsLoaded(true);
        })
        .catch((error) => {
          console.error('[ExtJsContainer] Failed to load ExtJS for view:', extView, error);
          setExtJsError(error);
        });
    }
  }, [extJsLoaded, extView]);

  // Resize the Ext JS component when the wrapper resizes.
  // Use window.innerHeight - wrapper.top rather than wrapper.height to avoid a
  // ResizeObserver feedback loop: ExtJS content can push the wrapper taller, which
  // fires the observer with a larger height, which makes ExtJS taller, ad infinitum.
  // The wrapper's top position is stable even as its height grows, so capping ExtJS
  // at (viewport bottom - wrapper top) breaks the amplification.
  useEffect(() => {
    if (extComponent && wrapperRef.current) {
      const top = wrapperRef.current.getBoundingClientRect().top;
      const height = Math.max(0, Math.floor(window.innerHeight - top));
      extComponent.setHeight(height);
      extComponent.setWidth(size.width);
      extComponent.updateLayout();
    }
  }, [size.height, size.width, extComponent]);

  useExtJsUnsavedChangesGuard(extContainerRef);

  // Show error state if ExtJS failed to load
  if (extJsError) {
    return (
      <NxPageMain ref={wrapperRef}>
        <NxErrorAlert>
          <h3>Failed to Load Classic UI Component</h3>
          <p>
            The legacy ExtJS framework failed to initialize. This component requires ExtJS to function.
          </p>
          <p>Error: {extJsError.message}</p>
          <NxButton onClick={() => window.location.reload()}>
            Reload Page
          </NxButton>
        </NxErrorAlert>
      </NxPageMain>
    );
  }

  // Show loading state while ExtJS loads
  if (!extJsLoaded) {
    return (
      <NxPageMain ref={wrapperRef}>
        <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: '400px' }}>
          <NxLoadingSpinner>Loading Classic UI component...</NxLoadingSpinner>
        </div>
      </NxPageMain>
    );
  }

  // Render ExtJS component once loaded
  return (
    <NxPageMain ref={wrapperRef} className='as-ext-js-wrapper'>
      <div className={className} ref={extContainerRef}></div>
    </NxPageMain>
  );
}

ExtJsContainer.propTypes = {
  className: PropTypes.string,
  extView: PropTypes.string,
};
