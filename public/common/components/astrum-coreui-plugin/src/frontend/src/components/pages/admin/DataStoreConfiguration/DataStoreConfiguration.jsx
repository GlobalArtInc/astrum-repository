
import React from 'react';
import {useMachine} from '@xstate/react';
import {faServer} from '@fortawesome/free-solid-svg-icons';

import {FormUtils, ReadOnlyField} from '@globalart/astrum-ui-plugin';
import {
  NxButton,
  NxTooltip,
  NxTextInput,
  NxFormGroup,
  NxStatefulForm,
  NxReadOnly,
  NxTile,
} from '@globalart/react-shared-components';

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
        <NxTile className="user-account-settings">
          <NxStatefulForm
            {...FormUtils.formProps(state, send)}
            additionalFooterBtns={
              <NxTooltip title={FormUtils.discardTooltip({isPristine})}>
                <NxButton
                  type="button"
                  className={isPristine && 'disabled'}
                  onClick={discard}
                >
                  {UIStrings.SETTINGS.DISCARD_BUTTON_LABEL}
                </NxButton>
              </NxTooltip>
            }
          >
            <NxReadOnly>
              <ReadOnlyField label={FIELDS.jdbcUrlLabel} value={data.jdbcUrl} />
              <ReadOnlyField
                label={FIELDS.usernameLabel}
                value={data.username}
              />
              <ReadOnlyField label={FIELDS.schemaLabel} value={data.schema} />
            </NxReadOnly>
            <NxFormGroup label={FIELDS.maxConnectionPoolLabel}>
              <NxTextInput
                {...FormUtils.fieldProps('maximumConnectionPool', state)}
                onChange={FormUtils.handleUpdate('maximumConnectionPool', send)}
              />
            </NxFormGroup>
            <NxFormGroup label={FIELDS.advancedLabel}>
              <NxTextInput
                {...FormUtils.fieldProps('advanced', state)}
                onChange={FormUtils.handleUpdate('advanced', send)}
                className="nx-text-input--long"
                type="textarea"
              />
            </NxFormGroup>
          </NxStatefulForm>
        </NxTile>
      </ContentBody>
    </Page>
  );
}
