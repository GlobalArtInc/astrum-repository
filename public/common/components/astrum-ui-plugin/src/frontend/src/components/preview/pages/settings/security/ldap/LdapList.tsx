

import React, { useState, useCallback, useMemo } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import * as Dialog from '@radix-ui/react-dialog';
import {
  GripVertical,
  Pencil,
  Search,
  Loader2,
  Info,
  ExternalLink,
  Trash2,
  RefreshCw,
  ArrowUpDown,
  ChevronUp,
  ChevronDown,
} from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { SettingsButton } from '../../../../shared/form';
import { ConfirmDialog } from '../shared/ConfirmDialog';
import { LdapServer, LdapListProps } from './types';

import './LdapList.scss';
import LdapStrings from '../../../../constants/pages/settings/security/LdapStrings';
import CommonStrings from '../../../../constants/CommonStrings';

/**
 * LdapList - Displays LDAP servers in a reorderable list
 */
export function LdapList({
  servers,
  onSelect,
  onCreate,
  onReorder,
  onDelete,
  onClearCache,
  loading = false,
}: LdapListProps) {
  const [filter, setFilter] = useState('');
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null);
  const [localServers, setLocalServers] = useState<LdapServer[]>(servers);
  const [showDeleteModal, setShowDeleteModal] = useState(false);
  const [serverToDelete, setServerToDelete] = useState<LdapServer | null>(null);
  const [showOrderModal, setShowOrderModal] = useState(false);
  const [orderServers, setOrderServers] = useState<LdapServer[]>([]);
  const [isReordering, setIsReordering] = useState(false);

  const canCreate = ExtJS.checkPermission('astrum:ldap:create');
  const canUpdate = ExtJS.checkPermission('astrum:ldap:update');
  const canDelete = ExtJS.checkPermission('astrum:ldap:delete');

  // Update local servers when prop changes
  React.useEffect(() => {
    setLocalServers(servers);
  }, [servers]);

  // Filtered servers
  const filteredServers = useMemo(() => {
    if (!filter) return localServers;
    const search = filter.toLowerCase();
    return localServers.filter((server, index) => {
      const url = (server.url || `${server.protocol}://${server.host}:${server.port}`).toLowerCase();
      // Match the order number as displayed (live position after drag-reorder), not the stale
      // server.order from the API — the Order column shows index + 1, so filter on the same value.
      return (
        server.name.toLowerCase().includes(search) ||
        server.host.toLowerCase().includes(search) ||
        url.includes(search) ||
        String(index + 1).includes(search)
      );
    });
  }, [localServers, filter]);

  // Drag handlers
  const handleDragStart = useCallback((e: React.DragEvent, index: number) => {
    setDraggedIndex(index);
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', index.toString());
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  }, []);

  const handleDrop = useCallback(async (e: React.DragEvent, dropIndex: number) => {
    e.preventDefault();
    if (draggedIndex === null || draggedIndex === dropIndex || isReordering) {
      setDraggedIndex(null);
      return;
    }

    const newServers = [...localServers];
    const [removed] = newServers.splice(draggedIndex, 1);
    newServers.splice(dropIndex, 0, removed);
    const previous = localServers;
    setLocalServers(newServers);
    // Drag indicator cleared before await; row order reverts on API failure.
    setDraggedIndex(null);
    setIsReordering(true);

    const serverNames = newServers.map((s) => s.name).filter(Boolean);
    try {
      await onReorder(serverNames);
    } catch {
      setLocalServers(previous);
    } finally {
      setIsReordering(false);
    }
  }, [draggedIndex, localServers, onReorder, isReordering]);

  const handleDragEnd = useCallback(() => {
    setDraggedIndex(null);
  }, []);

  const handleRowClick = useCallback((server: LdapServer) => {
    onSelect(server);
  }, [onSelect]);

  const handleDeleteClick = useCallback((e: React.MouseEvent, server: LdapServer) => {
    e.stopPropagation();
    setServerToDelete(server);
    setShowDeleteModal(true);
  }, []);

  const handleDeleteConfirm = useCallback(() => {
    if (serverToDelete) {
      onDelete(serverToDelete);
      setShowDeleteModal(false);
      setServerToDelete(null);
    }
  }, [serverToDelete, onDelete]);

  // Change order modal handlers
  const handleOpenOrderModal = useCallback(() => {
    setOrderServers([...localServers]);
    setShowOrderModal(true);
  }, [localServers]);

  const handleOrderDragStart = useCallback((e: React.DragEvent, index: number) => {
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', index.toString());
  }, []);

  const moveOrderItem = useCallback((fromIndex: number, toIndex: number) => {
    setOrderServers((prev) => {
      if (toIndex < 0 || toIndex >= prev.length) return prev;
      const newServers = [...prev];
      const [removed] = newServers.splice(fromIndex, 1);
      newServers.splice(toIndex, 0, removed);
      return newServers;
    });
  }, []);

  const handleOrderDrop = useCallback((e: React.DragEvent, dropIndex: number) => {
    e.preventDefault();
    const dragIndex = parseInt(e.dataTransfer.getData('text/plain'), 10);
    if (dragIndex === dropIndex) return;
    moveOrderItem(dragIndex, dropIndex);
  }, [moveOrderItem]);

  const handleSaveOrder = useCallback(async () => {
    const serverNames = orderServers.map((s) => s.name).filter(Boolean);
    const previous = localServers;
    setLocalServers(orderServers);
    setShowOrderModal(false);
    setIsReordering(true);
    try {
      await onReorder(serverNames);
    } catch {
      // Reopen the modal so the user can retry. The error reason is surfaced
      // via the page-level banner (useLdapApi sets error state; LdapPage
      // renders it when viewMode === 'list').
      setLocalServers(previous);
      setShowOrderModal(true);
    } finally {
      setIsReordering(false);
    }
  }, [orderServers, localServers, onReorder]);

  return (
    <Box className="ldap-list">
      {/* Toolbar */}
      <Flex justify="between" align="center" gap="3" className="ldap-list__toolbar">
        <Box className="ldap-list__search">
          <Search size={16} className="ldap-list__search-icon" aria-hidden="true" />
          <input
            type="text"
            placeholder={LdapStrings.LIST.FILTER_NAME_URL_PLACEHOLDER}
            aria-label={LdapStrings.LIST.FILTER_SERVERS_ARIA}
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            className="ldap-list__search-input"
            data-testid="ldap-search-input"
            data-analytics-id="as-ldap-list-filter"
          />
        </Box>
        <Flex gap="2">
          {canUpdate && (
            <SettingsButton
              variant="secondary"
              onClick={handleOpenOrderModal}
              disabled={loading || isReordering || servers.length <= 1}
              title={LdapStrings.LIST.CHANGE_SERVER_ORDER}
              data-testid="ldap-change-order-button"
              icon={ArrowUpDown}
            >
              {LdapStrings.LIST.CHANGE_ORDER}
            </SettingsButton>
          )}
          {canDelete && (
            <SettingsButton
              variant="secondary"
              onClick={onClearCache}
              disabled={loading || servers.length === 0}
              title={LdapStrings.LIST.CLEAR_LDAP_CACHE}
              data-testid="ldap-clear-cache-button"
              data-analytics-id="as-ldap-list-clear-cache"
              icon={RefreshCw}
            >
              {LdapStrings.LIST.CLEAR_CACHE}
            </SettingsButton>
          )}
        </Flex>
      </Flex>

      {/* Loading State */}
      {loading && servers.length === 0 && (
        <Flex role="status" aria-live="polite" align="center" justify="center" className="ldap-list__loading">
          <Loader2 size={24} className="ldap-list__spinner" aria-hidden="true" />
          <Text size="2">{LdapStrings.LIST.LOADING_LDAP_SERVERS}</Text>
        </Flex>
      )}

      {/* Empty State */}
      {!loading && servers.length === 0 && (
        <Box className="ldap-list__empty">
          <Text size="2">{LdapStrings.LIST.NO_LDAP_SERVERS_CONFIGURED}</Text>
          {canCreate && (
            <Text size="2" className="ldap-list__empty-hint">
              {LdapStrings.LIST.CLICK_CREATE_LDAP_SERVER_ADD}
            </Text>
          )}
        </Box>
      )}

      {/* Server List */}
      {servers.length > 0 && (
        <Box className="ldap-list__table-wrapper">
          <table className="ldap-list__table">
            <thead>
              <tr>
                <th className="ldap-list__th ldap-list__th--order">{LdapStrings.LIST.ORDER}</th>
                <th className="ldap-list__th">{CommonStrings.LABELS.NAME}</th>
                <th className="ldap-list__th">{LdapStrings.LIST.URL}</th>
                <th className="ldap-list__th ldap-list__th--actions"></th>
              </tr>
            </thead>
            <tbody>
              {filteredServers.map((server) => {
                // Resolve the drag index against localServers by identity, not the filtered map
                // position. When a filter is active the two arrays have different index spaces, so
                // splicing localServers by a filtered index silently corrupts the submitted order.
                const actualIndex = localServers.indexOf(server);
                return (
                <tr
                  key={server.id}
                  draggable={canUpdate && !isReordering}
                  onDragStart={(e) => handleDragStart(e, actualIndex)}
                  onDragOver={handleDragOver}
                  onDrop={(e) => handleDrop(e, actualIndex)}
                  onDragEnd={handleDragEnd}
                  onClick={() => handleRowClick(server)}
                  data-analytics-id="as-ldap-list-select-server"
                  className={`ldap-list__row ${draggedIndex === actualIndex ? 'ldap-list__row--dragging' : ''}`}
                >
                  <td className="ldap-list__td ldap-list__td--order">
                    <Flex align="center" gap="2">
                      {canUpdate && (
                        <GripVertical size={14} className="ldap-list__drag-handle" aria-hidden="true" />
                      )}
                      <span className="ldap-list__order-number">{localServers.indexOf(server) + 1}</span>
                    </Flex>
                  </td>
                  <td className="ldap-list__td ldap-list__td--name">
                    <Text weight="medium">{server.name}</Text>
                  </td>
                  <td className="ldap-list__td ldap-list__td--url">
                    {server.url || `${server.protocol}://${server.host}:${server.port}`}
                  </td>
                  <td className="ldap-list__td ldap-list__td--actions">
                    <Flex align="center" gap="2">
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleRowClick(server);
                        }}
                        className="ldap-list__action-button ldap-list__action-button--edit"
                        title={LdapStrings.LIST.EDIT_SERVER}
                        aria-label={LdapStrings.LIST.EDIT_LDAP_SERVER(server.name)}
                        data-analytics-id="as-ldap-list-edit-server"
                      >
                        <Pencil size={16} aria-hidden="true" />
                      </button>
                      {canDelete && (
                        <button
                          type="button"
                          onClick={(e) => handleDeleteClick(e, server)}
                          className="ldap-list__action-button ldap-list__action-button--delete"
                          title={LdapStrings.LIST.DELETE_SERVER}
                          aria-label={LdapStrings.LIST.DELETE_NAMED(server.name)}
                          data-analytics-id="as-ldap-list-delete-server"
                        >
                          <Trash2 size={16} aria-hidden="true" />
                        </button>
                      )}
                    </Flex>
                  </td>
                </tr>
                );
              })}
            </tbody>
          </table>
        </Box>
      )}

      {/* No matches */}
      {servers.length > 0 && filteredServers.length === 0 && (
        <Box className="ldap-list__no-matches">
          <Text size="2">{LdapStrings.LIST.NO_SERVERS_MATCH_FILTER}</Text>
        </Box>
      )}

      {/* Help Section */}
      <Box className="ldap-list__help">
        <Flex align="center" gap="2" className="ldap-list__help-header">
          <Info size={16} aria-hidden="true" />
          <Text size="2" weight="medium">{LdapStrings.LIST.ABOUT_LDAP_SERVERS}</Text>
        </Flex>
        <Text size="2" className="ldap-list__help-text">
          {LdapStrings.LIST.LDAP_SERVERS_USED_USER_AUTHENTICATION}
        </Text>
        <Text size="2" className="ldap-list__help-text">
          {LdapStrings.LIST.SEE_DOCUMENTATION(
            <a
              href="http://links.globalart.dev/products/astrum/docs/ldap"
              target="_blank"
              rel="noopener noreferrer"
              className="ldap-list__help-link"
            >
              {LdapStrings.LIST.DOCUMENTATION_LINK}
              <ExternalLink size={12} aria-hidden="true" />
            </a>
          )}
        </Text>
      </Box>

      {/* Delete Confirmation Modal */}
      <ConfirmDialog
        open={showDeleteModal}
        testId="delete-ldap-server-dialog"
        onOpenChange={setShowDeleteModal}
        title={LdapStrings.LIST.DELETE_LDAP_SERVER}
        message={LdapStrings.LIST.DELETE_SERVER_CONFIRM(serverToDelete?.name)}
        confirmLabel={LdapStrings.LIST.DELETE_SERVER_BUTTON}
        cancelLabel={CommonStrings.ACTIONS.CANCEL}
        variant="danger"
        onConfirm={handleDeleteConfirm}
        analyticsId="as-ldap-list-delete-confirm"
      />

      {/* Change Order Modal */}
      <Dialog.Root open={showOrderModal} onOpenChange={setShowOrderModal}>
        <Dialog.Portal>
          <Dialog.Overlay className="ldap-list__modal-overlay" />
          <Dialog.Content className="ldap-list__modal ldap-list__modal--order" data-testid="ldap-order-modal">
            <Dialog.Title className="ldap-list__modal-title">
              <Flex align="center" gap="2">
                <ArrowUpDown size={20} aria-hidden="true" />
                {LdapStrings.LIST.CHANGE_SERVER_ORDER_TITLE}
              </Flex>
            </Dialog.Title>
            <Dialog.Description className="ldap-list__modal-description">
              {LdapStrings.LIST.LDAP_SERVERS_QUERIED_ORDER_DURING}
            </Dialog.Description>
            <Box className="ldap-list__order-list">
              {orderServers.map((server, index) => (
                <Box
                  key={server.id}
                  draggable
                  onDragStart={(e) => handleOrderDragStart(e, index)}
                  onDragOver={(e) => e.preventDefault()}
                  onDrop={(e) => handleOrderDrop(e, index)}
                  className="ldap-list__order-item"
                >
                  <Flex align="center" gap="2">
                    <GripVertical size={16} className="ldap-list__drag-handle" aria-hidden="true" />
                    <Text weight="medium">{index + 1}.</Text>
                    <Text>{server.name}</Text>
                    <Flex className="ldap-list__order-item-move-buttons" gap="1">
                      <button
                        type="button"
                        className="ldap-list__order-move-button"
                        aria-label={LdapStrings.LIST.MOVE_UP(server.name)}
                        disabled={index === 0}
                        onClick={() => moveOrderItem(index, index - 1)}
                      >
                        <ChevronUp size={14} aria-hidden="true" />
                      </button>
                      <button
                        type="button"
                        className="ldap-list__order-move-button"
                        aria-label={LdapStrings.LIST.MOVE_DOWN(server.name)}
                        disabled={index === orderServers.length - 1}
                        onClick={() => moveOrderItem(index, index + 1)}
                      >
                        <ChevronDown size={14} aria-hidden="true" />
                      </button>
                    </Flex>
                  </Flex>
                </Box>
              ))}
            </Box>
            <Flex gap="2" justify="end" className="ldap-list__modal-actions">
              <SettingsButton
                variant="secondary"
                onClick={() => setShowOrderModal(false)}
                data-testid="ldap-order-cancel"
                data-analytics-id="as-ldap-order-cancel"
              >
                {CommonStrings.ACTIONS.CANCEL}
              </SettingsButton>
              <SettingsButton
                variant="primary"
                onClick={handleSaveOrder}
                disabled={isReordering}
                data-testid="ldap-order-save"
                data-analytics-id="as-ldap-order-save"
              >
                {LdapStrings.LIST.SAVE_ORDER}
              </SettingsButton>
            </Flex>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </Box>
  );
}

export default LdapList;
