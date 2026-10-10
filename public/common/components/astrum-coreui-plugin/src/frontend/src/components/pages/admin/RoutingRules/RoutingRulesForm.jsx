
import React, { useCallback } from 'react';
import {useMachine} from '@xstate/react';

import {FormUtils} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsErrorAlert,
  AsFontAwesomeIcon,
  AsFormGroup,
  AsFormSelect,
  AsInfoAlert,
  AsLoadWrapper,
  AsStatefulForm,
  AsSuccessAlert,
  AsTextInput,
  AsTooltip,
} from '@globalart/react-shared-components';

import {
  ContentBody,
  PageHeader,
  PageTitle,
  Section,
  Page
} from '@globalart/astrum-ui-plugin';

import RoutingRuleFormMachine from './RoutingRulesFormMachine';

import UIStrings from '../../../../constants/UIStrings';
import {faPlus, faTrash} from '@fortawesome/free-solid-svg-icons';
import RoutingRulesPreview from './RoutingRulesFormPreview';
import RoutingRuleMatcherRow from './RoutingRuleMatcherRow';
import { DeleteConfirmationModal } from '@globalart/astrum-ui-plugin';
import { isEmpty } from 'ramda';

import './RoutingRules.scss';
import { ROUTE_NAMES } from '../../../../routerConfig/routeNames/routeNames';
import { useCurrentStateAndParams, useRouter } from '@uirouter/react';


const {ROUTING_RULES, SETTINGS} = UIStrings;

