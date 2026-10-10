
import React from 'react';
import {useMachine} from '@xstate/react';
import {faServer} from '@fortawesome/free-solid-svg-icons';

import {FormUtils, ReadOnlyField} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsTooltip,
  AsTextInput,
  AsFormGroup,
  AsStatefulForm,
  AsReadOnly,
  AsTile,
} from '@globalart/astrum-react-shared-components';

import {
  ContentBody,
  Page,
  PageHeader,
  PageTitle
} from '@globalart/astrum-ui-plugin';

import DataStoreConfigurationMachine from './DataStoreConfigurationMachine';
import UIStrings from '../../../../constants/UIStrings';

const FIELDS = UIStrings.DATASTORE_CONFIGURATION.FIELDS;

export default function DataStoreConfiguration() {
  const [state, send] = useMachine(DataStoreConfigurationMachine, {
    devTools: true,
  });

  const {isPristine, data} = state.context;
  const discard = () => send({type: 'RESET'});

  return (
    <Page>
      <PageHeader>
        {/*icon does not match what's in constants faServer vs faDatabase */}
        <PageTitle
          icon={faServer}
          text={UIStrings.DATASTORE_CONFIGURATION.MENU.text}
          description={UIStrings.DATASTORE_CONFIGURATION.MENU.description}
        />
      </PageHeader>
      <ContentBody className="as-datastore-configuration">
        <AsTile className="user-account-settings">
          <AsStatefulForm
            {...FormUtils.formProps(state, send)}
            additionalFooterBtns={
              <AsTooltip title={FormUtils.discardTooltip({isPristine})}>
                <AsButton
                  type="button"
                  className={isPristine && 'disabled'}
                  onClick={discard}
                >
                  {UIStrings.SETTINGS.DISCARD_BUTTON_LABEL}
                </AsButton>
              </AsTooltip>
            }
          >
            <AsReadOnly>
              <ReadOnlyField label={FIELDS.jdbcUrlLabel} value={data.jdbcUrl} />
              <ReadOnlyField
                label={FIELDS.usernameLabel}
                value={data.username}
              />
              <ReadOnlyField label={FIELDS.schemaLabel} value={data.schema} />
            </AsReadOnly>
            <AsFormGroup label={FIELDS.maxConnectionPoolLabel}>
              <AsTextInput
                {...FormUtils.fieldProps('maximumConnectionPool', state)}
                onChange={FormUtils.handleUpdate('maximumConnectionPool', send)}
              />
            </AsFormGroup>
            <AsFormGroup label={FIELDS.advancedLabel}>
              <AsTextInput
                {...FormUtils.fieldProps('advanced', state)}
                onChange={FormUtils.handleUpdate('advanced', send)}
                className="as-text-input--long"
                type="textarea"
              />
            </AsFormGroup>
          </AsStatefulForm>
        </AsTile>
      </ContentBody>
    </Page>
  );
}
