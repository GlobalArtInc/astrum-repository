
import React from 'react';
import {useActor} from '@xstate/react';
import {
  NxInfoAlert,
  NxTile,
  NxH2,
  NxReadOnly,
} from '@globalart/react-shared-components';
import {ReadOnlyField, FormUtils} from '@globalart/astrum-ui-plugin';
import {findAuthMethod} from './LdapServersHelper';
import UIStrings from '../../../../constants/UIStrings';
const {
  LDAP_SERVERS: {FORM: LABELS},
  SETTINGS,
  USE_TRUST_STORE,
} = UIStrings;

export default function LdapServersConfigurationFormReadOnly({actor}) {
  const [state] = useActor(actor);
  const {data} = state.context;

  const authMethod = findAuthMethod(data.authScheme);

  const useTrustStore = FormUtils.readOnlyCheckboxValueLabel(
    data.useTrustStore
  );

  return (
    <>
      <NxInfoAlert>{SETTINGS.READ_ONLY.WARNING}</NxInfoAlert>
      <NxTile>
        <NxTile.Header>
          <NxTile.HeaderTitle>
            <NxH2>{LABELS.CONFIGURATION}</NxH2>
          </NxTile.HeaderTitle>
        </NxTile.Header>
        <NxTile.Content>
          <NxReadOnly>
            <ReadOnlyField label={LABELS.NAME} value={data.name} />
            <NxH2>{LABELS.SETTINGS.LABEL}</NxH2>
            <ReadOnlyField
              label={LABELS.PROTOCOL.LABEL}
              value={data.protocol}
            />
            <ReadOnlyField label={LABELS.HOSTNAME} value={data.host} />
            <ReadOnlyField label={LABELS.PORT} value={data.port} />
            <ReadOnlyField
              label={USE_TRUST_STORE.LABEL}
              value={useTrustStore}
            />
            <ReadOnlyField
              label={LABELS.SEARCH.LABEL}
              value={data.searchBase}
            />
            <ReadOnlyField
              label={LABELS.AUTHENTICATION.LABEL}
              value={authMethod.label}
            />
            <ReadOnlyField
              label={LABELS.SASL_REALM.LABEL}
              value={data.authRealm}
            />
            <ReadOnlyField
              label={LABELS.USERNAME.LABEL}
              value={data.authUsername}
            />
            <NxH2>{LABELS.CONNECTION_RULES.LABEL}</NxH2>
            <ReadOnlyField
              label={LABELS.WAIT_TIMEOUT.LABEL}
              value={data.connectionTimeoutSeconds}
            />
            <ReadOnlyField
              label={LABELS.RETRY_TIMEOUT.LABEL}
              value={data.connectionRetryDelaySeconds}
            />
            <ReadOnlyField
              label={LABELS.MAX_RETRIES.LABEL}
              value={data.maxIncidentsCount}
            />
          </NxReadOnly>
        </NxTile.Content>
      </NxTile>
    </>
  );
}
