
import React from 'react';
import {useMachine} from '@xstate/react';

import {FormUtils} from '@globalart/astrum-ui-plugin';
import {
  NxButton,
  NxTooltip,
  NxStatefulForm,
  NxStatefulTransferList,
} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

import ProprietaryRepositoriesMachine from './ProprietaryRepositoriesMachine';

const {PROPRIETARY_REPOSITORIES: {CONFIGURATION: LABELS, HELP_TEXT}, SETTINGS} = UIStrings;

export default function ProprietaryRepositories() {
  const [current, send] = useMachine(ProprietaryRepositoriesMachine, {devTools: true});
  const {data: {enabledRepositories}, possibleRepos, isPristine} = current.context;

  function discard() {
    send({type: 'RESET'});
  }

  const repositories = possibleRepos?.map((it) => ({id: it.id, displayName: it.name})) || [];

  return <NxStatefulForm
      {...FormUtils.formProps(current, send)}
      additionalFooterBtns={<>
        <NxTooltip title={FormUtils.discardTooltip({isPristine})}>
          <NxButton type="button" className={isPristine && 'disabled'} onClick={discard}>
            {SETTINGS.DISCARD_BUTTON_LABEL}
          </NxButton>
        </NxTooltip>
      </>}>
    {() => <>
      <p>{HELP_TEXT}</p>
      <NxStatefulTransferList
          id="proprietary_repositories_select"
          availableItemsLabel={LABELS.AVAILABLE_TITLE}
          selectedItemsLabel={LABELS.SELECTED_TITLE}
          allItems={repositories}
          selectedItems={enabledRepositories}
          onChange={FormUtils.handleUpdate('enabledRepositories', send)}
          showMoveAll
      />
    </>}
  </NxStatefulForm>;
}
