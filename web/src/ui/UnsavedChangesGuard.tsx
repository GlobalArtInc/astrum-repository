import { useEffect } from 'react';
import { useBlocker } from 'react-router';
import { useT } from '../i18n';
import { ConfirmDialog } from './ConfirmDialog';

export function UnsavedChangesGuard({ when }: { when: boolean }) {
  const t = useT();
  const blocker = useBlocker(
    ({ currentLocation, nextLocation }) => when && currentLocation.pathname !== nextLocation.pathname,
  );
  useEffect(() => {
    if (!when) {
      return;
    }
    const warn = (event: BeforeUnloadEvent) => {
      event.preventDefault();
    };
    window.addEventListener('beforeunload', warn);
    return () => window.removeEventListener('beforeunload', warn);
  }, [when]);
  return (
    <ConfirmDialog
      open={blocker.state === 'blocked'}
      onOpenChange={(open) => {
        if (!open && blocker.state === 'blocked') {
          blocker.reset();
        }
      }}
      onConfirm={() => blocker.proceed?.()}
      title={t('ui.unsaved.title')}
      description={t('ui.unsaved.description')}
      confirmLabel={t('ui.unsaved.leave')}
      cancelLabel={t('ui.unsaved.stay')}
      danger
    />
  );
}
