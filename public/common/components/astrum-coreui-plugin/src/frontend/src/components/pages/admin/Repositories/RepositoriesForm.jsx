
import React from 'react';
import {useMachine} from '@xstate/react';

import {FormUtils, ExtJS, UIStrings as UIStringsGlobal} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsFontAwesomeIcon,
  AsStatefulForm
} from '@globalart/react-shared-components';

import {faDatabase, faTrash} from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  PageHeader,
  PageTitle,
  Section
} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
import './Repositories.scss';

import RepositoriesFormMachine from './RepositoriesFormMachine';
import {DeleteConfirmationModal} from '@globalart/astrum-ui-plugin';

import {getFacets} from './RepositoryFormConfig';

import GenericFormatConfiguration from './facets/GenericFormatConfiguration';
import GenericNameConfiguration from './facets/GenericNameConfiguration';

export default function RepositoriesForm({itemId, onDone = () => {}}) {
  const isEdit = Boolean(itemId);
  const stateMachine = useMachine(RepositoriesFormMachine, {
    context: {
      pristineData: {
        name: itemId
      }
    },
    actions: {
      onSaveSuccess: onDone,
      onDeleteSuccess: onDone
    },
    devTools: true
  });

  const [current, send] = stateMachine;

  const {
    data: {format, type, name},
    showDeleteModal
  } = current.context;

  const {EDITOR} = UIStrings.REPOSITORIES;

  const canDelete =
    format && ExtJS.checkPermission(`astrum:repository-admin:${format}:${itemId}:delete`);

  const confirmDelete = () => canDelete && send({type: 'CONFIRM_DELETE'});

  const repositoryFacets = getFacets(format, type);

  return (
    <div className="as-repository-editor">
      <PageHeader>
        <PageTitle icon={faDatabase} {...(isEdit ? EDITOR.EDIT_TITLE : EDITOR.CREATE_TITLE)} />
      </PageHeader>
      <ContentBody>
        <Section className="as-repository-editor-form">
          <AsStatefulForm
            {...FormUtils.formProps(current, send)}
            onCancel={onDone}
            submitBtnText={isEdit ? EDITOR.SAVE_BUTTON : EDITOR.CREATE_BUTTON}
            additionalFooterBtns={
              isEdit && (
                <AsButton
                  type="button"
                  variant="tertiary"
                  onClick={confirmDelete}
                  className={!canDelete ? 'disabled' : ''}
                  title={!canDelete ? UIStringsGlobal.PERMISSION_ERROR : ''}
                >
                  <AsFontAwesomeIcon icon={faTrash} />
                  <span>{UIStrings.SETTINGS.DELETE_BUTTON_LABEL}</span>
                </AsButton>
              )
            }
          >
            <GenericFormatConfiguration parentMachine={stateMachine} />
            {format && type && (
              <>
                <GenericNameConfiguration parentMachine={stateMachine} />
                {repositoryFacets.map((Facet, index) => (
                  <Facet parentMachine={stateMachine} key={index} />
                ))}
              </>
            )}
          </AsStatefulForm>
        </Section>
      </ContentBody>

      <DeleteConfirmationModal
        open={showDeleteModal}
        onClose={() => send('HIDE_DELETE_MODAL')}
        onConfirm={() => send('DELETE')}
        entityName={name}
        entityType="repository"
        loading={current.matches('delete')}
      />
    </div>
  );
}
