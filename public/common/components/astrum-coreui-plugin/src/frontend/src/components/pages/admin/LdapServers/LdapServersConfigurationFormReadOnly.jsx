
import React from 'react';
import {useActor} from '@xstate/react';
import {
  AsInfoAlert,
  AsTile,
  AsH2,
  AsReadOnly,
} from '@globalart/astrum-react-shared-components';
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
      <AsInfoAlert>{SETTINGS.READ_ONLY.WARNING}</AsInfoAlert>
      <AsTile>
        <AsTile.Header>
          <AsTile.HeaderTitle>
            <AsH2>{LABELS.CONFIGURATION}</AsH2>
          </AsTile.HeaderTitle>
        </AsTile.Header>
        <AsTile.Content>
          <AsReadOnly>
            <ReadOnlyField label={LABELS.NAME} value={data.name} />
            <AsH2>{LABELS.SETTINGS.LABEL}</AsH2>
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
            <AsH2>{LABELS.CONNECTION_RULES.LABEL}</AsH2>
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
          </AsReadOnly>
        </AsTile.Content>
      </AsTile>
    </>
  );
}
