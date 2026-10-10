
import React, { useCallback } from 'react';
import {useMachine} from '@xstate/react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {
  AsButton,
  AsFormGroup,
  AsFormSelect,
  AsStatefulForm,
  AsTextInput
} from '@globalart/react-shared-components';

import {
  ContentBody,
  PageHeader,
  PageTitle,
  Section,
  Page
} from '@globalart/astrum-ui-plugin';

import LoggingConfigurationFormMachine from './LoggingConfigurationFormMachine';

import UIStrings from '../../../../constants/UIStrings';
import { useCurrentStateAndParams, useRouter } from '@uirouter/react';
import { ROUTE_NAMES } from '../../../../routerConfig/routeNames/routeNames';

export default function LoggingConfigurationForm() {
  const router = useRouter();
  const onDone = useCallback(() => router.stateService.go(ROUTE_NAMES.ADMIN.SUPPORT.LOGGING.LIST), [router]);
  const { params } = useCurrentStateAndParams();
  const itemId = params?.itemId;

  const [current, send] = useMachine(LoggingConfigurationFormMachine, {
    context: {
      pristineData: {
        name: itemId
      }
    },

    actions: {
      onCancel: onDone,
      onReset: onDone,
      onSaveSuccess: onDone
    },

    devTools: true
  });

  const {pristineData, data} = current.context;
  const isSaving = current.matches('saving');
  const isResetting = current.matches('resetting');

  function update(field, value) {
    send({
      type: 'UPDATE',
      data: {
        [field]: value
      }
    });
  }

  function updateName(value) {
    update('name', value);
  }

  function updateLevel(value) {
    update('level', value);
  }

  function save() {
    send({type: 'SAVE'});
  }

  function handleEnter(event) {
    if (event.key === 'Enter') {
      save(event);
    }
  }

  function cancel() {
    send({type: 'CANCEL'});
  }

  function reset() {
    send({type: 'RESET'});
  }

  return <Page className="as-logging-configuration">
    <PageHeader>
      <PageTitle
        icon={UIStrings.LOGGING.MENU.icon}
        text={UIStrings.LOGGING.MENU.text}
        description={UIStrings.LOGGING.MENU.description}
      />
    </PageHeader>
    <ContentBody>
      <Section className="as-logging-configuration-form" onKeyPress={handleEnter}>
        <AsStatefulForm
            {...FormUtils.formProps(current, send)}
            onCancel={cancel}
            submitMaskState={(isSaving || isResetting) ? false : null}
            submitMaskMessage={isSaving ? UIStrings.SAVING : UIStrings.LOGGING.MESSAGES.RESETTING}
            additionalFooterBtns={
              itemId && <AsButton type="button" variant="error" onClick={reset}>{UIStrings.LOGGING.RESET_BUTTON}</AsButton>
            }
        >
          {() => <>
            <AsFormGroup label={UIStrings.LOGGING.NAME_LABEL} isRequired>
              <AsTextInput
                  className="as-text-input--long"
                  {...FormUtils.fieldProps('name', current)}
                  disabled={pristineData.name}
                  onChange={updateName}/>
            </AsFormGroup>
            <AsFormGroup label={UIStrings.LOGGING.LEVEL_LABEL}>
              <AsFormSelect name="level" value={data.level} onChange={updateLevel}>
                {['OFF', 'ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'].map(logLevel =>
                    <option key={logLevel} value={logLevel}>{logLevel}</option>
                )}
              </AsFormSelect>
            </AsFormGroup>
          </>}
        </AsStatefulForm>
      </Section>
    </ContentBody>
  </Page>;
}
