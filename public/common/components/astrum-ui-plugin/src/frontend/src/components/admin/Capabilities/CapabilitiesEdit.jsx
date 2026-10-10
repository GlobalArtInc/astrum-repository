

import React from 'react';
import { useCurrentStateAndParams, useRouter } from '@uirouter/react';
import { ContentBody, Page, PageHeader, PageTitle, Section } from '../../layout';
import { useMachine } from '@xstate/react';
import CapabilitiesEditMachine from './CapabilitiesEditMachine';
import {
  AsInfoAlert,
  AsCheckbox,
  AsErrorAlert,
  AsFormGroup,
  AsH2,
  AsLoadWrapper,
  AsStatefulForm,
  AsTile,
  AsButton,
  AsFontAwesomeIcon,
  AsWarningAlert,
  AsGrid,
  AsReadOnly,
} from '@globalart/react-shared-components';
import { faSpinner, faTrash } from '@fortawesome/free-solid-svg-icons';
import FormUtils from '../../../interface/FormUtils';
import UIStrings from '../../../constants/UIStrings';
import { AdminRouteNames } from '../../../constants/admin/AdminRouteNames';
import DynamicFormField from '../../widgets/DynamicFormField/DynamicFormField';
import CapabilitiesEditDeleteModal from './CapabilitiesEditDeleteModal';
import './Capabilities.scss';
import ExtJS from '../../../interface/ExtJS';
import classNames from 'classnames';
import './CapabilitiesEdit.scss';

const { CAPABILITIES } = UIStrings;

export default function CapabilitiesEdit() {
  const router = useRouter();
  const { params } = useCurrentStateAndParams();
  const id = params.id;

  const [current, send] = useMachine(CapabilitiesEditMachine, {
    devTools: true,
    context: { id },
    actions: {
      onSaveSuccess: () => {
        router.stateService.go(AdminRouteNames.SYSTEM.CAPABILITIES.LIST);
      },
      onDeleteSuccess: () => {
        router.stateService.go(AdminRouteNames.SYSTEM.CAPABILITIES.LIST);
      },
    },
  });

  const isLoading = current.matches('loading');
  const isLoadError = current.matches('loadError');
  const showDeleteConfirmationModal = current.matches('awaitingDeleteConfirmation');

  const { capability, capabilityType, loadError, deleteError } = current.context;

  const isDeleting = current.matches('confirmDelete') || current.matches('delete');
  const isDeleteError = !!deleteError;

  const errorMessage = isLoadError ? loadError || 'An error occurred' : null;

  const handleRetry = () => {
    send({ type: 'RETRY' });
  };

  const updateField = (name, value) => {
    send({
      type: 'UPDATE',
      data: {
        [name]: value,
      },
    });
  };

  const handleEnabledChange = value => {
    updateField('enabled', value);
  };

  const handleCancel = () => {
    router.stateService.go(AdminRouteNames.SYSTEM.CAPABILITIES.LIST);
  };

  const onDeleteClicked = () => {
    send({ type: 'SHOW_DELETE_MODAL' });
  };

  const onConfirmDelete = () => {
    send({ type: 'CONFIRM_DELETE' });
  };

  const onDeleteCanceled = () => {
    send({ type: 'CANCEL_DELETE' });
  };

  const buildTitle = () => {
    if (!capability) {
      return '';
    }

    if (!!capability.description) {
      return `${capability.typeName} - ${capability.description}`;
    } else {
      return capability.typeName;
    }
  };

  const canEdit = ExtJS.checkPermission('astrum:capabilities:edit');
  const canDelete = ExtJS.checkPermission('astrum:capabilities:delete');

  const baseFormProps = FormUtils.formProps(current, send);
  const formProps = {
    ...baseFormProps,
    validationErrors: showDeleteConfirmationModal ? null : baseFormProps.validationErrors,
  };

  return (
    <Page className='as-capabilities-edit-page'>
      <PageHeader>
        <PageTitle text={buildTitle()} description={CAPABILITIES.EDIT.DESCRIPTION} />
      </PageHeader>

      <ContentBody>
        <AsLoadWrapper loading={isLoading} error={errorMessage} retryHandler={handleRetry}>
          <CapabilitiesEditDeleteModal
            showModal={showDeleteConfirmationModal}
            deleteWarningMessage={capability?.deleteWarningMessage}
            onConfirmDelete={onConfirmDelete}
            onDeleteCanceled={onDeleteCanceled}
          />

          {isDeleteError && <AsErrorAlert>{deleteError || 'An error occurred deleting the capability.'}</AsErrorAlert>}
          <SummaryTile canEdit={canEdit} capability={capability} capabilityType={capabilityType} />
          <Section data-testid='capabilities-form-section'>
            <AsStatefulForm
              {...formProps}
              onCancel={handleCancel}
              data-analytics-id='as-edit-capabilities-form'
              className='as-capabilities-form'
              submitBtnClasses={!canEdit && 'as-read-only-submit-button'}
              additionalFooterBtns={canEdit && canDelete && DeleteButton({ capability, isDeleting, onDeleteClicked })}
            >
              <AsTile.Subsection>
                <AsTile.SubsectionHeader>
                  <AsH2>{CAPABILITIES.EDIT.FORM.TITLE}</AsH2>
                </AsTile.SubsectionHeader>
              </AsTile.Subsection>

              <AsTile.Subsection>
                {canEdit ? (
                  <AsFormGroup label={CAPABILITIES.EDIT.FORM.CAPABILITY_STATE}>
                    <AsCheckbox
                      {...FormUtils.checkboxProps('enabled', current, false)}
                      onChange={handleEnabledChange}
                    >
                      {CAPABILITIES.EDIT.FORM.ENABLED_LABEL}
                    </AsCheckbox>
                  </AsFormGroup>
                ) : (
                  <AsReadOnly>
                    <AsReadOnly.Label>{CAPABILITIES.EDIT.FORM.CAPABILITY_STATE}</AsReadOnly.Label>
                    <AsReadOnly.Data>
                      {capability?.enabled ? 'Enabled' : 'Disabled'}
                    </AsReadOnly.Data>
                  </AsReadOnly>
                )}
              </AsTile.Subsection>

              {capabilityType && capabilityType.formFields && capabilityType.formFields.length > 0 && (
                <AsTile.Subsection data-testid='form-fields'>
                  {capabilityType.formFields.map(field => (
                    <DynamicFormField
                      key={`${capabilityType.id}-${field.id}`}
                      id={field.id}
                      current={current}
                      initialValue={capability?.properties?.[field.id] ?? field.initialValue}
                      onChange={updateField}
                      dynamicProps={field}
                      readOnly={!canEdit}
                    />
                  ))}
                </AsTile.Subsection>
              )}
              <AsTile.Subsection>
                <DynamicFormField
                  id='notes'
                  current={current}
                  initialValue={capability?.notes ?? ''}
                  onChange={updateField}
                  dynamicProps={{
                    type: 'text-area',
                    label: CAPABILITIES.EDIT.FORM.NOTES,
                    name: 'notes',
                    required: false,
                    helpText: CAPABILITIES.EDIT.FORM.NOTES_HELP_TEXT,
                    attributes: {
                      long: true,
                    },
                  }}
                  readOnly={!canEdit}
                />
              </AsTile.Subsection>
            </AsStatefulForm>
          </Section>
        </AsLoadWrapper>
      </ContentBody>
    </Page>
  );
}

