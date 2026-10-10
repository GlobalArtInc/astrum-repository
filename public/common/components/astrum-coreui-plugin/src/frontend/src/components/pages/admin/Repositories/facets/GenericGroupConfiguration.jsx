
import React, {useEffect} from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {useRepositoriesService} from '../RepositoriesContextProvider';

import {AsFormGroup, AsStatefulTransferList} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function GenericGroupConfiguration({parentMachine}) {
  const [repositoriesState, repositoriesSend] = useRepositoriesService();

  const repositories = repositoriesState.context.data;

  useEffect(() => {
    !repositories?.length && repositoriesSend({type: 'LOAD'});
  }, []);

  const [parentState, sendParent] = parentMachine;

  const {
    data: {
      format,
      group: {memberNames = []}
    },
    pristineData: {name}
  } = parentState.context;

  const availableRepositories =
    repositories
      ?.filter((repo) => repo.name !== name && repo.format === format)
      ?.map((repo) => ({id: repo.name, displayName: repo.name})) || [];

  return (
    <>
      <h2 className="as-h2">{EDITOR.GROUP_CAPTION}</h2>
      <AsFormGroup label={EDITOR.MEMBERS_LABEL} isRequired>
        <AsStatefulTransferList
          allItems={availableRepositories}
          selectedItems={availableRepositories.length ? memberNames : []}
          onChange={FormUtils.handleUpdate('group.memberNames', sendParent)}
          allowReordering
        />
      </AsFormGroup>
    </>
  );
}
