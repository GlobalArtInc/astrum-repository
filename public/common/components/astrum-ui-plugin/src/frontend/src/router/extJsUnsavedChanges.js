
import { useEffect } from 'react';

/**
 * Utility to check for unsaved changes in ExtJS forms and, if dirty, show the modal and run a callback if the user confirms.
 * @param {object} menuCtrl - ExtJS Menu controller
 * @param {Function} onProceed - Called if no unsaved changes or user confirms
 * @param {Function} [onCancel] - Called if user cancels (optional)
 * @returns {boolean} true if no modal was shown (proceed immediately), false if modal was shown
 */
export function handleExtJsUnsavedChanges(menuCtrl, onProceed, onCancel) {
  if (!menuCtrl || typeof menuCtrl.hasDirt !== 'function') {
    // No menuCtrl available, proceed immediately
    onProceed();
    return true;
  }

  const hasDirt = menuCtrl.hasDirt();

  if (!hasDirt) {
    // No unsaved changes, proceed immediately
    onProceed();
    return true;
  }

  // Show modal, only proceed if user confirms
  menuCtrl.warnBeforeNavigate(() => {
    clearFormDirty(menuCtrl);
    onProceed();
  }, onCancel);
  return false;
}

/**
 * Hook to guard against unsaved changes in ExtJS forms when navigating away.
 * Handles React-side <a href="#..."> clicks and browser navigation (popstate).
 * @param {object} extContainerRef - React ref to the ExtJS container DOM node
 */
export function useExtJsUnsavedChangesGuard(extContainerRef) {
  useEffect(() => {
    const menuCtrl = Ext.getApplication().getController('Menu');

    // These flags help prevent double-modals and unwanted navigation loops
    let skipNextPop = false;
    let isHandlingUnsaved = false;

    function goToHash(hash) {
      window.__extjsHandledUnsaved__ = false;
      skipNextPop = true;
      if (hash && hash !== '#') {

        try {
          const bookmark = NX.Bookmarks.fromToken(hash);
          if (bookmark) {
            NX.Bookmarks.navigateTo(bookmark);
          } else {
            console.warn('[goToHash] Could not create bookmark from hash:', hash);
            window.location.hash = hash;
          }
        } catch (err) {
          console.error('[goToHash] Error with bookmark navigation:', err);
          window.location.hash = hash;
        }
      } else {
        // If the hash is just '#', we don't want to change the URL
        console.warn('Attempted to navigate to a hash-only URL, which breaks the UI');
      }
    }

    // Intercept React-side <a href="#..."> clicks outside the ExtJS container
    // If there are unsaved changes, show the ExtJS modal before allowing navigation
    const clickHandler = e => {
      if (e.defaultPrevented || e.button !== 0) return;
      const a = e.target.closest('a[href^="#"]');
      if (!a) return;
      // Guard against null ref during fast navigation (component unmounting)
      if (!extContainerRef.current) return;
      if (extContainerRef.current.contains(a)) return; // Let ExtJS handle its own links
      e.preventDefault();
      const newHash = a.getAttribute('href');
      window.__extjsHandledUnsaved__ = true;
      const proceed = handleExtJsUnsavedChanges(menuCtrl, () => {
        goToHash(newHash);
      });
      if (proceed) {
        goToHash(newHash);
      }
    };

    // Intercept browser back/forward navigation
    const popstateHandler = () => {
      if (isHandlingUnsaved) return; // Prevent double-modals
      if (skipNextPop) {
        skipNextPop = false;
        return;
      }
      if (!menuCtrl || typeof menuCtrl.hasDirt !== 'function' || !menuCtrl.hasDirt()) return;
      // Store the previous hash before blocking navigation
      const previousHash = window.location.hash;
      history.forward();
      window.__extjsHandledUnsaved__ = true;
      isHandlingUnsaved = true;
      handleExtJsUnsavedChanges(
        menuCtrl,
        () => {
          window.__extjsHandledUnsaved__ = false;
          skipNextPop = true;
          menuCtrl.getFeatureContent().resetUnsavedChangesFlag(true);
          isHandlingUnsaved = false;
          window.location.hash = previousHash;
        },
        () => {
          window.__extjsHandledUnsaved__ = false;
          isHandlingUnsaved = false;
        }
      );
    };

    document.body.addEventListener('click', clickHandler, { capture: true });
    window.addEventListener('popstate', popstateHandler);

    return () => {
      document.body.removeEventListener('click', clickHandler, { capture: true });
      window.removeEventListener('popstate', popstateHandler);
    };
  }, [extContainerRef]);
}

function clearFormDirty(menuCtrl) {
  const content = menuCtrl.getFeatureContent();

  // Reset all forms marked with settingsForm=true, not just the first one
  // Check Ext and ComponentQuery exist (may not be available in tests)
  if (window.Ext && window.Ext.ComponentQuery) {
    const forms = window.Ext.ComponentQuery.query('form[settingsForm=true]');
    forms.forEach(form => {
      const basicForm = form.getForm();
      if (basicForm) {
        basicForm.reset();
      }
    });
  }

  // Also clear the React dirty state
  window.dirty = [];

  // Reset the content flag to prevent duplicate modals
  if (content && content.resetUnsavedChangesFlag) {
    content.resetUnsavedChangesFlag();
  }
}