function SummaryTile({ canEdit, capability, capabilityType }) {
  const inactiveWarning = capability.enabled && !capability?.active ? capability.stateDescription : null;

  const category = capability?.tags?.Category;

  return (
    <Section>
      {!canEdit && <AsInfoAlert>{UIStrings.SETTINGS.READ_ONLY.WARNING}</AsInfoAlert>}
      <AsTile.Header>
        <div className='as-tile-header__title'>
          <AsH2>Capability Summary</AsH2>
        </div>
      </AsTile.Header>

      {inactiveWarning && <AsWarningAlert role='alert'>{inactiveWarning}</AsWarningAlert>}

      <AsTile.Content>
        <AsGrid.Row>
          <ReadOnlyColumn label={CAPABILITIES.EDIT.LABELS.TYPE} value={capability.typeName} />

          <ReadOnlyColumn label={CAPABILITIES.EDIT.LABELS.DESCRIPTION} value={capability.description} />

          <ReadOnlyColumn
            className='summary-tile_state'
            label={CAPABILITIES.EDIT.LABELS.STATE}
            value={capability.state}
          />

          <ReadOnlyColumn label={CAPABILITIES.EDIT.LABELS.CATEGORY} value={category} />
        </AsGrid.Row>

        <AsGrid.Row>
          <ReadOnlyColumn
            gridColNum={100}
            label={CAPABILITIES.EDIT.LABELS.STATUS}
            value={capability.status}
            renderAsHtml={true}
          />
        </AsGrid.Row>

        <AsGrid.Row>
          <ReadOnlyColumn
            gridColNum={100}
            label={CAPABILITIES.EDIT.LABELS.ABOUT}
            value={capabilityType?.about}
            renderAsHtml={true}
          />
        </AsGrid.Row>
      </AsTile.Content>
    </Section>
  );
}

function ReadOnlyColumn({ label, value, gridColNum = 25, renderAsHtml = false, className = '' }) {
  value = value || 'N/A';

  return (
    <AsGrid.Column className={classNames(`as-grid-col--${gridColNum}`, className)}>
      <AsReadOnly>
        <AsReadOnly.Item>
          <AsReadOnly.Label data-testid={label} id={label}>
            {label}
          </AsReadOnly.Label>

          {renderAsHtml ? (
            <AsReadOnly.Data aria-labelledby={label} dangerouslySetInnerHTML={{ __html: value }} />
          ) : (
            <AsReadOnly.Data aria-labelledby={label}>{value}</AsReadOnly.Data>
          )}
        </AsReadOnly.Item>
      </AsReadOnly>
    </AsGrid.Column>
  );
}

function DeleteButton({ capability, isDeleting, onDeleteClicked }) {
  const hasEditPermissions = ExtJS.checkPermission('astrum:capabilities:delete');

  // show the button disabled if the user does not have permission or a delete operation is pending
  const disabled = !hasEditPermissions || isDeleting;

  // Don't render button if capability is not loaded yet or if it's a system capability
  if (!capability || capability.isSystem) {
    // hide the button all together if it's a system capability or capability is not loaded
    return null;
  }

  return (
    <AsButton
      variant='tertiary'
      disabled={disabled}
      data-analytics-id='as-capabilities-edit-page-delete-button'
      onClick={onDeleteClicked}
    >
      {isDeleting ? <AsFontAwesomeIcon icon={faSpinner} spin /> : <AsFontAwesomeIcon icon={faTrash} />}
      <span>Delete</span>
    </AsButton>
  );
}
