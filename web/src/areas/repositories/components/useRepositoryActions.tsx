import { useState } from 'react';
import { api, path, useInvalidateApi } from '../../../api';
import { useSession } from '../../../auth/state';
import { useT } from '../../../i18n';
import { copyText, describeError, useConfirm, useToast } from '../../../ui';
import { documentPath, repositoryPath, UI_REPOSITORIES, type UiRepository } from '../api';
import type { RepositoryDocument } from '../model/document';
import { apiFormat } from '../model/spec';
import { DeleteRepositoryDialog } from './DeleteRepositoryDialog';

export function canRebuild(repository: UiRepository): boolean {
  return repository.permissions.adminEdit && repository.type !== 'group';
}

export function canInvalidate(repository: UiRepository): boolean {
  return repository.permissions.adminEdit && repository.type !== 'hosted';
}

export function useRepositoryActions(options: { onDeleted?(name: string): void } = {}) {
  const t = useT();
  const toast = useToast();
  const confirm = useConfirm();
  const invalidate = useInvalidateApi();
  const { state } = useSession();
  const [deleting, setDeleting] = useState<UiRepository | null>(null);

  const docPath = (repository: UiRepository) =>
    documentPath(apiFormat(repository.format, state.formats), repository.type, repository.name);

  const refresh = async (repository: UiRepository) => {
    await Promise.all([invalidate(UI_REPOSITORIES), invalidate(docPath(repository))]);
  };

  const toggleOnline = async (repository: UiRepository) => {
    const online = repository.online;
    const agreed = await confirm({
      title: online ? t('repositories.actions.offlineTitle') : t('repositories.actions.onlineTitle'),
      description: online
        ? t('repositories.actions.offlineText', { name: repository.name })
        : t('repositories.actions.onlineText', { name: repository.name }),
      confirmLabel: online ? t('repositories.actions.takeOffline') : t('repositories.actions.bringOnline'),
      danger: online,
    });
    if (!agreed) {
      return;
    }
    try {
      const target = docPath(repository);
      const current = await api.get<RepositoryDocument>(target);
      await api.put(target, { ...current, online: !online });
      toast.success(
        online
          ? t('repositories.actions.nowOffline', { name: repository.name })
          : t('repositories.actions.nowOnline', { name: repository.name }),
      );
      await refresh(repository);
    } catch (error) {
      toast.error(describeError(t, error));
    }
  };

  const maintain = async (repository: UiRepository, action: 'invalidate-cache' | 'rebuild-index') => {
    const invalidating = action === 'invalidate-cache';
    const agreed = await confirm({
      title: invalidating
        ? t('repositories.actions.invalidateTitle')
        : t('repositories.actions.rebuildTitle'),
      description: invalidating
        ? t('repositories.actions.invalidateText', { name: repository.name })
        : t('repositories.actions.rebuildText', { name: repository.name }),
      confirmLabel: invalidating ? t('repositories.actions.invalidate') : t('repositories.actions.rebuild'),
    });
    if (!agreed) {
      return;
    }
    try {
      await api.post(path`/service/rest/v1/repositories/${repository.name}/${action}`);
      toast.success(
        invalidating
          ? t('repositories.actions.invalidated', { name: repository.name })
          : t('repositories.actions.rebuilt', { name: repository.name }),
      );
    } catch (error) {
      toast.error(describeError(t, error));
    }
  };

  const copyUrl = async (repository: UiRepository) => {
    try {
      await copyText(repository.url);
      toast.success(t('repositories.actions.urlCopied'));
    } catch {
      toast.error(t('repositories.actions.copyFailed'));
    }
  };

  const dialog = deleting ? (
    <DeleteRepositoryDialog
      open
      onOpenChange={(open) => {
        if (!open) {
          setDeleting(null);
        }
      }}
      name={deleting.name}
      type={deleting.type}
      groups={deleting.memberOf}
      onDelete={async () => {
        await api.delete(repositoryPath(deleting.name));
        toast.success(t('repositories.actions.deleted', { name: deleting.name }));
        options.onDeleted?.(deleting.name);
        setDeleting(null);
        void invalidate(UI_REPOSITORIES);
      }}
    />
  ) : null;

  return { toggleOnline, maintain, copyUrl, remove: setDeleting, dialog };
}
