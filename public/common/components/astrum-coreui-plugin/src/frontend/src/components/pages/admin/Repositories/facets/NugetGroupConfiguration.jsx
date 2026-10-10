
import React, {useEffect, useMemo, useState} from 'react';

import Axios from 'axios';

import {FormUtils, ExtJS} from '@globalart/astrum-ui-plugin';

import {useRepositoriesService} from '../RepositoriesContextProvider';

import {AsFormGroup, AsStatefulTransferList, AsReadOnly, AsWarningAlert} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {
  EDITOR,
  EDITOR: {NUGET}
} = UIStrings.REPOSITORIES;

export default function NugetGroupConfiguration({parentMachine}) {
  const [repositoriesState, repositoriesSend] = useRepositoriesService();

  const allRepositories = repositoriesState.context.data;

  useEffect(() => {
    !allRepositories?.length && repositoriesSend({type: 'LOAD'});
  }, []);

  const [parentState, sendParent] = parentMachine;

  const {
    data: {
      group: {memberNames = []},
      url
    },
    pristineData: {name}
  } = parentState.context;
  const isEdit = !!name;

  const chocolateyEnabled = useMemo(() => {
    try {
      return ExtJS.state().getValue('nugetChocolateyEnabled') === true;
    } catch {
      return false;
    }
  }, []);

  const [nugetProxyVersions, setNugetProxyVersions] = useState({});

  useEffect(() => {
    Axios.get('/service/rest/beta/repositories').then(({data}) => {
      const versions = {};
      data.forEach((repo) => {
        if (repo.format === 'nuget' && repo.type === 'proxy' && repo.nugetProxy?.nugetVersion) {
          versions[repo.name] = repo.nugetProxy.nugetVersion;
        }
      });
      setNugetProxyVersions(versions);
    }).catch((error) => {
      console.error('Failed to load repository list for NuGet version detection', error);
    });
  }, []);

  const availableGroupMembers = useMemo(
    () =>
      allRepositories
        ?.filter((repo) => repo.format === 'nuget' && repo.name !== name)
        ?.map((repo) => ({id: repo.name, displayName: repo.name})) || [],
    [allRepositories]
  );

  const mixedVersionConflict = useMemo(() => {
    let firstMemberName = null;
    let firstMemberVersion = null;
    for (const memberName of memberNames) {
      const version = nugetProxyVersions[memberName];
      if (!version) {
        continue;
      }
      if (!firstMemberName) {
        firstMemberName = memberName;
        firstMemberVersion = version;
      } else if (version !== firstMemberVersion) {
        return {
          conflictingName: memberName,
          conflictingVersion: version === 'V3' ? 'v3' : 'v2',
          firstMemberName,
          firstMemberVersion: firstMemberVersion === 'V3' ? 'v3' : 'v2'
        };
      }
    }
    return null;
  }, [memberNames, nugetProxyVersions]);

  return (
    <>
      {isEdit && url && (
        <AsReadOnly>
          <AsReadOnly.Label>{NUGET.SYMSRV_ENDPOINT.LABEL}</AsReadOnly.Label>
          <AsReadOnly.Data>{`${url}/symbols`}</AsReadOnly.Data>
        </AsReadOnly>
      )}
      <h2 className="as-h2">{EDITOR.GROUP_CAPTION}</h2>
      <AsFormGroup label={EDITOR.MEMBERS_LABEL} isRequired>
        <AsStatefulTransferList
          allItems={availableGroupMembers}
          selectedItems={availableGroupMembers.length ? memberNames : []}
          onChange={FormUtils.handleUpdate('group.memberNames', sendParent)}
          allowReordering
        />
      </AsFormGroup>
      {mixedVersionConflict && !chocolateyEnabled && (
        <AsWarningAlert>
          {NUGET.MIXED_VERSION_WARNING(
            mixedVersionConflict.conflictingName,
            mixedVersionConflict.conflictingVersion,
            mixedVersionConflict.firstMemberName,
            mixedVersionConflict.firstMemberVersion
          )}
        </AsWarningAlert>
      )}
    </>
  );
}
