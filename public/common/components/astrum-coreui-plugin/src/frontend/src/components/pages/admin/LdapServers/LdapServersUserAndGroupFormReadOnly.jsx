
import React from 'react';
import {useActor} from '@xstate/react';
import {
  AsInfoAlert,
  AsTile,
  AsH2,
  AsReadOnly,
} from '@globalart/astrum-react-shared-components';
import {ReadOnlyField, FormUtils} from '@globalart/astrum-ui-plugin';
import {isDynamicGroup, isStaticGroup} from './LdapServersHelper';
import UIStrings from '../../../../constants/UIStrings';
import {findGroupType} from './LdapServersHelper';

const {
  LDAP_SERVERS: {FORM: LABELS},
  SETTINGS,
} = UIStrings;

export default function LdapServersReadOnly({actor}) {
  const [state] = useActor(actor);
  const {data} = state.context;

  const userSubtree = FormUtils.readOnlyCheckboxValueLabel(data.userSubtree);
  const mapLdap = FormUtils.readOnlyCheckboxValueLabel(data.ldapGroupsAsRoles);
  const groupSubtree = FormUtils.readOnlyCheckboxValueLabel(data.groupSubtree);
  const groupType = findGroupType(data.groupType);

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
            <ReadOnlyField
              label={LABELS.USER_RELATIVE_DN.LABEL}
              value={data.userBaseDn}
            />
            <ReadOnlyField
              label={LABELS.USER_SUBTREE.LABEL}
              value={userSubtree}
            />
            <ReadOnlyField
              label={LABELS.OBJECT_CLASS.LABEL}
              value={data.userObjectClass}
            />
            <ReadOnlyField
              label={LABELS.USER_FILTER.LABEL}
              value={data.userLdapFilter}
            />
            <ReadOnlyField
              label={LABELS.USER_ID_ATTRIBUTE}
              value={data.userIdAttribute}
            />
            <ReadOnlyField
              label={LABELS.REAL_NAME_ATTRIBUTE}
              value={data.userRealNameAttribute}
            />
            <ReadOnlyField
              label={LABELS.EMAIL_ATTRIBUTE}
              value={data.userEmailAddressAttribute}
            />
            <ReadOnlyField
              label={LABELS.PASSWORD_ATTRIBUTE.LABEL}
              value={data.userPasswordAttribute}
            />
            <ReadOnlyField label={LABELS.MAP_LDAP.LABEL} value={mapLdap} />
            <ReadOnlyField
              label={LABELS.GROUP_TYPE.LABEL}
              value={groupType?.label}
            />
            {isDynamicGroup(data.groupType) && (
              <ReadOnlyField
                label={LABELS.GROUP_MEMBER_OF_ATTRIBUTE.LABEL}
                value={data.userMemberOfAttribute}
              />
            )}
            {isStaticGroup(data.groupType) && (
              <>
                <ReadOnlyField
                  label={LABELS.GROUP_DN.LABEL}
                  value={data.groupBaseDn}
                />
                <ReadOnlyField
                  label={LABELS.GROUP_SUBTREE.LABEL}
                  value={groupSubtree}
                />
                <ReadOnlyField
                  label={LABELS.GROUP_OBJECT_CLASS.LABEL}
                  value={data.groupObjectClass}
                />
                <ReadOnlyField
                  label={LABELS.GROUP_ID_ATTRIBUTE.LABEL}
                  value={data.groupIdAttribute}
                />
                <ReadOnlyField
                  label={LABELS.GROUP_MEMBER_ATTRIBUTE.LABEL}
                  value={data.groupMemberAttribute}
                />
                <ReadOnlyField
                  label={LABELS.GROUP_MEMBER_FORMAT.LABEL}
                  value={data.groupMemberFormat}
                />
              </>
            )}
          </AsReadOnly>
        </AsTile.Content>
      </AsTile>
    </>
  );
}