export default function RoutingRulesForm() {
  const router = useRouter();
  const onDone = useCallback(() => router.stateService.go(ROUTE_NAMES.ADMIN.REPOSITORY.ROUTINGRULES.LIST));
  const { params } = useCurrentStateAndParams();
  const itemId = params?.itemId;
  const [current, send] = useMachine(RoutingRuleFormMachine, {
    context: {
      pristineData: {
        name: itemId
      }
    },

    actions: {
      onCancel: onDone,
      onSaveSuccess: onDone,
      onDeleteSuccess: onDone
    },

    devTools: true
  });

  const {data, path, testError, testResult, showDeleteModal} = current.context;
  const isTesting = current.matches('testing');
  const hasData = data && !isEmpty(data);
  const assignedRepositoryCount = data?.assignedRepositoryCount || 0;
  const assignedRepositoryNames = data?.assignedRepositoryNames || [];
  const isEdit = Boolean(itemId);
  const hasAssignedRepositories = assignedRepositoryCount > 0;

  function addMatcher() {
    send({type: 'ADD_MATCHER'});
  }

  function removeMatcher(index) {
    send({type: 'REMOVE_MATCHER', index});
  }

  function updateMatcher(value, index) {
    send({type: 'UPDATE_MATCHER', index, value})
  }

  function update(value) {
    send({type: 'UPDATE', data: {mode: value}});
  }

  function cancel() {
    send({type: 'CANCEL'});
  }

  function remove(e) {
    if (!e.currentTarget.classList.contains('disabled')) {
      send({type: 'CONFIRM_DELETE'});
    }
  }

  function updatePath(value) {
    send({type: 'UPDATE_PATH', path: value});
  }

  function test() {
    send({type: 'TEST'});
  }

  return <Page className="as-routing-rules">
    {isEdit &&
    <AsInfoAlert>
      {!hasAssignedRepositories && <span dangerouslySetInnerHTML={{__html: ROUTING_RULES.FORM.UNUSED}}/>}
      {hasAssignedRepositories && <span dangerouslySetInnerHTML={{__html: ROUTING_RULES.FORM.USED_BY(assignedRepositoryNames)}}/>}
    </AsInfoAlert>}
    <PageHeader>
      <PageTitle text={itemId ? ROUTING_RULES.FORM.EDIT_TITLE : ROUTING_RULES.FORM.CREATE_TITLE}/>
    </PageHeader>
    <ContentBody>
      <Section className="as-routing-rules-form">
        <AsStatefulForm
            {...FormUtils.formProps(current, send)}
            onCancel={cancel}
            submitBtnText={itemId ? SETTINGS.SAVE_BUTTON_LABEL : ROUTING_RULES.FORM.CREATE_BUTTON}
            additionalFooterBtns={itemId &&
              <AsTooltip title={assignedRepositoryCount > 0 ? ROUTING_RULES.FORM.CANNOT_DELETE(assignedRepositoryNames) : undefined}>
                <AsButton type="button" variant="tertiary" onClick={remove} className={hasAssignedRepositories && 'disabled'}>
                  <AsFontAwesomeIcon icon={faTrash}/>
                  <span>{SETTINGS.DELETE_BUTTON_LABEL}</span>
                </AsButton>
              </AsTooltip>
            }
        >
          {hasData && <>
            <AsFormGroup label={ROUTING_RULES.FORM.NAME_LABEL} isRequired>
              <AsTextInput
                  {...FormUtils.fieldProps('name', current)}
                  onChange={FormUtils.handleUpdate('name', send)}/>
            </AsFormGroup>
            <AsFormGroup label={ROUTING_RULES.FORM.DESCRIPTION_LABEL}>
              <AsTextInput
                  className="as-text-input--long"
                  {...FormUtils.fieldProps('description', current)}
                  onChange={FormUtils.handleUpdate('description', send)}/>
            </AsFormGroup>
            <AsFormGroup
                id="as-routing-rules-mode"
                label={ROUTING_RULES.FORM.MODE_LABEL}
                sublabel={ROUTING_RULES.FORM.MODE_DESCRIPTION}>
              <AsFormSelect {...FormUtils.fieldProps('mode', current)} onChange={update}>
                <option value="ALLOW">{ROUTING_RULES.FORM.MODE.ALLOW}</option>
                <option value="BLOCK">{ROUTING_RULES.FORM.MODE.BLOCK}</option>
              </AsFormSelect>
            </AsFormGroup>
            <div className="as-form-group">
              <div id="matchers-label" className="as-label">
                <span className="as-label__text">{ROUTING_RULES.FORM.MATCHERS_LABEL}</span>
              </div>
              <div className="as-sub-label">{ROUTING_RULES.FORM.MATCHERS_DESCRIPTION}</div>

              {data.matchers?.map((value, index) =>
                  <RoutingRuleMatcherRow
                      key={`matcher-${index}`}
                      label={ROUTING_RULES.FORM.MATCHER_LABEL(index)}
                      {...FormUtils.fieldProps(`matchers[${index}]`, current)}
                      value={value}
                      onChange={(value) => updateMatcher(value, index)}
                      showRemoveButton={data.matchers.length > 1}
                      onRemove={() => removeMatcher(index)}/>)
              }
              <div className="add-matcher">
                <AsButton type="button" variant="tertiary" onClick={addMatcher}>
                  <AsFontAwesomeIcon icon={faPlus}/>
                  <span>{ROUTING_RULES.FORM.ADD_MATCHER_BUTTON}</span>
                </AsButton>
              </div>
            </div>
          </>}
        </AsStatefulForm>
      </Section>

      <Section className="as-routing-rules-preview">
        <RoutingRulesPreview value={path} onChange={updatePath} onTest={test}/>
        <AsLoadWrapper className="preview-result" loading={isTesting} loadError={testError} retryHandler={test}>
          {testResult === true && <AsSuccessAlert>{ROUTING_RULES.FORM.PREVIEW.ALLOWED}</AsSuccessAlert>}
          {testResult === false && <AsErrorAlert>{ROUTING_RULES.FORM.PREVIEW.BLOCKED}</AsErrorAlert>}
        </AsLoadWrapper>
      </Section>
    </ContentBody>

    <DeleteConfirmationModal
      open={showDeleteModal}
      onClose={() => send('HIDE_DELETE_MODAL')}
      onConfirm={() => send('DELETE')}
      entityType="routing rule"
      loading={current.matches('delete')}
    />
  </Page>;
}
