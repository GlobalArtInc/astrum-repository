
import React, {useMemo} from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {NxFormGroup, NxFormSelect} from '@globalart/react-shared-components';

import {useRepositoriesService} from '../RepositoriesContextProvider';

import UIStrings from '../../../../../constants/UIStrings';

const {WRITABLE} = UIStrings.REPOSITORIES.EDITOR;

export default function WritableRepositoryConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  const {
    group: {memberNames, writableMember}
  } = parentState.context.data;

  const [repositoriesState] = useRepositoriesService();

  const repositories = repositoriesState.context.data;

  const hostedDockerRepos = useMemo(() => getHostedDockerRepos(repositories), [repositories]);

  const availableWritableRepos = memberNames.filter((name) => hostedDockerRepos.includes(name));

  if (writableMember && !availableWritableRepos.includes(writableMember)) {
    availableWritableRepos.push(writableMember);
  }

  return (
    <NxFormGroup
      label={WRITABLE.LABEL}
      sublabel={WRITABLE.SUBLABEL}
    >
      <NxFormSelect
        {...FormUtils.selectProps('group.writableMember', parentState)}
        onChange={FormUtils.handleUpdate('group.writableMember', sendParent)}
        disabled={!availableWritableRepos.length}
      >
        <option value="">{WRITABLE.PLACEHOLDER}</option>
        {availableWritableRepos.map((repo) => (
          <option key={repo} value={repo}>
            {repo}
          </option>
        ))}
      </NxFormSelect>
    </NxFormGroup>
  );
}

const getHostedDockerRepos = (repositories) =>
  repositories
    .filter(({type, format}) => type === 'hosted' && format === 'docker')
    .map(({name}) => name);
