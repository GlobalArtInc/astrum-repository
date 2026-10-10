

import {useEffect, useState} from "react";

function renderExtComponent(extContainerRef, extView, extParams, currentTitle, currentIcon) {
  const page = document.getElementsByClassName("as-ext-js-wrapper")[0];
  // Use viewport height minus wrapper's top offset for the initial height.
  // page.offsetHeight can reflect a stale/overflowed value if the wrapper has already grown;
  // viewport-relative calculation gives the true available space.
  const top = page ? page.getBoundingClientRect().top : 0;
  const initialHeight = page ? Math.max(0, window.innerHeight - top) : 0;
  const initialWidth = page ? page.offsetWidth : 0;

  return Ext.create('AS.view.feature.Content', {
    itemId: 'feature-content',
    renderTo: extContainerRef.current,
    height: initialHeight,
    width: initialWidth,
    autoScroll: true,
    cls: 'as-feature-content',
    currentTitle,
    currentIcon,
    layout: 'fit',
    items: [Ext.create(extView, extParams)]
  });
}

/**
 * Render an ExtJs container to the DOM
 * @param extContainerRef ObjectRef
 * @param extView string
 * @param extParams object
 * @param currentTitle string | undefined
 * @param currentIcon string | undefined
 * @returns {unknown}
 */
export function useExtComponent(extContainerRef, extView, extParams, currentTitle, currentIcon) {
  const [extComponent, setExtComponent] = useState(null);

  // Render and track the Ext JS component
  useEffect(() => {
    if (!extContainerRef?.current) {
      return;
    }

    if (!extComponent) {
      setExtComponent(renderExtComponent(
          extContainerRef,
          extView,
          extParams,
          currentTitle,
          currentIcon
      ));
    }

    return () => {
      if (extComponent) {
        extComponent.destroy();
      }
    };
  }, [extComponent, extContainerRef]);

  return extComponent;
}
