
import React from 'react';
import {useActor} from '@xstate/react';

import {
  FormUtils,
  ValidationUtils,
} from '@globalart/astrum-ui-plugin';
import {
  AsFormGroup,
  AsH2,
  AsTextInput,
  AsStatefulForm,
  AsFormSelect,
  AsTile,
  AsCheckbox,
} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';
import {NOTIFICATION_CONDITIONS} from './TasksHelper';
import TasksSummary from './TasksSummary';

const {TASKS: {FORM: LABELS}} = UIStrings;

export default function TasksForm({service, onDone}) {
  const [state, send] = useActor(service);

  const {
    data,
    pristineData,
    runnable,
    stoppable,
    types,
  } = state.context;

  const isCreate = ValidationUtils.isBlank(pristineData.id);
  const isEdit = !isCreate;
  const isTypeSelected = Boolean(data.typeId);

  const confirmDelete = () => send({type: 'CONFIRM_DELETE'});
  const confirmRun = () => send({type: 'CONFIRM_RUN'});
  const confirmStop = () => send({type: 'CONFIRM_STOP'});
  const cancel = () => onDone();

  return (
      <>
        {isEdit && data?.id &&
            <TasksSummary
                task={pristineData}
                runnable={runnable}
                stoppable={stoppable}
                onDelete={confirmDelete}
                onRun={confirmRun}
                onStop={confirmStop}
            />
        }
        <AsTile>
          <AsTile.Content>
            <AsStatefulForm
                {...FormUtils.formProps(state, send)}
                onCancel={cancel}
            >
              <AsH2>{LABELS.SECTIONS.SETTINGS}</AsH2>
              {isCreate && (
                  <AsFormGroup label={LABELS.TYPE.LABEL} isRequired>
                    <AsFormSelect
                        {...FormUtils.fieldProps('typeId', state)}
                        onChange={FormUtils.handleUpdate('typeId', send)}
                        disabled={isEdit}
                    >
                      <option disabled={isTypeSelected} value=""/>
                      {types?.map(({id, name}) =>
                          <option key={id} value={id}>{name}</option>
                      )}
                    </AsFormSelect>
                  </AsFormGroup>
              )}
              {(isEdit || isTypeSelected) && <>
                <AsFormGroup label={LABELS.ENABLED.LABEL}>
                  <AsCheckbox
                      {...FormUtils.checkboxProps('enabled', state)}
                      onChange={FormUtils.handleUpdate('enabled', send)}>
                    {LABELS.ENABLED.SUB_LABEL}
                  </AsCheckbox>
                </AsFormGroup>
                <AsFormGroup
                    label={LABELS.NAME.LABEL}
                    sublabel={LABELS.NAME.SUB_LABEL}
                    isRequired
                >
                  <AsTextInput
                      {...FormUtils.fieldProps('name', state)}
                      onChange={FormUtils.handleUpdate('name', send)}
                  />
                </AsFormGroup>
                <AsFormGroup
                    label={LABELS.EMAIL.LABEL}
                    sublabel={LABELS.EMAIL.SUB_LABEL}
                >
                  <AsTextInput
                      {...FormUtils.fieldProps('alertEmail', state)}
                      onChange={FormUtils.handleUpdate('alertEmail', send)}
                  />
                </AsFormGroup>
                <AsFormGroup
                    label={LABELS.SEND_NOTIFICATION_ON.LABEL}
                    sublabel={LABELS.SEND_NOTIFICATION_ON.SUB_LABEL}
                >
                  <AsFormSelect
                      {...FormUtils.fieldProps('notificationCondition', state)}
                      onChange={FormUtils.handleUpdate('notificationCondition', send)}
                  >
                    {Object.values(NOTIFICATION_CONDITIONS).map(({ID, LABEL}) =>
                        <option key={ID} value={ID}>{LABEL}</option>
                    )}
                  </AsFormSelect>
                </AsFormGroup>
              </>}
            </AsStatefulForm>
          </AsTile.Content>
        </AsTile>
      </>
  );
}
