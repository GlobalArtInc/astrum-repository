
import React from 'react';
import {useMachine} from '@xstate/react';

import {ExtJS, DateUtils} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsButtonBar,
  AsFooter,
  AsH2,
  AsH3,
  AsModal,
  AsP,
  AsTable,
  AsTableHead,
  AsTableRow,
  AsTableCell,
  AsTableBody,
  AsWarningAlert,
} from '@globalart/astrum-react-shared-components';

import {
  ContentBody,
  Page,
  PageHeader,
  PageTitle,
  PageActions,
  Section
} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
import RecoveryModeMachine from './RecoveryModeMachine';

const {LABELS, MENU, TABLE, CONFIRMATION_MODAL} = UIStrings.RECOVERY_MODE;

export default function RecoveryMode() {
  const [current, send] = useMachine(RecoveryModeMachine, {devTools: true});

  if (!ExtJS.useUser()) {
    return null;
  }

  const disable = () => {
    send({ type: 'DISABLE' });
  };

  const confirmDisable = () => {
    send({ type: 'CONFIRM' });
  };

  const cancelDisable = () => {
    send({ type: 'CANCEL' });
  };

  const enable = () => {
    send({ type: 'ENABLE' });
  };
  const onTaskClick = (taskId) => {
    window.location.hash = `admin/system/tasks:${encodeURIComponent(taskId)}`;
  };

  const {data, error} = current.context;
  const isLoading = current.matches('loading');
  const showConfirmModal = current.matches('confirmingDisable');
  const hasRunningTasks = data?.reconcileTasks?.some(
    ({currentState}) => currentState?.startsWith('RUNNING')
  );

  return (
    <Page>
      <PageHeader>
        <PageTitle
          icon={MENU.icon}
          text={MENU.text}
          description={MENU.description}
        />
        <PageActions>
        {data?.enabled ?
          (<AsButton variant="primary" onClick={disable} disabled={hasRunningTasks}>
            <span>{LABELS.DISABLE_BUTTON}</span>
          </AsButton>)
          :
          (<AsButton variant="primary" onClick={enable}>
            <span>{LABELS.ENABLE_BUTTON}</span>
          </AsButton>)
          }
        </PageActions>
      </PageHeader>
      <ContentBody className='as-recovery-mode'>
        {hasRunningTasks && (
          <AsWarningAlert>{LABELS.TASKS_RUNNING_WARNING}</AsWarningAlert>
        )}
        <Section>
          <AsH2>{LABELS.HOW_IT_WORKS}</AsH2>
          <AsP>{LABELS.HOW_IT_WORKS_DESCRIPTION}</AsP>
          <ul>
            {data?.blockedTaskNames?.map((item, index) => (
              <li key={index}>{item}</li>
            ))}
          </ul>
          <AsH3>{LABELS.STATUS_LABEL}</AsH3>
          <AsP>{data?.enabled ? LABELS.STATE_ENABLED : LABELS.STATE_DISABLED}</AsP>
        </Section>

        <Section>
          <AsH2>{LABELS.DATA_REPAIR_TASKS}</AsH2>
          <AsTable>
            <AsTableHead>
              <AsTableRow>
                <AsTableCell>
                  {TABLE.NAME_LABEL}
                </AsTableCell>
                <AsTableCell>
                  {TABLE.STATUS_LABEL}
                </AsTableCell>
                <AsTableCell>
                  {TABLE.LAST_RUN_LABEL}
                </AsTableCell>
                <AsTableCell>
                  {TABLE.LAST_RESULT_LABEL}
                </AsTableCell>
                <AsTableCell chevron/>
              </AsTableRow>
            </AsTableHead>
            <AsTableBody isLoading={isLoading} error={error} emptyMessage={TABLE.EMPTY_MESSAGE}>
              {data?.reconcileTasks?.map(({id, name, currentState, lastRun, lastRunResult}) => (
                  <AsTableRow key={name}  onClick={() => onTaskClick(id)} isClickable>
                    <AsTableCell>{name}</AsTableCell>
                    <AsTableCell>{currentState}</AsTableCell>
                    <AsTableCell>{lastRun ? DateUtils.prettyDateTime(new Date(lastRun)) : ''}</AsTableCell>
                    <AsTableCell>{lastRunResult}</AsTableCell>
                    <AsTableCell chevron />
                  </AsTableRow>
              ))}
            </AsTableBody>
          </AsTable>
        </Section>
      </ContentBody>
      {showConfirmModal && (
        <AsModal aria-label="disable-recovery-mode-confirmation" variant="narrow" onCancel={cancelDisable}>
          <AsModal.Header>
            <AsH2>{CONFIRMATION_MODAL.TITLE}</AsH2>
          </AsModal.Header>
          <AsModal.Content>
            {CONFIRMATION_MODAL.MESSAGE}
          </AsModal.Content>
          <AsFooter>
            <AsButtonBar>
              <AsButton onClick={cancelDisable}>{CONFIRMATION_MODAL.CANCEL_BUTTON}</AsButton>
              <AsButton onClick={confirmDisable} variant="primary">{CONFIRMATION_MODAL.CONFIRM_BUTTON}</AsButton>
            </AsButtonBar>
          </AsFooter>
        </AsModal>
      )}
    </Page>
  );
}
